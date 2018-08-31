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

///Encapsulates the state of a server connection as far as required by [server
///handlers](trait.Handler.html). Specific handlers may require additional
///traits beyond this one (see documentation/example on the Handler trait).
///
///For applications using std, the methods `enable_module` and
///`is_module_enabled` are implemented by
///[vt6::core::server::Tracker](../core/server/struct.Tracker.html), so
///implementors of Connection can just hold a Tracker instance and forward those
///methods to it. Applications using no_std can provide their own non-allocating
///implementations of these methods instead.
pub trait Connection {
    ///Returns the maximum size of the buffer where messages from the client are
    ///received, or `None` if the buffer can grow arbitrarily.
    fn receive_buffer_size(&self) -> Option<&usize>;
    ///Returns the maximum size of the buffer where messages are formatted
    ///before being sent to the client, or `None` if the buffer can grow
    ///arbitrarily.
    fn send_buffer_size(&self) -> Option<&usize>;

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
