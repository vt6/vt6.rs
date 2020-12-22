/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::server;

///Connector for client sockets in msgio mode.
///
///One MessageConnector instance is maintained for each client socket in msg mode. The connector
///allows for library code in this crate to call into application-specific logic when handling
///messages sent by the client. The implementation is therefore highly application-dependent and
///typically not supplied by a library.
pub trait MessageConnector: Sized + Send + Sync {
    fn new(id: server::ClientIdentity) -> Self;

    fn identity(&self) -> &server::ClientIdentity;
}

///Connector for client sockets in stdout mode.
///
///One StdoutConnector instance is maintained for each client socket in stdout mode. The connector
///allows for library code in this crate to call into application-specific logic when handling
///messages sent by the client. The implementation is therefore highly application-dependent and
///typically not supplied by a library.
pub trait StdoutConnector: Sized + Send + Sync {
    fn new(id: server::ScreenIdentity) -> Self;

    ///Called by the Connection whenever stdout has been received from the client.
    fn receive(&mut self, buf: &[u8]);
}

///Main integration point for application-specific logic.
///
///Every application using any part of `vt6::server` needs to supply a type implementing this trait.
///Code in this crate uses that type to call into application-specific code in response to input or
///requests from clients. The implementor must implement `Clone`, `Send` and `Sync` since it is
///passed around inside the [`Dispatch`](trait.Dispatch.html) and all of its associated jobs and
///worker threads. Therefore, in most cases, an instance of `Application` is an `Arc<Mutex<...>>`
///or similar containing references to the application core.
///
///Besides an `Application` type, the application also has to choose and/or implement handler and
///connector types. These types are bundled into the `Application` type as associated types on this
///trait. Therefore most library types only need one or two type arguments: the `Application` and
///possibly the [`Dispatch`](trait.Dispatch.html).
///
///```ignore
///#[derive(Clone)]
///struct MyApplication;
///
///impl Application for MyApplication {
///    type MessageConnector = MyMessageConnector;
///    type StdoutConnector = MyStdoutConnector;
///    type MessageHandler = MyMessageHandler;
///    type HandshakeHandler = MyHandshakeHandler;
///
///    //... trait methods ...
///}
///
///let mut dispatch = MyDispatch::new(MyApplication::new());
///```
pub trait Application: Clone + Send + Sync + 'static {
    type MessageConnector: MessageConnector;
    type StdoutConnector: StdoutConnector;
    type MessageHandler: server::MessageHandler<Self>;
    type HandshakeHandler: server::HandshakeHandler<Self>;

    ///Hook for the application to receive miscellaneous informational messages or non-fatal
    ///errors.
    fn notify(&self, n: &server::Notification);

    ///Register a new client with the terminal. This does not return an `Option<>` since the
    ///terminal is not allowed to refuse new clients. The handler generating this call will have
    ///made sure that the prospective client is below the requesting client, i.e. that the
    ///requesting client's ID is a prefix of `i.client_id()`, and that `i.client_id()` is not yet
    ///in use.
    fn register_client(&self, i: server::ClientIdentity) -> server::ClientCredentials;
    ///Authorize a client's attempt to handshake for an msgio socket. Since each client ID is only
    ///supposed to map to exactly one msgio socket, implementations SHALL NOT authorize the same
    ///secret multiple times.
    fn authorize_client(&self, secret: &str) -> Option<server::ClientIdentity>;
    ///Returns information about the client with the given ID if it has been registered with the
    ///terminal.
    fn find_client(&self, id: crate::common::core::ClientID<'_>) -> Option<server::ClientIdentity>;

    ///Authorize a client's attempt to handshake for an stdin socket. To ensure that each screen
    ///has at most one stdin socket connected to it, implementations SHALL NOT authorize the same
    ///secret multiple times.
    fn authorize_stdin(&self, secret: &str) -> Option<server::ScreenIdentity>;
    ///Authorize a client's attempt to handshake for an stdout socket. To ensure that each screen
    ///has at most one stdout socket connected to it, implementations SHALL NOT authorize the same
    ///secret multiple times.
    fn authorize_stdout(&self, secret: &str) -> Option<server::ScreenIdentity>;
}
