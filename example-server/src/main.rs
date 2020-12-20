/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use std::sync::{Arc, Mutex};
use vt6::common::core::{msg, ClientID};
use vt6::server::{
    Application, ClientCredentials, ClientIdentity, Connection, Dispatch, Handler,
    HandshakeHandler, MessageHandler, Notification, ScreenCredentials, ScreenIdentity,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    belog::init();

    let app = MyApplication {
        pending_clients: Vec::new(),
        screen_identity: ScreenIdentity::new("screen1"),
        screen_credentials: ScreenCredentials::generate(),
        stdin_authorized: false,
        stdout_authorized: false,
    };
    let app = MyApplicationRef(Arc::new(Mutex::new(app)));

    let socket_path = vt6::server::default_socket_path()?;
    let dispatch = vt6::server::tokio::Dispatch::new(socket_path, app)?;
    dispatch.run_listener().await
}

////////////////////////////////////////////////////////////////////////////////
// Application object

struct MyApplication {
    pending_clients: Vec<(ClientIdentity, ClientCredentials, bool)>,
    //This example server has exactly one screen, allocated statically on startup.
    screen_identity: ScreenIdentity,
    screen_credentials: ScreenCredentials,
    stdin_authorized: bool,
    stdout_authorized: bool,
}

#[derive(Clone)]
struct MyApplicationRef(Arc<Mutex<MyApplication>>);

impl vt6::server::Application for MyApplicationRef {
    type MessageConnector = MyMessageConnector;
    type StdoutConnector = MyStdoutConnector;
    type MessageHandler =
        LoggingHandler<vt6::server::core::MessageHandler<vt6::server::reject::MessageHandler>>;
    type HandshakeHandler =
        LoggingHandler<vt6::server::core::HandshakeHandler<vt6::server::reject::HandshakeHandler>>;

    fn notify(&self, n: &Notification) {
        if n.is_error() {
            log::error!("{}", n);
        } else {
            log::info!("{}", n);
        }
    }

    fn register_client(&self, i: ClientIdentity) -> ClientCredentials {
        let creds = ClientCredentials::generate();
        let mut app = self.0.lock().unwrap();
        app.pending_clients.push((i, creds.clone(), false));
        creds
    }

    fn authorize_client(&self, secret: &str) -> Option<ClientIdentity> {
        let mut app = self.0.lock().unwrap();
        let (id, _, ref mut is_authorized) = app
            .pending_clients
            .iter_mut()
            .find(|(_, creds, _)| creds.secret() == secret)?;
        if *is_authorized {
            None
        } else {
            *is_authorized = true;
            Some(id.clone())
        }
    }

    fn find_client(&self, id: ClientID<'_>) -> Option<ClientIdentity> {
        let app = self.0.lock().unwrap();
        app.pending_clients
            .iter()
            .find(|(i, _, _)| i.client_id() == id)
            .map(|(i, _, _)| i.clone())
    }

    fn authorize_stdin(&self, secret: &str) -> Option<ScreenIdentity> {
        let mut app = self.0.lock().unwrap();
        if !app.stdin_authorized && app.screen_credentials.stdin_secret() == secret {
            app.stdin_authorized = true;
            Some(app.screen_identity.clone())
        } else {
            None
        }
    }

    fn authorize_stdout(&self, secret: &str) -> Option<ScreenIdentity> {
        let mut app = self.0.lock().unwrap();
        if !app.stdout_authorized && app.screen_credentials.stdout_secret() == secret {
            app.stdout_authorized = true;
            Some(app.screen_identity.clone())
        } else {
            None
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Connector objects

#[derive(Clone)]
struct MyMessageConnector {
    id: vt6::server::ClientIdentity,
}

impl vt6::server::MessageConnector for MyMessageConnector {
    fn new(id: vt6::server::ClientIdentity) -> Self {
        Self { id }
    }
}

#[derive(Clone)]
struct MyStdoutConnector {
    id: vt6::server::ScreenIdentity,
}

impl vt6::server::StdoutConnector for MyStdoutConnector {
    fn new(id: vt6::server::ScreenIdentity) -> Self {
        Self { id }
    }
}

////////////////////////////////////////////////////////////////////////////////
// custom handlers

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
