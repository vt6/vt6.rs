/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

#[cfg(feature = "use_std")]
mod connection;
#[cfg(feature = "use_std")]
pub use connection::*;
#[cfg(feature = "use_std")]
mod env;
#[cfg(feature = "use_std")]
pub use env::*;
#[cfg(feature = "use_std")]
mod stream;
#[cfg(feature = "use_std")]
pub use stream::*;
mod traits;
pub use traits::*;

///Client-side implementation of the [vt6/core module](https://vt6.io/std/core/).
pub mod core;
