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

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufRead};

use regex::Regex;

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
    let msg = Message(SExpression(vec![
        Element::Atom(Atom::new(String::from("core1.test"))),
        Element::Atom(Atom::new(String::from(r#"a"\"b"#))),
    ]));
    assert_eq!(format!("{}", msg), r#"(core1.test "a\"\\\"b")"#);
}

fn parse_sexp_get_error_msg(input: &[u8]) -> String {
    format!("{}", SExpression::parse_byte_string(input).unwrap_err())
}
fn parse_message_get_error_msg(input: &[u8]) -> String {
    format!("{}", Message::parse_byte_string(input).unwrap_err())
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
    assert_eq!(
        parse_message_get_error_msg(b"(foo bar)"),
        "Parse error at offset 0: invalid message type",
    );
    assert_eq!(
        parse_message_get_error_msg(b"((want core1) core2)"),
        "Parse error at offset 0: invalid message type",
    );
    assert_eq!(
        parse_message_get_error_msg(b"()"),
        "Parse error at offset 0: missing message type",
    );
    assert_eq!(
        parse_message_get_error_msg(b"(want core1)(something else)"),
        "Parse error at offset 12: expected EOF",
    );
}

#[test]
fn conformance_test_parse_sexp() {
    let file = File::open("../conformance-tests/core/parse-sexp.txt").unwrap();
    let mut current_input = String::from("");
    let mut current_sexp: ParseResult<SExpression> = Ok(SExpression(Vec::new()));

    let test_rx = Regex::new(r"^test\s*(.+)$").unwrap();
    let ok_rx = Regex::new(r"^ok\s*(.+)$").unwrap();
    let error_rx = Regex::new(r"^error\s*(\d+)\s*(.+)$").unwrap();

    let error_map = {
        let mut m = HashMap::new();
        m.insert("expected EOF", ParseErrorKind::ExpectedEOF);
        m.insert("invalid escape sequence", ParseErrorKind::UnknownEscapeSequence);
        m.insert("invalid start of atom", ParseErrorKind::InvalidToken);
        m.insert("invalid start of S-expression", ParseErrorKind::InvalidToken);
        m.insert("invalid UTF-8", ParseErrorKind::InvalidUTF8);
        m.insert("unexpected EOF in quoted string", ParseErrorKind::UnexpectedEOF);
        m.insert("unexpected EOF in S-expression", ParseErrorKind::UnexpectedEOF);
        m
    };

    for (idx, line) in BufReader::new(file).lines().enumerate() {
        let line = line.unwrap();
        if line.starts_with("#") {
            continue;
        }
        if let Some(caps) = test_rx.captures(&line) {
            let input = unescape_str(&caps[1]);
            current_input = String::from_utf8_lossy(&input).into();
            current_sexp = SExpression::parse_byte_string(&input);
        } else if let Some(caps) = ok_rx.captures(&line) {
            let sexp_str = String::from_utf8(unescape_str(&caps[1])).unwrap();
            assert_eq!(
                current_sexp.as_ref().map(serialize_sexp), Ok(sexp_str),
                "\ninput: {:?} at line {}", current_input, idx + 1,
            );
        } else if let Some(caps) = error_rx.captures(&line) {
            assert!(current_sexp.is_err(),
                "\n input: {:?} at line {}\noutput: {:?}", current_input, idx + 1, current_sexp,
            );
            if let Err(ref err) = current_sexp {
                assert_eq!(err.offset, caps[1].parse().unwrap(),
                    "\n input: {:?} at line {}\noutput: Err({:?})", current_input, idx + 1, err,
                );
                assert_eq!(Some(&err.kind), error_map.get(&caps[2]),
                    "\n input: {:?} at line {}\noutput: Err({:?})", current_input, idx + 1, err,
                );
            }
        }
    }
}

fn unescape_str(input: &str) -> Vec<u8> {
    let escape_rx = Regex::new(r"\\x([0-9A-F]{2})|(.)").unwrap();
    let mut result = Vec::with_capacity(input.len());
    for caps in escape_rx.captures_iter(input) {
        if let Some(match1) = caps.get(1) {
            result.push(u8::from_str_radix(match1.as_str(), 16).unwrap());
        } else {
            result.extend_from_slice(caps[2].as_bytes());
        }
    }
    result
}

fn serialize_sexp(input: &SExpression) -> String {
    format!("{}", input)
}
