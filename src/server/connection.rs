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

use core::ModuleVersion;

///Encapsulates the server connection as far as required by [server
///handlers](trait.Handler.html). Specific handlers may require additional
///traits beyond this one.
///
///For applications using std, the methods `enable_module` and
///`is_module_enabled` are implemented by
///[vt6::core::server::Tracker](../core/server/struct.Tracker.html), so
///implementors of Connection can just hold a Tracker instance and forward those
///methods to it. Applications using no_std can provide their own non-allocating
///implementations of these methods instead.
pub trait Connection {
    ///Appends content to the send buffer of the connection. This can be used in
    ///conjunction with
    ///[vt6::core::msg::MessageFormatter](../core/msg/struct.MessageFormatter.html)
    ///to send messages:
    ///
    ///```rust,ignore
    ///use vt6::core::msg::MessageFormatter;
    ///conn.write_to_send_buffer(|buf| {
    ///    MessageFormatter::format(buf, "core.pub", 2, |fmt| {
    ///        fmt.add_argument("example.title")?;
    ///        fmt.add_argument("Hello World")
    ///    }).unwrap_or(0)
    ///});
    ///```
    ///
    ///When `action` is executed, the argument `buf` is the writable part of the
    ///send buffer. The return value of `action` shall be the number of bytes
    ///written into `buf` (at its start, at `&buf[0..byte_count]`).
    ///
    ///The `action` may be executed lazily, possibly on a different thread. When
    ///`write_to_send_buffer()` is called multiple times by the same thread, it
    ///is guaranteed that the provided `action` closures will be executed in the
    ///same order, therefore preserving the intended order in which messages are
    ///sent.
    fn write_to_send_buffer<F>(&mut self, action: F)
        where F: FnOnce(&mut [u8]) -> usize + Send;

    ///Record the fact that the server handler agrees to using the given module
    ///version on this connection.
    ///
    ///Callers (i.e., server handlers) are expected to perform the necessary
    ///validation of module dependencies. enable_module() may not perform any
    ///validation.
    ///
    ///# Panics
    ///
    ///May panic when enable_module() is called multiple times for the same module.
    ///Callers should check is_module_enabled() before calling enable_module().
    fn enable_module(&mut self, name: &str, version: ModuleVersion);
    ///When enable_module() has been called for this module before, returns the
    ///module version that has been agreed to. Returns None otherwise.
    fn is_module_enabled(&self, name: &str) -> Option<ModuleVersion>;
}
