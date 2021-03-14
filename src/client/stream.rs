/*******************************************************************************
* Copyright 2021 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::client::{AsyncRuntime, DelayedResponseHandler};
use futures::io::BufReader;

///Manages the read side of a connection. This type is not exposed to the public directly. It is
///only used by more high-level types within this library, particularly `struct Connection` and
///`struct Poller`.
pub(crate) struct StreamReader<R: AsyncRuntime, D: DelayedResponseHandler> {
    runtime: R,
    reader: BufReader<R::StreamReader>,
    handler: D,
}

impl<R: AsyncRuntime, D: DelayedResponseHandler> StreamReader<R, D> {
    pub fn new(runtime: R, reader: R::StreamReader, handler: D) -> Self {
        let reader = BufReader::with_capacity(1024, reader);
        Self {
            runtime,
            reader,
            handler,
        }
    }
}
