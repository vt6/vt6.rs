/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::server;
use crate::server::tokio as my;
use futures::future::{AbortRegistration, Abortable};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::Notify;

pub(crate) struct SendBuffer {
    //Assuming a 64-bit platform, this makes sizeof(SendBuffer) = 4080. General-purpose allocators
    //usually need 8-16 bytes per allocation for bookkeeping, so overall Box<SendBuffer> allocates
    //just enough to fit snugly into a single 4 KiB memory page.
    buf: [u8; 4072],
    filled: usize,
}

impl Default for SendBuffer {
    fn default() -> Self {
        Self {
            buf: [0; 4072],
            filled: 0,
        }
    }
}

impl SendBuffer {
    ///Executes `action` on the unfilled portion and if successful, marks the parts that were
    ///written as filled. This is used for enqueuing messages: Messages are only enqueued
    ///completely or not at all, to increase the chance that they are transmitted in one piece.
    pub(crate) fn fill_if_ok<E, F>(&mut self, action: F) -> Result<(), E>
    where
        F: FnOnce(&mut [u8]) -> Result<usize, E>,
    {
        match action(&mut self.buf[self.filled..]) {
            Err(e) => Err(e),
            Ok(more_filled) => {
                self.filled = self.filled.saturating_add(more_filled);
                if self.filled >= self.buf.len() {
                    self.filled = self.buf.len();
                }
                Ok(())
            }
        }
    }

    ///Fills up the unfilled portion of this buffer as much as possible from `input`, and returns
    ///the part of `input` that did not fit. This is used for enqueuing stdin: It is possible that
    ///we get a ton of stdin at once (e.g. from a clipboard paste) that does not fit into one send
    ///buffer at all.
    pub(crate) fn fill_until_full<'a, 'b>(&'a mut self, input: &'b [u8]) -> &'b [u8] {
        //how much can we copy?
        let len = std::cmp::min(
            input.len(),                  //how much input bytes we havej
            self.buf.len() - self.filled, //how many bytes we have unfilled
        );
        if len > 0 {
            self.buf[self.filled..(self.filled + len)].copy_from_slice(&input[0..len]);
            self.filled += len; //no overflow check necessary here
        }
        &input[len..]
    }

    pub(crate) fn filled(&self) -> &[u8] {
        &self.buf[0..self.filled]
    }

    pub(crate) fn filled_len(&self) -> usize {
        self.filled
    }

    pub(crate) fn clear(&mut self) {
        self.filled = 0;
    }
}

pub(crate) fn spawn_transmitter<A: server::Application>(
    dispatch: Arc<my::InnerDispatch<A>>,
    abort_reg: AbortRegistration,
    conn_id: u64,
    mut writer: tokio::net::unix::OwnedWriteHalf,
    tx_notify: Arc<Notify>,
) {
    let mut buf = None;
    let job = async move {
        loop {
            //wait for data to become available
            tx_notify.notified().await;

            loop {
                //get the next send buffer
                buf = match dispatch.connection_mut(conn_id).alive() {
                    //the connection is being torn down
                    None => return,
                    //the connection is alive -> return the old send buffer and get a new one
                    Some(conn) => dispatch.swap_send_buffer(conn, buf),
                };
                match buf {
                    //no data waiting anymore -> go back to sleep
                    None => break,
                    //write the entire send buffer into the socket
                    Some(ref buf) => {
                        if let Err(e) = writer.write_all(buf.filled()).await {
                            let n = server::Notification::ConnectionIOError(e.into());
                            dispatch.app.notify(&n);
                            if let Some(conn) = dispatch.connection_mut(conn_id).alive() {
                                conn.set_state(server::ConnectionState::Teardown);
                            }
                            return;
                        }
                    }
                }
            }
        }
    };
    tokio::spawn(Abortable::new(job, abort_reg));
}
