/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

#![cfg_attr(all(not(test), not(feature = "use_std")), no_std)]

/*!

[VT6](https://vt6.io/) is a modern protocol for virtual terminals, which, among
other things, runs almost entirely in userspace and provides a modular basis on
which to serve both legacy clients and forward-looking terminals that want to
provide modern capabilities to applications.

This crate contains the basic implementation of the VT6 protocol, from the
encoding and decoding of VT6 messages on the lowest level, to providing common
data structures for VT6 terminals and clients on the higher levels.

## std or no_std?

Even though VT6 shifts most of the implementation burden from clients
(applications running in the terminal) to the servers (the terminal itself or
any shell wrapper proxying as one), implementing the VT6 protocol can arguably
be more involved than dealing with the ANSI escape sequences and terminal
attributes from the days of yore.

Regardless, it is an explicit design goal of VT6 to be useful for clients
running in embedded systems and other resource-constrained situations. To that
end, this crate can be used in a no_std environment (without the `std` and
`alloc` crates) by disabling the `use_std` feature which is enabled by default.

When actually going down that road, however, you will find the crate's API to be
unpleasantly sparse, because most useful things in this crate depend on
`std::io`, `std::path`, or even async runtimes like Tokio, all of which are in
or depend on std. Until these things are available in no_std, there is not much
to be done about that. If you are interested in that particular rabbit hole,
here are some links for you to jump off from:

* for `std::io`: <https://github.com/rust-lang/rfcs/issues/2262>
* for Tokio: <https://github.com/tokio-rs/mio/issues/21>

*/

///Implementation parts for VT6 clients.
pub mod client;
///Common types and definitions that can be used both by VT6 servers and clients.
pub mod common;
///Decoded representations of common VT6 messages.
pub mod msg;
#[cfg(feature = "use_std")]
///Implementation parts for VT6 servers (terminals or shell wrappers proxying as a terminal).
pub mod server;
