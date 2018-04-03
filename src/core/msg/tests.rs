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

use core::msg::*;

fn parse_atom(input: &[u8]) -> ParseResult<Atom> {
    let mut state = ParserState::new(&input[..]);
    Atom::parse(&mut state)
}
fn parse_sexp(input: &[u8]) -> ParseResult<SExpression> {
    let mut state = ParserState::new(&input[..]);
    SExpression::parse(&mut state)
}

#[test]
fn parse_barewords() {
    assert_eq!(parse_atom(b"core.set"),
        Ok(Atom { value: String::from("core.set"), was_quoted: false }),
    );
    assert_eq!(parse_atom(b"42x"),
        Ok(Atom { value: String::from("42x"), was_quoted: false }),
    );
    assert_eq!(parse_atom(b"?set"),
        Err(ParseError { kind: ParseErrorKind::InvalidToken, offset: 0 }),
    );
}

#[test]
fn parse_quoted_strings() {
    assert_eq!(parse_atom(b"\"core.set\""),
        Ok(Atom { value: String::from("core.set"), was_quoted: true }),
    );
    assert_eq!(parse_atom(br#""42\\x""#),
        Ok(Atom { value: String::from("42\\x"), was_quoted: true }),
    );
    assert_eq!(parse_atom(br#""?set""#),
        Ok(Atom { value: String::from("?set"), was_quoted: true }),
    );
    assert_eq!(parse_atom(br#""foo\nbar""#),
        Err(ParseError { kind: ParseErrorKind::UnknownEscapeSequence, offset: 4 }),
    );
}

//A shorthand to make the literals down below more compact.
fn make_atom(was_quoted: bool, value: &'static str) -> Atom {
    Atom { was_quoted: was_quoted, value: String::from(value) }
}

#[test]
fn parse_sexpressions() {
    assert_eq!(parse_sexp(b"()"),
        Ok(SExpression(vec![])),
    );
    assert_eq!(parse_sexp(br#"(aaa)"#),
        Ok(SExpression(vec![
            Element::Atom(make_atom(false, "aaa")),
        ])),
    );
    assert_eq!(parse_sexp(br#"(aaa (bbb) fff)"#),
        Ok(SExpression(vec![
            Element::Atom(make_atom(false, "aaa")),
            Element::SExpression(SExpression(vec![
                Element::Atom(make_atom(false, "bbb")),
            ])),
            Element::Atom(make_atom(false, "fff")),
        ])),
    );
    assert_eq!(parse_sexp(br#"(aaa (bbb "c\"\\d" eee) fff)"#),
        Ok(SExpression(vec![
            Element::Atom(make_atom(false, "aaa")),
            Element::SExpression(SExpression(vec![
                Element::Atom(make_atom(false, "bbb")),
                Element::Atom(make_atom(true, "c\"\\d")),
                Element::Atom(make_atom(false, "eee")),
            ])),
            Element::Atom(make_atom(false, "fff")),
        ])),
    );
}

#[test]
fn serialize_message() {
    let msg = SExpression(vec![
        Element::Atom(Atom::new(String::from("test"))),
        Element::Atom(Atom::new(String::from(r#"a"\"b"#))),
    ]);
    assert_eq!(format!("{}", msg), r#"(test "a\"\\\"b")"#);
}
