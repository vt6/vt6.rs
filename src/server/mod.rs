/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

/*!
Since servers need to do a lot of bookkeeping that is not plausible in a no_std context, this
entire module requires the "use_std" feature.

Besides the infrastructure in this module, the following pieces need to be supplied to obtain a
complete VT6 server implementation:

1. A type implementing [trait Dispatch](trait.Dispatch.html). The implementation of this depends on
   which IO library you use. This crate includes an implementation for use with the Tokio library
   in the [vt6::server::tokio](tokio/index.html) submodule if the "use_tokio" feature is enabled on
   the crate. Similar implementations for other IO libraries may be added in the future, but you
   can always provide your own implementations if the ones supplied with this crate don't fit your
   use case.

2. A set of connector types, i.e. types implementing
   [trait MessageConnector](trait.MessageConnector.html),
   [trait StdinConnector](trait.StdinConnector.html) and
   [trait StdoutConnector](trait.StdoutConnector.html) respectively. One Connector instance is
   maintained for each connected client socket in the respective socket mode. The Connector allows
   the code in this crate to call into application-specific logic in response upon receiving
   messages or data from the client. The implementation of the Connector types is therefore highly
   application-dependent and typically not supplied by a library.

3. A type implementing [trait Handler](trait.Handler.html). This crate includes various modular
   handler types, each implementing support for one specific VT6 module, that can be chained
   together (similar to middlewares in a HTTP server) to create a complete handler type. Custom
   handler types can be mixed and matched with those in the library if non-standard VT6 modules
   need to be supported. Most handler types impose additional trait bounds on the Connector types
   to forward decoded messages etc. into application logic.

The "tokio_server" example in this crate provides a minimal working example of all those pieces
working together.
*/

mod connector;
pub use connector::*;
mod connection;
pub use connection::*;
mod dispatch;
pub use dispatch::*;
mod handler;
pub use handler::*;
mod notification;
pub use notification::*;

#[cfg(feature = "use_tokio")]
///An implementation of a server listener using the [Tokio library](https://tokio.rs/).
pub mod tokio;
