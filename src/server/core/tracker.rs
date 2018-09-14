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

use std::collections::HashMap;

use core::ModuleVersion;

///Tracks which modules a VT6 server has agreed to on a specific server
///connection. This type implements several methods required by [the
///vt6::server::Connection trait](../trait.Connection.html), so
///instances of it are commonly held by implementors of that trait. (See
///documentation over there.)
#[derive(Clone, Default)]
pub struct Tracker {
    agreed_modules: HashMap<String, ModuleVersion>,
}

impl Tracker {
    ///Create a new empty tracker. This is the same as `default()`.
    pub fn new() -> Tracker {
        Tracker::default()
    }

    ///This provides a general-purpose implementation for
    ///[`Connection::enable_module()`](../trait.Connection.html).
    pub fn enable_module(&mut self, name: &str, version: ModuleVersion) {
        match self.agreed_modules.get(name) {
            Some(_) => panic!("cannot enable_module({:?}) twice on the same connection", name),
            None => self.agreed_modules.insert(name.into(), version),
        };
    }

    ///This provides a general-purpose implementation for
    ///[`Connection::is_module_enabled()`](../trait.Connection.html).
    pub fn is_module_enabled(&self, name: &str) -> Option<ModuleVersion> {
        self.agreed_modules.get(name).cloned()
    }
}
