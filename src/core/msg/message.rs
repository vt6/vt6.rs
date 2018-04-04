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

use core::msg::sexp::{Element, SExpression};
use core::msg::parse::{self, Parse};

use std::fmt;

///This struct represents a message, as defined in
///[vt6/core1.0, section 2.2](https://vt6.io/std/core/1.0/#section-2-2).
///
///To parse a message, use the methods provided by the Parse trait.
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Message(pub SExpression);

impl Message {
    ///Consumes `sexp` and returns `Ok(Message(sexp))` if it is a valid message.
    pub fn from(sexp: SExpression) -> parse::ParseResult<Message> {
        {
            let type_atom = match sexp.0.first() {
                Some(&Element::Atom(ref atom)) => atom,
                Some(_) => return make_error(parse::ParseErrorKind::InvalidMessageType),
                None    => return make_error(parse::ParseErrorKind::EmptyMessage),
            };
            if !type_atom.is_message_type() {
                return make_error(parse::ParseErrorKind::InvalidMessageType);
            }
        } // stop borrowing `sexp`
        Ok(Message(sexp))
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

fn make_error(kind: parse::ParseErrorKind) -> parse::ParseResult<Message> {
    Err(parse::ParseError { offset: 0, kind: kind })
}

impl Parse for Message {
    fn parse<'a>(state: &'a mut parse::ParserState) -> parse::ParseResult<Message> {
        Message::from(SExpression::parse(state)?)
    }
}
