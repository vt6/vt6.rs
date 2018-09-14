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

mod encode_argument;
pub use self::encode_argument::*;
mod module_version;
pub use self::module_version::*;

///Parsing and serializing of VT6 messages.
pub mod msg;

///Checks if the argument is an identifier, as defined by
///[vt6/core1.0, section 1.4](https://vt6.io/std/core/1.0/#section-1-4).
///
///This function can be used to validate module names.
///
///```
///# use vt6::common::core::is_identifier;
///assert_eq!(is_identifier("core"),          true);
///assert_eq!(is_identifier("core1"),         false);
///assert_eq!(is_identifier("scoped.ident"),  false);
///assert_eq!(is_identifier("what is: this"), false);
///assert_eq!(is_identifier("_what-is-this"), true);
///```
pub fn is_identifier(name: &str) -> bool {
    let mut iter = name.chars();
    match iter.next() {
        Some(ch) if is_ident_leader(ch) => {},
        _ => return false,
    };
    iter.all(is_ident_char)
}

fn is_ident_leader(ch: char) -> bool {
    (ch >= 'A' && ch <= 'Z') ||
    (ch >= 'a' && ch <= 'z') ||
    ch == '_'
}

fn is_ident_char(ch: char) -> bool {
    (ch >= 'A' && ch <= 'Z') ||
    (ch >= 'a' && ch <= 'z') ||
    ch == '_' ||
    ch == '-'
}

///Checks if the argument is a scoped identifier, as defined by
///[vt6/core1.0, section 1.4](https://vt6.io/std/core/1.0/#section-1-4).
///
///If the argument is a scoped identifier, the pair in `Some` contains the part
///before the dot (the module name) and the part after the dot (the name inside
///the module).
///
///This function can be used to validate property names. It cannot be used to
///validate message types because it does not recognize the eternal message
///types that are not scoped. See [`is_message_type()`](fn.is_message_type.html)
///instead.
///
///```
///# use vt6::common::core::is_scoped_identifier;
///assert_eq!(is_scoped_identifier("core"),          None);
///assert_eq!(is_scoped_identifier("core.set"),      Some(("core", "set")));
///assert_eq!(is_scoped_identifier("what is: this"), None);
///assert_eq!(is_scoped_identifier("_what.is-this"), Some(("_what", "is-this")));
///assert_eq!(is_scoped_identifier("want"),          None);
///assert_eq!(is_scoped_identifier("have"),          None);
///assert_eq!(is_scoped_identifier("nope"),          None);
///assert_eq!(is_scoped_identifier("."),             None);
///assert_eq!(is_scoped_identifier(".foo"),          None);
///assert_eq!(is_scoped_identifier("foo."),          None);
///```
pub fn is_scoped_identifier(name: &str) -> Option<(&str, &str)> {
    let dot_idx = name.find('.')?;
    let (left, right) = (&name[0..dot_idx], &name[dot_idx+1..]);
    if is_identifier(left) && is_identifier(right) {
      Some((left, right))
    } else {
      None
    }
}

///Checks if the argument is a message type, as defined by
///[vt6/core1.0, section 1.4](https://vt6.io/std/core/1.0/#section-1-4).
///
///If the argument is a message type, the pair in `Some` contains the part
///before the dot (the module name) and the part after the dot (the name inside
///the module). As a special case, if the message type is eternal (i.e. not
///scoped to a module), the first string in the pair will be an empty string
///(see examples below).
///
///To validate property names, see
///[`is_scoped_identifier()`](fn.is_scoped_identifier.html) instead.
///
///```
///# use vt6::common::core::is_message_type;
///assert_eq!(is_message_type("core"),          None);
///assert_eq!(is_message_type("core.set"),      Some(("core", "set")));
///assert_eq!(is_message_type("what is: this"), None);
///assert_eq!(is_message_type("_what.is-this"), Some(("_what", "is-this")));
///assert_eq!(is_message_type("want"),          Some(("", "want")));
///assert_eq!(is_message_type("have"),          Some(("", "have")));
///assert_eq!(is_message_type("nope"),          Some(("", "nope")));
///assert_eq!(is_message_type("."),             None);
///assert_eq!(is_message_type(".foo"),          None);
///assert_eq!(is_message_type("foo."),          None);
///```
pub fn is_message_type(name: &str) -> Option<(&str, &str)> {
  if name == "want" || name == "have" || name == "nope" {
    Some(("", name))
  } else {
    is_scoped_identifier(name)
  }
}
