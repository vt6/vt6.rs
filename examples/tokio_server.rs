/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use vt6::common::core::msg;
use vt6::server::{Application, Connection, Dispatch, Handler, HandshakeHandler, MessageHandler, Notification};

fn main() -> std::io::Result<()> {
    belog::init();
    //TODO We should use the default runtime and #[tokio::main], but that would bloat the feature
    //selection on the main lib. Move this example into a separate crate to remove this
    //restriction and make the example more idiomatic.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    let dispatch = vt6::server::tokio::Dispatch::<Dummy>::new("/tmp/vt6-tokio-server", Dummy)?;
    rt.block_on(async move { dispatch.run_listener().await })
}

#[derive(Clone, Default)]
struct Dummy;

impl Application for Dummy {
    type MessageConnector = Dummy;
    type StdoutConnector = Dummy;
    type MessageHandler = LoggingHandler<vt6::server::reject::MessageHandler>;
    type HandshakeHandler = LoggingHandler<vt6::server::reject::HandshakeHandler>;

    fn notify(&self, n: &Notification) {
        if n.is_error() {
            log::error!("{}", n);
        } else {
            log::info!("{}", n);
        }
    }
}

impl vt6::server::MessageConnector for Dummy {}

impl vt6::server::StdoutConnector for Dummy {}

///This handler is a minimal useful example of how handlers can be combined through chaining,
///similar to the middlewares that exist in most HTTP server frameworks.
#[derive(Default)]
struct LoggingHandler<H> {
    next: H,
}

impl<A: Application, H: Handler<A>> Handler<A> for LoggingHandler<H> {
    fn handle<D: Dispatch<A>>(&self, msg: &msg::Message, conn: &mut Connection<A, D>) {
        log::info!(
            "received message {} in connection state {}",
            msg,
            conn.state().type_name()
        );
        self.next.handle(msg, conn)
    }

    fn handle_error<D: Dispatch<A>>(&self, e: &msg::ParseError, conn: &mut Connection<A, D>) {
        log::error!("parse error: {} at offset {}", e.kind, e.offset);
        self.next.handle_error(e, conn)
    }
}

impl<A: Application, H: MessageHandler<A>> MessageHandler<A> for LoggingHandler<H> {}

impl<A: Application, H: HandshakeHandler<A>> HandshakeHandler<A> for LoggingHandler<H> {}
