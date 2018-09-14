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

use common::core::{EncodeArgument, msg};

///Convenience function for formatting a `core.pub` message into the given
///`buffer`. This is intended for usage by implementations of
///[`vt6::server::Handler::handle_property()`](../../../../server/trait.Handler.html).
///For example,
///
///```rust,ignore
///let result = vt6::common::core::msg::prerecorded::publish_property(buf, "example.counter", &42);
///```
///
///is equivalent to:
///
///```rust,ignore
///let mut f = vt6::common::core::msg::MessageFormatter::new(buf, "core.pub", 2);
///f.add_argument("example.counter");
///f.add_argument(&42);
///let result = f.finalize();
///```
pub fn publish_property<T: EncodeArgument + ?Sized>(buffer: &mut [u8], property_name: &str, property_value: &T) -> Option<usize> {
    let mut f = msg::MessageFormatter::new(buffer, "core.pub", 2);
    f.add_argument(property_name);
    f.add_argument(property_value);
    f.finalize().ok()
}
