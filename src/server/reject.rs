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

use core::{msg, EncodeArgument};
use server::*;

///A [handler](trait.Handler.html) that rejects all messages and requests sent
///to it. This handler is usually the last in the chain of handlers used by a
///server.
///
///Refer to the documentation on [the Handler trait](trait.Handler.html) for
///usage examples.
#[derive(Clone)]
pub struct RejectHandler {}

impl<C: Connection> Handler<C> for RejectHandler {
    fn handle(&self, _msg: &msg::Message, _conn: &mut C) -> Result<(), HandlerError> {
        Ok(())
    }

    fn can_use_module(&self, _name: &str, _major_version: u16, _conn: &C) -> Option<u16> {
        None
    }

    fn get_set_property<'c>(&self, _name: &str, _requested_value: Option<&[u8]>, _conn: &'c mut C) -> Option<&'c EncodeArgument> {
        None
    }
}

impl<C: Connection> EarlyHandler<C> for RejectHandler {
    fn handle(&self, _msg: &msg::Message, _conn: &mut C) -> Result<(), HandlerError> {
        Ok(())
    }
}
