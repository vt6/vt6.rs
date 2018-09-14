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

#![cfg_attr(all(not(test), not(feature = "use_std")), no_std)]
#[cfg(all(not(test), not(feature = "use_std")))]
extern crate core as std;

///Common types and definitions that can be used both by VT6 servers and clients.
pub mod common;
///Implementation parts for VT6 servers.
pub mod server;
