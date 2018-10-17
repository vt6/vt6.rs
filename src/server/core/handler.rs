/******************************************************************************
*
*  Copyright 2018 Stefan Majewsky <majewsky@gmx.net>
*
*  Licensed under the Apache License, Version 2.0 (the "License");
*  you may not use this file except in compliance with the License.
*  You may obtain a copy of the License at
*
*      http://www.apache.org/licenses/LICENSE-2.0
*
*  Unless required by applicable law or agreed to in writing, software
*  distributed under the License is distributed on an "AS IS" BASIS,
*  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*  See the License for the specific language governing permissions and
*  limitations under the License.
*
******************************************************************************/

use std::str;

use common::core::*;
use server::{self, Connection};

///A [handler](../trait.Handler.html) that implements the [vt6/core
///module](https://vt6.io/std/core/).
///
///The type argument `H` is the next handler which is wrapped by this handler:
///
///```rust,ignore
///let handler = vt6::server::core::Handler::new(next_handler);
///```
///
///See documentation on the [Handler trait](../trait.Handler.html) for
///how handlers are chained together.
///
///This handler is notable because every handler preceding it implements the
///[EarlyHandler trait](../trait.EarlyHandler.html), but every handler
///succeeding it implements the [Handler trait](../trait.Handler.html).
pub struct Handler<H> {
    next: H,
}

impl<H> Handler<H> {
    ///Constructor. The argument is the next handler after this handler.
    pub fn new(next: H) -> Self {
        Handler { next: next }
    }

    fn handle_want<C: Connection>(&self, msg: &msg::Message, conn: &mut C, send_buffer: &mut [u8]) -> Option<usize>
        where H: server::Handler<C>
    {
        //validate arguments: first argument is module name
        let mut args_iter = msg.arguments();
        let module_name = str::from_utf8(args_iter.next()?).ok()?;
        if !is_identifier(module_name) {
            return None;
        }

        //validate arguments: remaining arguments are major versions, need at least one
        if args_iter.len() == 0 {
            return None;
        }
        for arg in args_iter.clone() {
            let major_version = str::from_utf8(arg).ok()?.parse::<u16>().ok()?;
            if major_version == 0 {
                return None;
            }
        }
        let major_versions_iter = args_iter.map(|arg| str::from_utf8(arg).unwrap().parse::<u16>().unwrap());
        let check_want_result = self.check_want(module_name, major_versions_iter, conn);

        match check_want_result {
            Some((version, store)) => {
                if store {
                    conn.enable_module(module_name, version);
                }
                let mut f = msg::MessageFormatter::new(send_buffer, "have", 2);
                f.add_argument(module_name);
                f.add_argument(&version);
                f.finalize().ok()
            },
            None => {
                msg::MessageFormatter::new(send_buffer, "have", 0).finalize().ok()
            },
        }
    }

    fn check_want<C: Connection, I: Iterator<Item=u16> + Clone>(&self, module_name: &str, major_versions_iter: I, conn: &C) -> Option<(ModuleVersion, bool)>
        where H: server::Handler<C>
    {
        //did we agree to this module already?
        if let Some(agreed_version) = conn.is_module_enabled(module_name) {
            //answer consistently: positively if the same major version is requested again,
            //otherwise negatively
            for major_version in major_versions_iter.clone() {
                if major_version == agreed_version.major {
                    return Some((agreed_version, false));
                }
            }
            return None;
        }

        //find the highest major version that we can agree to
        let mut best_major: u16 = 0;
        let mut best_minor: u16 = 0;
        for major_version in major_versions_iter {
            if major_version > best_major {
                let can_use_module_result = (self as &server::Handler<C>).can_use_module(
                    module_name, major_version, conn);
                if let Some(minor_version) = can_use_module_result {
                    best_major = major_version;
                    best_minor = minor_version;
                }
            }
        }
        if best_major == 0 {
            None
        } else {
            Some((ModuleVersion { major: best_major, minor: best_minor }, true))
        }
    }

    fn subscribe_to_property<C: Connection>(&self, msg: &msg::Message, conn: &mut C, send_buffer: &mut [u8]) -> Option<usize>
    where H: server::Handler<C>
    {
        //expect exactly one argument (property name)
        let mut args = msg.arguments();
        let name = str::from_utf8(args.next()?).ok()?;
        if is_scoped_identifier(name).is_none() {
            return None;
        }
        if args.len() != 0 {
            return None;
        }

        (self as &server::Handler<C>).subscribe_to_property(name, conn, send_buffer)
    }

    fn set_property<C: Connection>(&self, msg: &msg::Message, conn: &mut C, send_buffer: &mut [u8]) -> Option<usize>
        where H: server::Handler<C>
    {
        //expects exactly two arguments (property name and requested value)
        let mut args = msg.arguments();
        let name = str::from_utf8(args.next()?).ok()?;
        if is_scoped_identifier(name).is_none() {
            return None;
        }
        let requested_value = args.next()?;
        if args.len() != 0 {
            return None;
        }

        (self as &server::Handler<C>).set_property(name, requested_value, conn, send_buffer)
    }
}

impl<C: Connection, H: server::Handler<C>> server::EarlyHandler<C> for Handler<H> {
    fn handle(&self, msg: &msg::Message, conn: &mut C, send_buffer: &mut [u8]) -> Option<usize> {
        let has_core1 = conn.is_module_enabled("core").map_or(false, |version| version.major == 1);
        match msg.type_name() {
            ("", "want")                 => self.handle_want(msg, conn, send_buffer),
            ("core", "sub") if has_core1 => self.subscribe_to_property(msg, conn, send_buffer),
            ("core", "set") if has_core1 => self.set_property(msg, conn, send_buffer),
            //forward unrecognized messages to next handler
            _ => self.next.handle(msg, conn, send_buffer),
        }
    }
}

impl<C: Connection, H: server::Handler<C>> server::Handler<C> for Handler<H> {
    fn handle(&self, msg: &msg::Message, conn: &mut C, send_buffer: &mut [u8]) -> Option<usize> {
        (self as &server::EarlyHandler<C>).handle(msg, conn, send_buffer)
    }

    fn can_use_module(&self, name: &str, major_version: u16, conn: &C) -> Option<u16> {
        if name == "core" {
            if major_version == 1 { Some(0) } else { None }
        } else {
            self.next.can_use_module(name, major_version, conn)
        }
    }

    fn subscribe_to_property(&self, name: &str, conn: &mut C, send_buffer: &mut [u8]) -> Option<usize> {
        use common::core::msg::prerecorded::publish_property;
        if name == "core.server-msg-bytes-max" {
            publish_property(send_buffer, name, &conn.max_server_message_length())
        } else if name == "core.client-msg-bytes-max" {
            publish_property(send_buffer, name, &conn.max_client_message_length())
        } else {
            self.next.subscribe_to_property(name, conn, send_buffer)
        }
    }

    fn set_property(&self, name: &str, requested_value: &[u8], conn: &mut C, send_buffer: &mut [u8]) -> Option<usize> {
        use common::core::msg::prerecorded::publish_property;
        // we do not support changing any properties yet, so just return the
        // current value
        if name == "core.server-msg-bytes-max" {
            publish_property(send_buffer, name, &conn.max_server_message_length())
        } else if name == "core.client-msg-bytes-max" {
            publish_property(send_buffer, name, &conn.max_client_message_length())
        } else {
            self.next.set_property(name, requested_value, conn, send_buffer)
        }
    }
}
