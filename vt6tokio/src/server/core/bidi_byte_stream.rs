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

use std;

use tokio::prelude::*;
use tokio_uds::UnixStream;
use vt6::server as vt6s;

use server::core::Connection;

pub(crate) struct BidiByteStream<C: Connection> {
    pub conn: C,
    stream: UnixStream,
}

impl<C: Connection> BidiByteStream<C> {
    pub fn new(conn: C, stream: UnixStream) -> Self {
        unimplemented!() //TODO
    }

    pub fn poll<H: vt6s::EarlyHandler<C>>(&mut self, handler: &H) -> Poll<(), std::io::Error> {
        unimplemented!() //TODO
    }

    pub fn append_to_send_buffer(&mut self, bytes: &[u8]) {
        unimplemented!() //TODO
    }
}

