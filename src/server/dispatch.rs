/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::common::core::msg;
use crate::server;

pub trait Dispatch<A: server::Application>: Clone + Sized {
    type ConnectionID: Clone + Send + Sync;

    fn notify(&self, n: &server::Notification);

    fn enqueue_broadcast(&self, action: Box<dyn Fn(&mut server::Connection<A, Self>)>);

    fn enqueue_message<F>(&self, conn: &mut server::Connection<A, Self>, action: F)
    where
        F: Fn(&mut [u8]) -> Result<usize, msg::BufferTooSmallError>;
}
