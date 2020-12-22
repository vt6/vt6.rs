/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use std::sync::{Arc, Mutex};
use vt6::common::core::{msg, ClientID};
use vt6::server::{
    Application, ClientCredentials, ClientIdentity, ClientSelector, Connection, Dispatch, Handler,
    HandshakeHandler, MessageHandler, Notification, ScreenCredentials, ScreenIdentity,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    belog::init();

    //log the handshake messages that users can use to connect
    let client_identity = ClientIdentity::new(&ClientID::parse("a").unwrap());
    let client_credentials = ClientCredentials::generate();
    let screen_identity = ScreenIdentity::new("screen1");
    let screen_credentials = ScreenCredentials::generate();
    let msg1 = vt6::msg::posix::StdinHello {
        secret: screen_credentials.stdin_secret(),
    };
    log::info!("{}", encode_to_string(msg1));
    let msg2 = vt6::msg::posix::StdoutHello {
        secret: screen_credentials.stdout_secret(),
    };
    log::info!("{}", encode_to_string(msg2));
    let msg3 = vt6::msg::ClientHello {
        secret: client_credentials.secret(),
    };
    log::info!("{}", encode_to_string(msg3));

    //create an Application instance
    let app = MyApplication {
        clients: vec![(client_identity, client_credentials, false)],
        screen_identity: screen_identity.clone(),
        screen_credentials,
        stdin_authorized: false,
        stdout_authorized: false,
    };
    let app = MyApplicationRef(Arc::new(Mutex::new(app)));

    //create a Dispatch, we will run its event loop down below
    let socket_path = vt6::server::default_socket_path()?;
    let dispatch = vt6::server::tokio::Dispatch::new(socket_path, app.clone())?;

    //shutdown server on Ctrl-C
    {
        let dispatch = dispatch.clone();
        tokio::spawn(async move {
            use tokio::signal::unix::{signal, SignalKind};
            let mut stream = signal(SignalKind::interrupt())?;
            stream.recv().await;
            log::info!("interrupt received: shutting down...");
            dispatch.shutdown();
            Ok(()) as std::io::Result<()>
        });
    }

    //demonstrate sending stdin to client
    //
    //Since this example server does not take user input, we just send a static string every
    //second. This will be a no-op until a client connects to the stdin since the loop will not
    //find a matching Connection object in the right state.
    {
        let dispatch = dispatch.clone();
        tokio::spawn(async move {
            let one_second = std::time::Duration::new(1, 0);
            loop {
                tokio::time::sleep(one_second).await;
                let screen_identity = screen_identity.clone();
                dispatch.enqueue_broadcast(Box::new(move |conn| {
                    if conn.state().can_receive_stdin_for_screen(&screen_identity) {
                        conn.enqueue_stdin(b"Hello stdin.\n");
                    }
                }));
            }
        });
    }

    //run the dispatch event loop and exit the program once it's done (either via error or via
    //Ctrl-C as set up above)
    dispatch.run_listener().await
}

fn encode_to_string<M: vt6::common::core::msg::EncodeMessage>(msg: M) -> String {
    let mut buf = [0u8; 1024];
    let len = msg.encode(&mut buf).unwrap();
    String::from_utf8_lossy(&buf[0..len]).into()
}

////////////////////////////////////////////////////////////////////////////////
// Application object

struct MyApplication {
    clients: Vec<(ClientIdentity, ClientCredentials, bool)>,
    //This example server has exactly one screen, allocated statically on startup.
    screen_identity: ScreenIdentity,
    screen_credentials: ScreenCredentials,
    stdin_authorized: bool,
    stdout_authorized: bool,
}

#[derive(Clone)]
struct MyApplicationRef(Arc<Mutex<MyApplication>>);

impl std::fmt::Debug for MyApplicationRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MyApplicationRef(<opaque>)")
    }
}

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
        app.clients.push((i, creds.clone(), false));
        creds
    }

    fn unregister_clients(&self, s: ClientSelector) {
        let mut app = self.0.lock().unwrap();
        app.clients = app
            .clients
            .drain(..)
            .filter(|(ident, _, _)| !s.contains(ident.client_id()))
            .collect();
    }

    fn has_clients(&self, s: ClientSelector) -> bool {
        let app = self.0.lock().unwrap();
        app.clients
            .iter()
            .any(|(ident, _, _)| s.contains(ident.client_id()))
    }

    fn authorize_client(&self, secret: &str) -> Option<ClientIdentity> {
        let mut app = self.0.lock().unwrap();
        let (id, _, ref mut is_authorized) = app
            .clients
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
        app.clients
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

#[derive(Clone, Debug)]
struct MyMessageConnector {
    id: vt6::server::ClientIdentity,
}

impl vt6::server::MessageConnector for MyMessageConnector {
    fn new(id: vt6::server::ClientIdentity) -> Self {
        Self { id }
    }

    fn identity(&self) -> &vt6::server::ClientIdentity {
        &self.id
    }
}

#[derive(Clone, Debug)]
struct MyStdoutConnector {
    id: vt6::server::ScreenIdentity,
}

impl vt6::server::StdoutConnector for MyStdoutConnector {
    fn new(id: vt6::server::ScreenIdentity) -> Self {
        Self { id }
    }

    fn receive(&mut self, data: &[u8]) {
        log::info!(
            "stdout received for screen {}: {:?}",
            self.id.screen_id(),
            String::from_utf8_lossy(data)
        );
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
