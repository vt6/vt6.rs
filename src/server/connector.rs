/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::server;

///Connector for client sockets in msgio mode.
///
///The basic concept of connector types is explained in the
///[module-level documentation](index.html).
pub trait MessageConnector: Sized {
    fn new() -> Self;
}

///Connector for client sockets in stdin mode.
///
///The basic concept of connector types is explained in the
///[module-level documentation](index.html).
pub trait StdinConnector: Sized {
    fn new() -> Self;
}

///Connector for client sockets in stdout mode.
///
///The basic concept of connector types is explained in the
///[module-level documentation](index.html).
pub trait StdoutConnector: Sized {
    fn new() -> Self;
}

///Helper type used in implementations of [trait Dispatch](trait.Dispatch.html).
///
///An implementation of [trait Dispatch](trait.Dispatch.html) cannot take its respective Connector
///types as type arguments directly: Connector types are typically not `Clone`, but a type
///implementing Dispatch must be `Clone`. To circumvent this restriction, the application is
///expected to declare an empty struct type that implements this trait. The Dispatch type then
///takes that type as a type argument.
///
///We also bundle the handler types in there to avoid an excessive amount of type arguments on the
///types implementing [trait Dispatch](trait.Dispatch.html).
///
///```ignore
///#[derive(Clone)]
///struct MyApplication;
///
///impl Application for MyApplication {
///    type MessageConnector = MyMessageConnector;
///    type StdinConnector = MyStdinConnector;
///    type StdoutConnector = MyStdoutConnector;
///    type MessageHandler = MyMessageHandler;
///    type HandshakeHandler = MyHandshakeHandler;
///}
///
///let mut dispatch = MyDispatch<MyApplication>::new();
///```
pub trait Application: Clone + Send + Sync + 'static {
    type MessageConnector: MessageConnector + Send + Sync;
    type StdinConnector: StdinConnector + Send + Sync;
    type StdoutConnector: StdoutConnector + Send + Sync;
    type MessageHandler: server::MessageHandler<Self> + Send + Sync + Default;
    type HandshakeHandler: server::HandshakeHandler<Self> + Send + Sync + Default;
}
