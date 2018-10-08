/******************************************************************************
*
*  Copyright 2018 Stefan Majewsky <majewsky@gmx.net>
*
*  Licensed under the Apache License, Version 2.0 (the "License");
*  you may not use this file except in compliance with the License.
*  You may obtain a copy of the License at
*
*      http://www.apache.org/licenses/LICENSE-2.0
*
*  Unless required by applicable law or agreed to in writing, software
*  distributed under the License is distributed on an "AS IS" BASIS,
*  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*  See the License for the specific language governing permissions and
*  limitations under the License.
*
******************************************************************************/

use std;
use std::path::PathBuf;

use futures::sync::mpsc;
use tokio::prelude::*;
use tokio_uds::UnixListener;
use vt6::server as vt6s;

use server::core::bidi_byte_stream::BidiByteStream;
use server::core::{Connection, IncomingEvent};

///A task future that creates a VT6 server socket, accepts and handles incoming
///connections from clients.
///
///There are two type arguments on this type:
///
///* `C` is a type in your application that implements the various `Connection`
///  traits in vt6 and vt6tokio. The integration points between `C` and this
///  type are provided by this module's
///  [`trait Connection`](trait.Connection.html).
///
///* `H` is the (chain of) handlers for incoming VT6 messages. See documentation
///  on `trait vt6::server::Handler` for details.
///
///The following extra ingredients go into making a server future:
///
///* an `mpsc::Receiver<IncomingEvent>` that other threads (or other futures)
///  can use to wake up the server future when work needs to be done; see
///  documentation on [`enum IncomingEvent`](enum.IncomingEvent.html) for
///  details
///
///* an `mpsc::Sender<C::OutgoingEvent>` and a `C::ModelRef`, which get passed
///  to the instances of `C` (the individual connections) to communicate to the
///  outside world; see documentation on
///  [`trait Connection`](trait.Connection.html) for details
pub struct Server<C: Connection, H: vt6s::EarlyHandler<C> + Send + Sync> {
    handler: H,
    socket_path: PathBuf,
    socket: UnixListener,
    streams: Vec<BidiByteStream<C>>,
    next_connection_id: u32,
    event_rx: mpsc::Receiver<IncomingEvent>,
    event_tx: mpsc::Sender<C::OutgoingEvent>,
    model_ref: C::ModelRef,
}

impl<C: Connection, H: vt6s::EarlyHandler<C> + Send + Sync> Server<C, H> {
    ///Creates a new socket at `socket_path` (or returns `Err` if that fails)
    ///and prepares a server future to listen on it. See documentation on type
    ///for details.
    pub fn new(
        handler: H,
        socket_path: PathBuf,
        event_rx: mpsc::Receiver<IncomingEvent>,
        event_tx: mpsc::Sender<C::OutgoingEvent>,
        model_ref: C::ModelRef,
    ) -> std::io::Result<Self> {
        //FIXME This opens the socket with SOCK_STREAM, but vt6/posix1 mandates
        //SOCK_SEQPACKET. I'm doing the prototyping with this for now because
        //neither mio-uds nor tokio-uds support SOCK_SEQPACKET.
        let listener = UnixListener::bind(&socket_path)?;

        Ok(Server {
            handler: handler,
            socket_path: socket_path,
            socket: listener,
            streams: Vec::new(),
            next_connection_id: 0,
            event_rx: event_rx,
            event_tx: event_tx,
            model_ref: model_ref,
        })
    }
}

impl<C: Connection, H: vt6s::EarlyHandler<C> + Send + Sync> Drop for Server<C, H> {
    fn drop(&mut self) {
        if let Err(err) = std::fs::remove_file(&self.socket_path) {
            error!("socket cleanup failed: {}", err);
        }
    }
}

impl<C: Connection, H: vt6s::EarlyHandler<C> + Send + Sync> Future for Server<C, H> {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<(), ()> {
        //check for new client connections
        match self.socket.poll_accept() {
            Err(e) => {
                error!("error accepting new client connection: {}", e);
                return Err(()); //this error is fatal
            },
            Ok(Async::Ready((stream, _))) => {
                let id = self.next_connection_id;
                self.next_connection_id += 1;

                let conn = C::new(id, self.model_ref.clone(), self.event_tx.clone());
                let bidi = BidiByteStream::new(conn, stream);
                self.streams.push(bidi);
            },
            _ => {},
        };

        //recurse into client connections to handle input received on them
        let mut closed_stream_ids = std::collections::hash_set::HashSet::new();
        for c in self.streams.iter_mut() {
            match c.poll(&self.handler) {
                Err(e) => {
                    error!("error on connection {}: {}", c.conn.id(), e);
                    //fatal error for this connection - close it from our side
                    closed_stream_ids.insert(c.conn.id());
                },
                Ok(Async::Ready(())) => {
                    //client disconnected
                    closed_stream_ids.insert(c.conn.id());
                },
                Ok(Async::NotReady) => {},
            }
        }
        self.streams.retain(|ref c| !closed_stream_ids.contains(&c.conn.id()) );

        //see if there's any events we need to react to
        match self.event_rx.poll() {
            Err(e) => {
                error!("error receiving events from frontend: {:?}", e);
                Err(()) //this error is fatal
            },
            Ok(Async::NotReady) => Ok(Async::NotReady),
            //closed channel signals shutdown request from GUI thread
            Ok(Async::Ready(None)) => Ok(Async::Ready(())),
            Ok(Async::Ready(Some(event))) => {
                match event {
                    IncomingEvent::UserInput(text) => {
                        let mut search_result = self.streams.iter_mut()
                            .filter(|s| s.conn.stream_state().mode == vt6s::core::StreamMode::Stdio)
                            .max_by_key(|s| s.conn.stream_state().entered);
                        if let Some(stream) = search_result {
                            stream.append_to_send_buffer(text.as_bytes());
                        }
                    },
                }
                //restart function call to send outstanding messages or data implied by this event
                self.poll()
            },
        }
    }
}
