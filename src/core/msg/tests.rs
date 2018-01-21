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

use core::msg::{Atom, Element, ParserState, SExpression};

#[test]
fn parse_sexp_empty_sexp() {
    let input = b"()";
    let mut state = ParserState::new(&input[..]);
    let parsed = SExpression(vec![]);
    assert_eq!(SExpression::parse(&mut state), Ok(parsed));
}

#[test]
fn parse_sexp_atom() {
    let input = br#"(aaa)"#;
    let mut state = ParserState::new(&input[..]);
    let parsed = SExpression(vec![
        Element::Atom(Atom::new("aaa".to_owned())),
    ]);
    assert_eq!(SExpression::parse(&mut state), Ok(parsed));
}

#[test]
fn parse_sexp_atom_and_sexp() {
    let input = br#"(aaa (bbb) fff)"#;
    let mut state = ParserState::new(&input[..]);
    let parsed = SExpression(vec![
        Element::Atom(Atom::new("aaa".to_owned())),
        Element::SExpression(SExpression(vec![
            Element::Atom(Atom::new("bbb".to_owned())),
        ])),
        Element::Atom(Atom::new("fff".to_owned())),
    ]);
    assert_eq!(SExpression::parse(&mut state), Ok(parsed));
}

#[test]
fn parse_sexp_quoted() {
    let input = br#"(aaa (bbb "c\"\\d" eee) fff)"#;
    let mut state = ParserState::new(&input[..]);
    let parsed = SExpression(vec![
        Element::Atom(Atom::new("aaa".to_owned())),
        Element::SExpression(SExpression(vec![
            Element::Atom(Atom::new("bbb".to_owned())),
            Element::Atom(Atom::new("c\"\\d".to_owned())),
            Element::Atom(Atom::new("eee".to_owned())),
        ])),
        Element::Atom(Atom::new("fff".to_owned())),
    ]);
    assert_eq!(SExpression::parse(&mut state), Ok(parsed));
}

#[test]
fn serialize_message() {
    let msg = SExpression(vec![
        Element::Atom(Atom::new("test".to_owned())),
        Element::Atom(Atom::new(r#"a"\"b"#.to_owned())),
    ]);
    assert_eq!(format!("{}", msg), r#"(test "a\"\\\"b")"#);
}
