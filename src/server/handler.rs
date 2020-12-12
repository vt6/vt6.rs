/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::common::core::msg;
use crate::server;

pub trait Handler<A: server::Application> {
    fn handle<D: server::Dispatch<A>>(
        &self,
        msg: &msg::Message,
        conn: &mut server::Connection<A, D>,
    );
    fn handle_error<D: server::Dispatch<A>>(
        &self,
        err: &msg::ParseError,
        conn: &mut server::Connection<A, D>,
    );
}

pub trait MessageHandler<A: server::Application>: Handler<A> {}

pub trait HandshakeHandler<A: server::Application>: Handler<A> {}
