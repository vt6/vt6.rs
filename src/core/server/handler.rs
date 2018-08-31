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

use libcore::str;
use std::marker::PhantomData;

use core::*;
use server::{self, Connection, HandlerError, try_or_message_invalid};

///A [handler](../../server/trait.Handler.html) that implements the [vt6/core
///module](https://vt6.io/std/core/).
///
///This handler is notable because every handler preceding it implements the
///[EarlyHandler trait](../../server/trait.EarlyHandler.html), but every handler
///succeeding it implements the [Handler trait](../../server/trait.Handler.html).
pub struct Handler<C: Connection, H: server::Handler<C>> {
    next: H,
    phantom: PhantomData<C>,
}

//The auto trait implementations of Send and Sync add a "where C: Send/Sync" bound because
//PhantomData<C> usually implies ownership of some C, but that's not true here.
unsafe impl<C: Connection, H: server::Handler<C>> Send for Handler<C, H> where H: Send {}
unsafe impl<C: Connection, H: server::Handler<C>> Sync for Handler<C, H> where H: Sync {}

impl<C: Connection, H: server::Handler<C>> Handler<C, H> {
    ///Constructor. The argument is the next handler after this handler.
    pub fn new(next: H) -> Self {
        Handler { next: next, phantom: PhantomData }
    }

    fn handle_want(&self, msg: &msg::Message, conn: &mut C, send_buffer: &mut [u8]) -> Result<usize, HandlerError> {
        //validate arguments: first argument is module name
        let mut args_iter = msg.arguments();
        let module_name = try_or_message_invalid(|| str::from_utf8(args_iter.next()?).ok())?;
        if !is_identifier(module_name) {
            return Err(HandlerError::InvalidMessage);
        }

        //validate arguments: remaining arguments are major versions, need at least one
        if args_iter.len() == 0 {
            return Err(HandlerError::InvalidMessage);
        }
        for arg in args_iter.clone() {
            let major_version = try_or_message_invalid(|| str::from_utf8(arg).ok()?.parse::<u16>().ok())?;
            if major_version == 0 {
                return Err(HandlerError::InvalidMessage);
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
                f.finalize_or_handler_error()
            },
            None => {
                msg::MessageFormatter::new(send_buffer, "have", 0)
                    .finalize_or_handler_error()
            },
        }
    }

    fn check_want<I: Iterator<Item=u16> + Clone>(&self, module_name: &str, major_versions_iter: I, conn: &C) -> Option<(ModuleVersion, bool)> {
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

    fn handle_set_sub(&self, msg: &msg::Message, conn: &mut C, send_buffer: &mut [u8], is_set: bool) -> Result<usize, HandlerError> {
        //expect exactly one argument (property name) for core.sub, or exactly two
        //arguments (property name and requested value) for core.set
        let mut args = msg.arguments();
        let name = try_or_message_invalid(|| str::from_utf8(args.next()?).ok())?;
        if is_scoped_identifier(name).is_none() {
            return Err(HandlerError::InvalidMessage);
        }
        let requested_value = if is_set {
            Some(try_or_message_invalid(|| args.next())?)
        } else {
            None
        };
        if args.len() != 0 {
            return Err(HandlerError::InvalidMessage);
        }

        let result = (self as &server::Handler<C>)
            .get_set_property(name, requested_value, conn);
        match result {
            None => Err(HandlerError::InvalidMessage),
            Some(value) => {
                let mut f = msg::MessageFormatter::new(send_buffer, "core.pub", 2);
                f.add_argument(name);
                f.add_argument(value);
                f.finalize_or_handler_error()
            },
        }
    }
}

impl<C: Connection, H: server::Handler<C>> server::EarlyHandler<C> for Handler<C, H> {
    fn handle(&self, msg: &msg::Message, conn: &mut C, send_buffer: &mut [u8]) -> Result<usize, HandlerError> {
        let has_core1 = conn.is_module_enabled("core").map_or(false, |version| version.major == 1);
        match msg.type_name() {
            ("", "want")                 => self.handle_want(msg, conn, send_buffer),
            ("core", "sub") if has_core1 => self.handle_set_sub(msg, conn, send_buffer, false),
            ("core", "set") if has_core1 => self.handle_set_sub(msg, conn, send_buffer, true),
            //forward unrecognized messages to next handler
            _ => self.next.handle(msg, conn, send_buffer),
        }
    }
}

impl<C: Connection, H: server::Handler<C>> server::Handler<C> for Handler<C, H> {
    fn handle(&self, msg: &msg::Message, conn: &mut C, send_buffer: &mut [u8]) -> Result<usize, HandlerError> {
        (self as &server::EarlyHandler<C>).handle(msg, conn, send_buffer)
    }

    fn can_use_module(&self, name: &str, major_version: u16, conn: &C) -> Option<u16> {
        if name == "core" {
            if major_version == 1 { Some(0) } else { None }
        } else {
            self.next.can_use_module(name, major_version, conn)
        }
    }

    fn get_set_property<'c>(&self, name: &str, requested_value: Option<&[u8]>, conn: &'c mut C) -> Option<&'c EncodeArgument> {
        //we do not support changing any properties yet, so just return the
        //current value
        if name == "core.server-msg-bytes-max" {
            Some(conn.send_buffer_size().unwrap_or(&1024))
        } else if name == "core.client-msg-bytes-max" {
            Some(conn.receive_buffer_size().unwrap_or(&1024))
        } else {
            self.next.get_set_property(name, requested_value, conn)
        }
    }
}
