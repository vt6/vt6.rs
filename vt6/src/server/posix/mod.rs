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

use common::core::*;
use server::{self, Connection};

///A handler that implements the vt6/posix module.
///
///This type is not public! It is automatically chained after vt6::core::Handler
///on appropriate platforms.
pub(crate) struct Handler<H> {
    next: H,
}

impl<H> Handler<H> {
    ///Constructor. The argument is the next handler after this handler.
    pub fn new(next: H) -> Self {
        Handler { next: next }
    }

    fn enter_state<C: Connection>(&self, mode: server::core::StreamMode, msg: &msg::Message, conn: &mut C, send_buffer: &mut [u8]) -> Option<usize>
        where H: server::Handler<C>
    {
        //expect no arguments
        if msg.arguments().len() != 0 {
            return None;
        }
        //extra sanity check: expect StreamMode::Message before switching to
        //stdio (this is only defense in depth; we should not have to check this
        //because Handler::handle() is only called while in message mode)
        use server::core::StreamMode::*;
        if conn.stream_state().mode != Message {
            return None;
        }

        conn.set_stream_state(server::core::StreamState::enter(mode));
        let msg_type = match mode {
            Stdin  => "posix.to-stdin",
            Stdout => "posix.to-stdout",
            _ => unreachable!(),
        };
        msg::MessageFormatter::new(send_buffer, msg_type, 0).finalize().ok()
    }
}

impl<C: Connection, H: server::Handler<C>> server::Handler<C> for Handler<H> {
    fn handle(&self, msg: &msg::Message, conn: &mut C, send_buffer: &mut [u8]) -> Option<usize> {
        let has_posix1 = conn.is_module_enabled("posix").map_or(false, |version| version.major == 1);
        use server::core::StreamMode::*;
        match msg.type_name() {
            ("posix", "to-stdin") if has_posix1
                => self.enter_state(Stdin, msg, conn, send_buffer),
            ("posix", "to-stdout") if has_posix1
                => self.enter_state(Stdout, msg, conn, send_buffer),
            //forward unrecognized messages to next handler
            _ => self.next.handle(msg, conn, send_buffer),
        }
    }

    fn can_use_module(&self, name: &str, major_version: u16, conn: &C) -> Option<u16> {
        if name == "posix" {
            if major_version == 1 { Some(0) } else { None }
        } else {
            self.next.can_use_module(name, major_version, conn)
        }
    }

    fn subscribe_to_property(&self, name: &str, conn: &mut C, send_buffer: &mut [u8]) -> Option<usize> {
        self.next.subscribe_to_property(name, conn, send_buffer)
    }

    fn set_property(&self, name: &str, requested_value: &[u8], conn: &mut C, send_buffer: &mut [u8]) -> Option<usize> {
        self.next.set_property(name, requested_value, conn, send_buffer)
    }
}
