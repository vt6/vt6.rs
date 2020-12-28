/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

/*!
Since servers need to do a lot of bookkeeping that is not feasible in a no_std context, this entire
module requires the "use_std" feature.

This module (`vt6::server`) contains some basic types and most importantly a bunch of traits for
the various bits and pieces of a VT6 server. Most of the submodules (e.g. `vt6::server::core`)
implement generic support for a specific VT6 module.

To get started, you will need to build a type that implements
[trait Application](trait.Application.html). This trait contains several methods that you need to
implement, and has a bunch of associated types that you also need to provide. If you follow the
trail of breadcrumbs from `trait Application`, you'll see everything that you need to choose or
implement.

Once you have a type implementing [trait Application](trait.Application.html), you just need to
choose an implementation of [trait Dispatch](trait.Dispatch.html) to go with it. The bird's eye
perspective is that `trait Application` integrates with your application, whereas `trait Dispatch`
integrates with the IO library or framework that you use. This crate provides implementations of
`trait Dispatch` for some common IO libraries; see documentation on
[trait Dispatch](trait.Dispatch.html) for details.

The "example-server" crate in this repo provides a minimal working example of all those pieces
working together.
*/

mod application;
pub use application::*;
mod auth;
pub use auth::*;
mod connection;
pub use connection::*;
mod dispatch;
pub use dispatch::*;
mod handler;
pub use handler::*;
mod notification;
pub use notification::*;
mod reject;
pub use reject::*;
mod util;
pub use util::*;

///Handlers and types for the [vt6::core](https://vt6.io/std/core/) module. Also implements some
///behavior defined in [vt6::foundation](https://vt6.io/std/foundation/).
pub mod core;

#[cfg(feature = "use_tokio")]
///An implementation of a server listener using the [Tokio library](https://tokio.rs/).
pub mod tokio;
