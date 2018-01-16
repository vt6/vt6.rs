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

use core::msg::atom::Atom;
use core::msg::parse;

///This enum represents an element of an s-expression, as defined in
///[vt6/core1.0, section 2.1](https://vt6.io/std/core/1.0/#section-2-1).
///
///An element can be converted into its encoding inside a VT6 message with `format!("{}",
///&element)`.
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum Element {
    Atom(Atom),
    SExpression(SExpression),
}

impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Element::Atom(ref a) => a.fmt(f),
            Element::SExpression(ref s) => s.fmt(f),
        }
    }
}

///This struct represents an s-expression, as defined in
///[vt6/core1.0, section 2.1](https://vt6.io/std/core/1.0/#section-2-1).
///
///An s-expression can be converted into its encoding inside a VT6 message with `format!("{}",
///&sexp)`.
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct SExpression(pub Vec<Element>);

impl fmt::Display for SExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0.len() == 0 {
            f.write_str("()")
        } else {
            let mut first = true;
            for expr in self.0.iter() {
                f.write_char(if first { '(' } else { ' ' })?;
                write!(f, "{}", expr)?;
                first = false;
            }
            f.write_char(')')
        }
    }
}

impl SExpression {
    ///Parses an s-expression. Before the call, `state.cursor` must point to its opening
    ///parenthesis, or whitespace before it. After the call, `state.cursor` will point to the
    ///position directly following its closing parenthesis.
    pub fn parse<'a>(mut state: &'a mut parse::ParserState) -> parse::ParseResult<SExpression> {
        parse::parse_sexp(&mut state)
    }
}
