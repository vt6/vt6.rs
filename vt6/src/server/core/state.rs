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

use std::time::Instant;

///Enumerates the modes of a bidirectional byte-stream, as specified in
///[vt6/core1.0, section 1.2](https://vt6.io/std/core/1.0/#section-1-2).
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum StreamMode {
    Message,
    Stdio,
    #[cfg(unix)]
    Stdin,
    #[cfg(unix)]
    Stdout,
}

///The state of a bidirectional byte-stream, as specified in
///[vt6/core1.0, section 1.2](https://vt6.io/std/core/1.0/#section-1-2).
///
///Connections must hold one of these for manipulation by handlers.
///See [`trait Connection`](../trait.Connection.html) for details.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct StreamState {
    ///The mode that this stream is currently in.
    pub mode: StreamMode,
    ///When the current mode was entered.
    pub entered: Instant,
}

impl StreamState {
    ///Return a new `StreamState` instance that indicates the given mode as
    ///having been entered at the time of the call of this method.
    pub fn enter(mode: StreamMode) -> StreamState {
        StreamState {
            mode:    mode,
            entered: Instant::now(),
        }
    }
}
