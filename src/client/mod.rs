/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

#[cfg(feature = "use_std")]
mod env;
#[cfg(feature = "use_std")]
pub use env::*;

///Client-side implementation of the [vt6/core module](https://vt6.io/std/core/).
pub mod core;
