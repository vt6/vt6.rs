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

///THIS MODULE IS PRIVATE! The public exports are used by sibling modules (vt6::core::msg::atom and
///vt6::core::msg:sexp), and some things are reexported into the globally public API by the
///vt6::core::msg module.

use std::error::Error;
use std::fmt;

use core::msg::atom::Atom;
use core::msg::sexp::{Element, SExpression};

////////////////////////////////////////////////////////////////////////////////
// result and error types

///Enumeration of the kinds of errors that can occur in the parsing functions in this module.
///See `struct ParseError` for details.
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum ParseErrorKind {
    UnexpectedEOF,
    InvalidUTF8,
    InvalidToken,
    InvalidWithinBareword,
    UnknownEscapeSequence,
}

impl ParseErrorKind {
    ///Returns a human-readable name for this kind.
    pub fn to_str(&self) -> &'static str {
        match *self {
            ParseErrorKind::UnexpectedEOF => "unexpected EOF",
            ParseErrorKind::InvalidUTF8 => "invalid UTF-8",
            ParseErrorKind::InvalidToken => "unexpected character at start of token",
            ParseErrorKind::InvalidWithinBareword => "unexpected character within bareword",
            ParseErrorKind::UnknownEscapeSequence => "unknown escape sequence",
        }
    }
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.to_str())
    }
}

///The error type that is returned by the parsing functions in this module.
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct ParseError {
    ///The position within the original bytestring (the `buffer` attribute of
    ///vt6::core::msg::ParserState).
    pub offset: usize,
    ///The kind of parse error that occurred.
    pub kind: ParseErrorKind,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Parse error at offset {}: {}", self.offset, self.kind)
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        self.kind.to_str()
    }
}

///The Result type that is returned by the parsing functions in this module.
pub type ParseResult<T> = Result<T, ParseError>;

////////////////////////////////////////////////////////////////////////////////
// struct ParserState

///This holds the state used by the parsing functions in this module.
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct ParserState<'a> {
    ///The byte string whose contents (or parts thereof) shall be parsed. This is only read, never
    ///modified during parsing.
    pub buffer: &'a [u8],
    ///Points to the next char that will be read (initially 0, up to `buffer.len()` after all
    ///characters have been consumed). This will move forward during parsing.
    pub cursor: usize,
}

impl<'a> Iterator for ParserState<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<u8> { self.advance().ok() }
}

impl<'a> ParserState<'a> {
    ///Constructs a new ParserState pointing to the front of `buffer`.
    pub fn new(buffer: &'a [u8]) -> Self {
        ParserState { buffer: buffer, cursor: 0 }
    }

    //assorted helper methods to make the parsing functions shorter
    fn error<T>(&self, kind: ParseErrorKind) -> ParseResult<T> {
        Err(ParseError { offset: self.cursor, kind: kind })
    }
    fn current(&self) -> ParseResult<u8> {
        if self.cursor < self.buffer.len() {
            Ok(self.buffer[self.cursor])
        } else {
            self.error(ParseErrorKind::UnexpectedEOF)
        }
    }
    fn advance(&mut self) -> ParseResult<u8> {
        self.cursor += 1;
        self.current()
    }
}

////////////////////////////////////////////////////////////////////////////
//parsing functions

//This is exported for internal use by vt6::core::msg::atom. Note that the
//module as a whole is private, so this does not end up in the public API.
pub fn isbareword(c: u8) -> bool {
    (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z') || (c >= b'0' && c <= b'9') || c == b'.' || c == b'-' || c == b'_'
}
fn isspace(c: u8) -> bool {
    c == b' ' || (c >= 9 && c <= 13)
}

fn parse_bareword(state: &mut ParserState) -> ParseResult<Atom> {
    //This expects the cursor to point to the first char of a bareword atom.
    match state.current()? {
        c if isbareword(c) => {
            //parse bareword (`c` is added to the front of the vector explicitly because
            //take_while() will start by calling next() and therefore see the 2nd char of the
            //bareword and beyond)
            let mut bareword = vec![c];
            bareword.extend(state.take_while(|ch| isbareword(*ch)));
            let value = String::from_utf8_lossy(&bareword).into_owned();
            Ok(Atom { value: value, was_quoted: false })
        },
        _ => state.error(ParseErrorKind::InvalidToken),
    }
}

fn parse_quoted_string(state: &mut ParserState) -> ParseResult<Atom> {
    //This expects `input.current` to point to the opening quote of a quoted string.
    let mut string_buffer: Vec<u8> = vec![];
    loop {
        match state.advance()? {
            b'"' => break,
            b'\\' => {
                let c = state.advance()?;
                if c == b'\\' || c == b'"' {
                    string_buffer.push(c);
                } else {
                    state.cursor -= 1; //report error where escape sequence started
                    return state.error(ParseErrorKind::UnknownEscapeSequence);
                };
            },
            c => string_buffer.push(c),
        }
    }
    state.cursor += 1;
    match String::from_utf8(string_buffer) {
        Ok(s) => Ok(Atom { value: s, was_quoted: true }),
        Err(_) => state.error(ParseErrorKind::InvalidUTF8),
    }
}

pub fn parse_atom(state: &mut ParserState) -> ParseResult<Atom> {
    match state.current()? {
        b'"' => parse_quoted_string(state),
        c if isbareword(c) => parse_bareword(state),
        c if isspace(c) => {
            //consume leading whitespace
            state.cursor += 1;
            return parse_atom(state);
        },
        _ => return state.error(ParseErrorKind::InvalidToken),
    }
}

pub fn parse_sexp(state: &mut ParserState) -> ParseResult<SExpression> {
    match state.current()? {
        c if isspace(c) => {
            //consume leading whitespace
            state.cursor += 1;
            return parse_sexp(state);
        },
        b'(' => {},
        _ => return state.error(ParseErrorKind::InvalidToken),
    }
    //consume opening paren
    state.cursor += 1;

    let mut elements: Vec<Element> = vec![];
    loop {
        match state.current()? {
            b'(' => { elements.push(Element::SExpression(parse_sexp(state)?)); },
            b')' => {
                //consume closing paren
                state.cursor += 1;
                return Ok(SExpression(elements));
            },
            b'"' => { elements.push(Element::Atom(parse_quoted_string(state)?)); },
            //skip over whitespace between elements
            c if isspace(c) => { state.cursor += 1; },
            c if isbareword(c) => { elements.push(Element::Atom(parse_bareword(state)?)); },
            _ => return state.error(ParseErrorKind::InvalidToken),
        }
    }
}
