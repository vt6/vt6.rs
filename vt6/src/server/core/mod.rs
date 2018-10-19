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

mod handler;
pub use self::handler::*;
mod state;
pub use self::state::*;

#[cfg(any(test, feature = "use_std"))]
mod tracker;
#[cfg(any(test, feature = "use_std"))]
pub use self::tracker::*;

#[cfg(test)]
mod tests;

use server::Connection as ConnectionBase;

///Extends [`vt6::server::Connection`](../trait.Connection.html) with methods
///required by the [handler](struct.Handler.html) in this module.
pub trait Connection: ConnectionBase {
    ///Returns the maximum length in bytes of client messages that can be
    ///received on this connection.
    fn max_client_message_length(&self) -> usize;
    ///Returns the maximum length in bytes of server messages that can be
    ///sent on this connection.
    fn max_server_message_length(&self) -> usize;
}
