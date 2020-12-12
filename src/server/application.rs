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
pub trait MessageConnector: Sized + Send + Sync {}

///Connector for client sockets in stdout mode.
///
///One StdoutConnector instance is maintained for each client socket in stdout mode. The connector
///allows for library code in this crate to call into application-specific logic when handling
///messages sent by the client. The implementation is therefore highly application-dependent and
///typically not supplied by a library.
pub trait StdoutConnector: Sized + Send + Sync {}

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

    fn notify(&self, n: &server::Notification);
}
