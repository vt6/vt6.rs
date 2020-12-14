/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

#![cfg_attr(all(not(test), not(feature = "use_std")), no_std)]

///Implementation parts for VT6 clients.
pub mod client;
///Common types and definitions that can be used both by VT6 servers and clients.
pub mod common;
///Decoded representations of common VT6 messages.
pub mod msg;
#[cfg(feature = "use_std")]
///Implementation parts for VT6 servers (terminals or shell wrappers proxying as a terminal).
pub mod server;
