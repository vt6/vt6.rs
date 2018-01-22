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

use std::fmt;
use std::fmt::Write;

use core::msg::parse;

///An atom is either a bareword or a quoted string, as defined in
///[vt6/core1.0, section 2.1](https://vt6.io/std/core/1.0/#section-2-1).
///
///An atom can be converted into its most compact encoding inside a VT6 message
///with `format!("{}", &atom)`.
///
///Instances behave like String instances, e.g. `&atom` is equivalent to
///`atom.value` and equality is defined as `self.value == other.value`.
#[derive(Clone,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub struct Atom {
    ///The string value represented by this atom.
    pub value: String,
    ///For parsed atoms, this field is true when the atom was represented as a
    ///quoted string in the source. This attribute is only interesting in places
    ///where barewords and quoted strings cannot be used interchangably. (Message
    ///types and arguments in the `want` and `have` messages must be encoded as
    ///barewords.)
    pub was_quoted: bool,
}

impl AsRef<str> for Atom {
    fn as_ref(&self) -> &str {
        self.value.as_ref()
    }
}

impl PartialEq<str> for Atom {
    fn eq(&self, other: &str) -> bool { self.value == other }
}
impl PartialEq<Atom> for str {
    fn eq(&self, other: &Atom) -> bool { self == other.value }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for c in self.value.bytes() {
            if !parse::isbareword(c) {
                //cannot encode this character in a bareword -> write a quoted string
                return format_quoted(self, f);
            }
        }
        //can encode this string as a bareword (in other words, no encoding
        //needed)
        f.write_str(&self.value)
    }
}

fn format_quoted(a: &Atom, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_char('"')?;
    for c in a.value.chars() {
        if c == '\\' || c == '\"' {
            f.write_char('\\')?;
        }
        f.write_char(c)?;
    }
    f.write_char('"')
}

impl Atom {
    ///Constructs an atom representing the given string value.
    pub fn new(s: String) -> Atom {
        //NOTE: This constructor is *not* used by the parsing code since it
        //needs to control the value of the `was_quoted` attribute.
        Atom{ value: s, was_quoted: false }
    }

    ///Parses a bareword or quoted strings. Before the call, `state.cursor` must point to its first
    ///character (or, for quoted strings, the opening quote), or whitespace before it. After the
    ///call, `state.cursor` will point to the position directly following the last character (or,
    ///for quoted strings, the closing quote).
    pub fn parse<'a>(mut state: &'a mut parse::ParserState) -> parse::ParseResult<Atom> {
        parse::parse_atom(&mut state)
    }
}
