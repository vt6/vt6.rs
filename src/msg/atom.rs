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

use std::cmp::Ordering;
use std::fmt;

use msg::parse;

///An atom is either a bareword or a quoted string, as defined in
///[vt6/core1.0, section 2.1](https://vt6.io/std/core/1.0/#section-2-1).
///
///An atom can be converted into:
///
///* the string value it represents with `atom.as_ref()` or just `&atom`,
///* its most compact encoding inside a VT6 message with `format!("{}", &atom)`.
#[derive(Clone,Debug)]
pub struct Atom {
    unquoted: String,
    quoted: String,
}

impl AsRef<str> for Atom {
    fn as_ref(&self) -> &str {
        self.unquoted.as_ref()
    }
}

impl PartialEq for Atom {
    fn eq(&self, other: &Atom) -> bool { self.unquoted == other.unquoted }
}

impl Eq for Atom {}

impl PartialOrd for Atom {
    fn partial_cmp(&self, other: &Atom) -> Option<Ordering> { self.unquoted.partial_cmp(&other.unquoted) }
    fn lt(&self, other: &Atom) -> bool { self.unquoted <  other.unquoted }
    fn le(&self, other: &Atom) -> bool { self.unquoted <= other.unquoted }
    fn gt(&self, other: &Atom) -> bool { self.unquoted >  other.unquoted }
    fn ge(&self, other: &Atom) -> bool { self.unquoted >= other.unquoted }
}

impl Ord for Atom {
    fn cmp(&self, other: &Atom) -> Ordering { self.unquoted.cmp(&other.unquoted) }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.quoted)
    }
}

impl Atom {
    ///Constructs an atom representing the given string value.
    pub fn new(s: String) -> Atom {
        let q = add_quotes(&s);
        Atom{
            unquoted: s,
            quoted: q,
        }
    }

    ///Parses a bareword or quoted strings. Before the call, `state.cursor` must point to its first
    ///character (or, for quoted strings, the opening quote), or whitespace before it. After the
    ///call, `state.cursor` will point to the position directly following the last character (or,
    ///for quoted strings, the closing quote).
    pub fn parse<'a>(mut state: &'a mut parse::ParserState) -> parse::ParseResult<Atom> {
        parse::parse_atom(&mut state)
    }
}

fn add_quotes(input: &String) -> String {
    let mut to_escape: usize = 0;
    for c in input.chars() {
        if c == '\\' || c == '\"' {
            to_escape += 1;
        }
    }
    if to_escape == 0 {
        return input.clone();
    }
    let mut s = String::from("\"");
    s.reserve_exact(input.len() + to_escape + 1);
    for c in input.chars() {
        if c == '\\' || c == '\"' {
            s.push('\\');
        }
        s.push(c);
    }
    s.push('"');
    s
}
