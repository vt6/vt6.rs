/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::common::core::msg;
use crate::server;
use crate::server::tokio as my;
use futures::future::{AbortHandle, AbortRegistration};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockWriteGuard};
use tokio::sync::Notify;

struct ConnectionPoolEntry<A: server::Application> {
    conn: server::Connection<A, Dispatch<A>>,
    rx_abort: AbortHandle,
    tx_abort: AbortHandle,
}

struct ConnectionPool<A: server::Application> {
    conns: HashMap<u64, ConnectionPoolEntry<A>>,
    next_connection_id: u64,
}

struct TxConnector {
    //The boxes shall be allocated individually since we pass them around outside the Vec.
    #[allow(clippy::vec_box)]
    bufs: Vec<Box<my::SendBuffer>>,
    notify: Arc<Notify>,
}

pub(crate) struct InnerDispatch<A: server::Application> {
    //NOTE: The `self.pool` lock is semantically dominant over the `self.tx` lock. To prevent
    //deadlocks, the implementation must guarantee that `self.transmit` will only ever be locked
    //when `self.pool` is already locked (both for read locks and for write locks). Across
    //functions, this is usually guaranteed by passing refs to Connection instances around (which
    //can only be obtained by holding the `self.pool` lock).
    path: std::path::PathBuf,
    pub(crate) app: A,
    pool: RwLock<ConnectionPool<A>>,
    tx: RwLock<HashMap<u64, TxConnector>>,
}

impl<A: server::Application> InnerDispatch<A> {
    fn new(path: std::path::PathBuf, app: A) -> Arc<Self> {
        Arc::new(InnerDispatch {
            path,
            app,
            pool: RwLock::new(ConnectionPool {
                conns: HashMap::new(),
                next_connection_id: 0,
            }),
            tx: RwLock::new(HashMap::new()),
        })
    }

    pub(crate) fn dispatch(self: &Arc<Self>) -> Dispatch<A> {
        Dispatch(self.clone())
    }

    fn create_connection_object(
        self: &Arc<Self>,
    ) -> (u64, AbortRegistration, AbortRegistration, Arc<Notify>) {
        let (rx_ah, rx_ar) = AbortHandle::new_pair();
        let (tx_ah, tx_ar) = AbortHandle::new_pair();

        let mut pool = self.pool.write().unwrap();
        let conn_id = pool.next_connection_id;
        pool.next_connection_id += 1;
        let conn = server::Connection::new(self.dispatch(), conn_id);
        pool.conns.insert(
            conn_id,
            ConnectionPoolEntry {
                conn,
                rx_abort: rx_ah,
                tx_abort: tx_ah,
            },
        );
        std::mem::drop(pool); //release the write lock

        let tx_notify = Arc::new(Notify::new());
        let tx_connector = TxConnector {
            notify: tx_notify.clone(),
            bufs: Vec::new(),
        };
        self.tx.write().unwrap().insert(conn_id, tx_connector);

        (conn_id, rx_ar, tx_ar, tx_notify)
    }

    pub(crate) fn connection_mut<'a>(self: &'a Arc<Self>, conn_id: u64) -> ConnectionRefMut<'a, A> {
        ConnectionRefMut {
            inner: self,
            guard: self.pool.write().unwrap(),
            conn_id,
        }
    }

    pub(crate) fn swap_send_buffer(
        self: &Arc<Self>,
        conn: &mut server::Connection<A, Dispatch<A>>,
        buf: Option<Box<my::SendBuffer>>,
    ) -> Option<Box<my::SendBuffer>> {
        //This function is called by the tx job to obtain more data to send. `connector.bufs` is
        //well-ordered, so index 0 contains the next send buffer in line. As an optimization, we
        //allow the tx job to give us the previous buffer back, and we recycle it by putting it at
        //the back of our send buffer queue.

        let mut tx = self.tx.write().unwrap();
        let connector = tx.get_mut(&conn.id())?;

        if let Some(mut buf) = buf {
            buf.clear();
            connector.bufs.push(buf);
        }

        if connector.bufs.iter().all(|b| b.filled_len() == 0) {
            //we don't have any data to send right now
            None
        } else {
            Some(connector.bufs.remove(0))
        }
    }

    fn do_maintenance_on_conn(
        self: &Arc<Self>,
        pool: &mut RwLockWriteGuard<'_, ConnectionPool<A>>,
        conn_id: u64,
    ) {
        //This function is called whenever we are about to drop a
        //ConnectionRefMut obtained from `self.connection_mut(conn_id)`. Since
        //the caller had a mutable reference to the connection with the given
        //ID, the connection state may have changed. Depending on the new state,
        //we need to perform various maintenance tasks on this connection.

        //if the connection has been set to state Teardown, abort the rx/tx jobs
        //(this will close the client connection as the respective halfs of the
        //UnixSocket instance get dropped)
        if let Some(conn_ref) = pool.conns.get(&conn_id) {
            if matches!(conn_ref.conn.state(), server::ConnectionState::Teardown) {
                conn_ref.rx_abort.abort();
                conn_ref.tx_abort.abort();
                pool.conns.remove(&conn_id);
                self.tx.write().unwrap().remove(&conn_id);
                let n = server::Notification::ConnectionClosed;
                self.app.notify(&n);
            }
        }

        //TODO run any pending broadcasts
    }
}

////////////////////////////////////////////////////////////////////////////////
// connection smart pointers
//
// We pass these around instead of bare `&Connection` and `&mut Connection`
// because we want to do some housekeeping whenever the `inner.pool` lock is
// about to release. Currently we only do this for the write lock, but we may do
// it for the read lock as well in the future.

pub(crate) struct ConnectionRefMut<'a, A: server::Application> {
    inner: &'a Arc<InnerDispatch<A>>,
    guard: RwLockWriteGuard<'a, ConnectionPool<A>>,
    conn_id: u64,
}

impl<'a, A: server::Application> ConnectionRefMut<'a, A> {
    pub(crate) fn alive(&mut self) -> Option<&mut server::Connection<A, Dispatch<A>>> {
        self.guard
            .conns
            .get_mut(&self.conn_id)
            .map(|conn_ref| &mut conn_ref.conn)
    }
}

impl<'a, A: server::Application> Drop for ConnectionRefMut<'a, A> {
    fn drop(&mut self) {
        self.inner
            .do_maintenance_on_conn(&mut self.guard, self.conn_id)
    }
}

////////////////////////////////////////////////////////////////////////////////
// public API

///An implementation of [trait Dispatch](../trait.Dispatch.html) using the
///[Tokio library](https://tokio.rs/).
#[derive(Clone)]
pub struct Dispatch<A: server::Application>(Arc<InnerDispatch<A>>);

impl<A: server::Application> Dispatch<A> {
    ///Creates a new instance. The server socket will be opened at the given path.
    pub fn new(path: impl Into<std::path::PathBuf>, app: A) -> std::io::Result<Self> {
        Ok(Dispatch(InnerDispatch::new(path.into(), app)))
    }

    ///Runs the dispatch's event loop. Run this with `tokio::spawn()` or
    ///`tokio::runtime::Runtime::block_on()` etc.
    pub async fn run_listener(&self) -> std::io::Result<()> {
        let listener = tokio::net::UnixListener::bind(&self.0.path)?;

        loop {
            let (stream, _addr) = listener.accept().await?;
            let (stream_reader, stream_writer) = stream.into_split();
            let (conn_id, rx_abort, tx_abort, tx_notify) = self.0.create_connection_object();
            my::spawn_receiver(self.0.clone(), rx_abort, conn_id, stream_reader);
            my::spawn_transmitter(self.0.clone(), tx_abort, conn_id, stream_writer, tx_notify);
            self.0.app.notify(&server::Notification::ConnectionOpened);
        }
    }
}

impl<A: server::Application> server::Dispatch<A> for Dispatch<A> {
    type ConnectionID = u64;

    fn application(&self) -> &A {
        &self.0.app
    }

    fn enqueue_broadcast(&self, _action: Box<dyn Fn(&mut server::Connection<A, Self>)>) {
        //TODO implement enqueue_broadcast
        unimplemented!();
    }

    fn enqueue_message<M: msg::EncodeMessage>(
        &self,
        conn: &mut server::Connection<A, Self>,
        msg: &M,
    ) {
        //NOTE: The mutability of `conn` is only used to enforce that the current thread holds the
        //`self.0.pool` write lock, cf. comment on declaration of `struct InnerDispatch`.
        let mut tx = self.0.tx.write().unwrap();
        let connector = match tx.get_mut(&conn.id()) {
            Some(c) => c,
            //`None` should not happen, since the `inner.pool` and `inner.tx` entries are deleted
            //the same time, but if it's missing, we're in teardown anyway
            None => return,
        };

        //try to fit the message into the current send buffer (the last one in line that already
        //contains some data)
        let mut enqueued = false;
        let filled_bufs = connector.bufs.iter_mut().filter(|b| b.filled_len() > 0);
        if let Some(send_buffer) = filled_bufs.last() {
            enqueued = send_buffer.fill(|buf| msg.encode(buf)).is_ok();
        }

        //if it doesn't work, try to fit the message into the send buffer directly following that
        //one (the first one that does not have any data in it)
        if !enqueued {
            let send_buffer = match connector.bufs.iter_mut().find(|b| b.filled_len() == 0) {
                Some(b) => b,
                None => {
                    connector.bufs.push(Default::default());
                    connector.bufs.last_mut().unwrap()
                }
            };
            //if the fill() errors out this time, it's because the rendered message is
            //legimitately too long, so it's a good time to panic
            send_buffer.fill(|buf| msg.encode(buf)).unwrap();
        }

        //wake up the transmitter job if necessary
        connector.notify.notify_one();
    }
}
