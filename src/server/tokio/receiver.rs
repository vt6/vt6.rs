/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::server;
use crate::server::tokio as my;
use crate::server::Dispatch;
use futures::future::{AbortRegistration, Abortable};
use std::sync::Arc;
use tokio::io::AsyncReadExt;

impl server::ReceiveBuffer for bytes::BytesMut {
    fn contents(&self) -> &[u8] {
        &self[..]
    }
    fn discard(&mut self, consumed: usize) {
        //TODO: use memmove for efficiency
        if consumed > 0 {
            for idx in consumed..self.len() {
                self[idx - consumed] = self[idx];
            }
            self.truncate(self.len() - consumed);
        }
    }
}

pub(crate) fn spawn_receiver<A: server::Application>(
    dispatch: Arc<my::InnerDispatch<A>>,
    abort_reg: AbortRegistration,
    conn_id: u64,
    mut reader: tokio::net::unix::OwnedReadHalf,
) {
    let job = async move {
        let mut buf = bytes::BytesMut::with_capacity(1024);
        loop {
            //attempt to fill the buffer
            if let Err(e) = reader.read_buf(&mut buf).await {
                let n = server::Notification::ConnectionIOError(e.into());
                dispatch.dispatch().notify(&n);
                if let Some(conn) = dispatch.connection_mut(conn_id).alive() {
                    conn.set_state(server::ConnectionState::Teardown);
                }
                return;
            }

            if buf.is_empty() {
                //EOF is reached, i.e. the client has disconnected
                if let Some(conn) = dispatch.connection_mut(conn_id).alive() {
                    conn.set_state(server::ConnectionState::Teardown);
                }
                return;
            }

            if let Some(conn) = dispatch.connection_mut(conn_id).alive() {
                conn.handle_incoming(&mut buf);
            }
        }
    };
    tokio::spawn(Abortable::new(job, abort_reg));
}
