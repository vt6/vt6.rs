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

use common::core::EncodeArgument;

use std::fmt;

///A module version like "2.3", as defined by the `<full-version>` grammar element
///in [vt6/core1.0, section 1.5](https://vt6.io/std/core/1.0/#section-1-5).
#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Debug)]
pub struct ModuleVersion {
    pub major: u16,
    pub minor: u16,
}

impl fmt::Display for ModuleVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

//NOTE: Tests for this trait impl are in 
impl EncodeArgument for ModuleVersion {
    fn get_size(&self) -> usize {
        self.major.get_size() + 1 + self.minor.get_size()
    }

    fn encode(&self, buf: &mut[u8]) {
        let major_size = self.major.get_size();
        self.major.encode(&mut buf[0 .. major_size]);
        buf[major_size] = b'.';
        self.minor.encode(&mut buf[major_size+1 .. ]);
    }
}
