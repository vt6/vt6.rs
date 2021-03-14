/*******************************************************************************
* Copyright 2021 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::client::{AsyncRuntime, DelayedResponseHandler, Request, StreamReader};
use crate::common::core::msg::Message;
use futures::future::{AbortHandle, AbortRegistration};
use futures::channel::oneshot::{channel, Sender, Receiver};
use std::sync::Mutex;

///TODO doc
pub struct Connection<R: AsyncRuntime, D: DelayedResponseHandler> {
    inner: Mutex<ConnectionInner<R, D>>,
}

struct ConnectionInner<R: AsyncRuntime, D: DelayedResponseHandler> {
    runtime: R,
    //We cannot hold the StreamReaderState or StreamWriter inside the Connection object. Either
    //there are queries running, in which case the task corresponding to the running query needs
    //them. Or there are no queries in flight, in which case we need to give the StreamReaderState
    //to a Poller task that listens for delayed responses.
    reader_rx: Receiver<StreamReader<R, D>>,
    writer_rx: Receiver<R::StreamWriter>,
    //When the StreamReaderState is given out to a Poller task that just listens for delayed
    //responses, we need to be able to get the StreamReaderState back immediately when we want to
    //start a query. This handle allows us to instruct the Poller to do that.
    poll_abort_handle: Option<AbortHandle>,
}

impl<R: AsyncRuntime, D: DelayedResponseHandler> Connection<R, D> {
    pub fn new(runtime: R, reader: R::StreamReader, writer: R::StreamWriter, handler: D) -> Self {
        let reader_state = StreamReader::new(runtime.clone(), reader, handler);

        let (reader_tx, reader_rx) = channel();
        let (writer_tx, writer_rx) = channel();
        //store `writer` inside `writer_rx` until we need it
        let _ = writer_tx.send(writer);
        //^ We can safely discard the Result here since Err only occurs when the receiver end
        //was dropped, which is clearly impossible here.

        let (poll_abort_handle, poll_abort_reg) = AbortHandle::new_pair();
        let poller = Poller {
            reader_state,
            reader_tx,
            poll_abort_reg,
        };
        runtime.spawn_poller(poller);

        let inner = ConnectionInner {
            runtime,
            reader_rx,
            writer_rx,
            poll_abort_handle: Some(poll_abort_handle),
        };
        Self { inner: Mutex::new(inner) }
    }

    pub async fn query<Req: Request, F: FnOnce(&Message)>(&self, req: Req, action: F) {
        TODO("We want to be able to have multiple queries in flight. What kind of Arc/Mutex/etc. do we need in ConnectionInner to achieve that?")
    }
}

///Error type returned from [`Connection::spawn`](struct.Connection.html).
#[derive(Debug)]
pub enum HandshakeError {
    IOError(futures::io::Error),
}

///TODO doc (This is the type that listens for delayed responses while no query is running.)
///TODO better name?
pub struct Poller<R: AsyncRuntime, D: DelayedResponseHandler> {
    reader_state: StreamReader<R, D>,
    reader_tx: Sender<StreamReader<R, D>>,
    poll_abort_reg: AbortRegistration,
}

impl<R: AsyncRuntime, D: DelayedResponseHandler> Poller<R, D> {
    pub async fn run() {
        //TODO
    }
}
