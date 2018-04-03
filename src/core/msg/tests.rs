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

#[test]
fn parse_barewords() {
    assert_eq!(Atom::parse_byte_string(b"core.set"),
        Ok(Atom { value: String::from("core.set"), was_quoted: false }),
    );
    assert_eq!(Atom::parse_byte_string(b" \tcore.set"),
        Ok(Atom { value: String::from("core.set"), was_quoted: false }),
    );
    assert_eq!(Atom::parse_byte_string(b" \tcore.set\t "),
        Err(ParseError { kind: ParseErrorKind::ExpectedEOF, offset: 10 }),
    );
    assert_eq!(Atom::parse_byte_string(b"42x"),
        Ok(Atom { value: String::from("42x"), was_quoted: false }),
    );
    assert_eq!(Atom::parse_byte_string(b"?set"),
        Err(ParseError { kind: ParseErrorKind::InvalidToken, offset: 0 }),
    );
}

#[test]
fn parse_quoted_strings() {
    assert_eq!(Atom::parse_byte_string(b"\"core.set\""),
        Ok(Atom { value: String::from("core.set"), was_quoted: true }),
    );
    assert_eq!(Atom::parse_byte_string(b" \t\"core.set\""),
        Ok(Atom { value: String::from("core.set"), was_quoted: true }),
    );
    assert_eq!(Atom::parse_byte_string(b" \t\"core.set\"\t "),
        Err(ParseError { kind: ParseErrorKind::ExpectedEOF, offset: 12 }),
    );
    assert_eq!(Atom::parse_byte_string(br#""42\\x""#),
        Ok(Atom { value: String::from("42\\x"), was_quoted: true }),
    );
    assert_eq!(Atom::parse_byte_string(br#""?set""#),
        Ok(Atom { value: String::from("?set"), was_quoted: true }),
    );
    assert_eq!(Atom::parse_byte_string(br#""foo\nbar""#),
        Err(ParseError { kind: ParseErrorKind::UnknownEscapeSequence, offset: 4 }),
    );
}

//A shorthand to make the literals down below more compact.
fn make_atom(was_quoted: bool, value: &'static str) -> Atom {
    Atom { was_quoted: was_quoted, value: String::from(value) }
}

#[test]
fn atom_behavior() {
    let a1 = make_atom(false, "abc");
    let a2 = make_atom(true, "abc");
    assert_eq!(&a1, "abc");
    assert_eq!(&a2, "abc");
    assert_eq!("abc", &a1);
    assert_eq!("abc", &a2);
}

#[test]
fn parse_sexpressions() {
    assert_eq!(SExpression::parse_byte_string(b"()"),
        Ok(SExpression(vec![])),
    );
    assert_eq!(SExpression::parse_byte_string(br#"(aaa)"#),
        Ok(SExpression(vec![
            Element::Atom(make_atom(false, "aaa")),
        ])),
    );
    assert_eq!(SExpression::parse_byte_string(br#"(aaa (bbb) fff)"#),
        Ok(SExpression(vec![
            Element::Atom(make_atom(false, "aaa")),
            Element::SExpression(SExpression(vec![
                Element::Atom(make_atom(false, "bbb")),
            ])),
            Element::Atom(make_atom(false, "fff")),
        ])),
    );
    assert_eq!(SExpression::parse_byte_string(br#"(aaa (bbb "c\"\\d" eee) fff)"#),
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

fn parse_sexp_get_error_msg(input: &[u8]) -> String {
    format!("{}", SExpression::parse_byte_string(input).unwrap_err())
}

#[test]
fn check_error_messages() {
    assert_eq!(
        parse_sexp_get_error_msg(b"(foo (bar"),
        "Parse error at offset 9: unexpected EOF",
    );
    assert_eq!(
        parse_sexp_get_error_msg(b"(foo \"aaa\x80bbb\")"),
        "Parse error at offset 9: invalid UTF-8",
    );
    assert_eq!(
        parse_sexp_get_error_msg(b"(foo [bar])"),
        "Parse error at offset 5: unexpected character at start of token",
    );
    assert_eq!(
        parse_sexp_get_error_msg(b"(foo \"bar\\nbaz\")"),
        "Parse error at offset 9: unknown escape sequence",
    );
}
