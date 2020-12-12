/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use vt6::common::core::msg;

fn main() -> std::io::Result<()> {
    belog::init();
    //TODO We should use the default runtime and #[tokio::main], but that would bloat the feature
    //selection on the main lib. Move this example into a separate crate to remove this
    //restriction and make the example more idiomatic.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    let dispatch = vt6::server::tokio::Dispatch::<Dummy>::new("/tmp/vt6-tokio-server")?;
    rt.block_on(async move { dispatch.run_listener().await })
}

#[derive(Clone, Default)]
struct Dummy;

impl vt6::server::Application for Dummy {
    type MessageConnector = Dummy;
    type StdinConnector = Dummy;
    type StdoutConnector = Dummy;
    type MessageHandler = Dummy;
    type HandshakeHandler = Dummy;
}

impl vt6::server::MessageConnector for Dummy {
    fn new() -> Self {
        Self
    }
}

impl vt6::server::StdinConnector for Dummy {
    fn new() -> Self {
        Self
    }
}

impl vt6::server::StdoutConnector for Dummy {
    fn new() -> Self {
        Self
    }
}

impl<A: vt6::server::Application> vt6::server::Handler<A> for Dummy {
    fn handle<D: vt6::server::Dispatch<A>>(
        &self,
        msg: &msg::Message,
        conn: &mut vt6::server::Connection<A, D>,
    ) {
        log::info!("received message: {}", msg);
        let d = conn.dispatch().clone();
        d.enqueue_message(conn, |buf| {
            let mut f = msg::MessageFormatter::new(buf, "beep", 1);
            f.add_argument(&42usize);
            f.finalize()
        });
    }

    fn handle_error<D: vt6::server::Dispatch<A>>(
        &self,
        e: &msg::ParseError,
        _conn: &mut vt6::server::Connection<A, D>,
    ) {
        log::error!("parse error: {} at offset {}", e.kind, e.offset)
    }
}

impl vt6::server::MessageHandler<Dummy> for Dummy {}

impl vt6::server::HandshakeHandler<Dummy> for Dummy {}
