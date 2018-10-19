# vt6.rs

Reusable parts for [VT6](https://vt6.io/) clients and servers. This repository contains two crates:

* `vt6` provides generic client and server code for handling and generating VT6 messages. Most facilities in this crate
  can be used in a `#![no_std]` application if the `use_std` feature is disabled.
* `vt6tokio` provides an implementation of a VT6 server main loop using [Tokio 0.1](https://tokio.rs/).
