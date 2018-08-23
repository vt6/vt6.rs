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

use regex::Regex;

mod encode_argument;
pub use self::encode_argument::*;
mod module_version;
pub use self::module_version::*;

///Parsing and serializing of VT6 messages.
pub mod msg;

lazy_static! {
    //regex matching <identifier>
    static ref IDENT: Regex = Regex::new(r"^[a-zA-Z_][a-zA-Z_-]*$").unwrap();
    //regex matching <scoped-identifier>
    static ref SCOPED_IDENT: Regex = Regex::new(r"^[a-zA-Z_][a-zA-Z_-]*\.[a-zA-Z_][a-zA-Z_-]*$").unwrap();
}

///Checks if the argument is an identifier, as defined by
///[vt6/core1.0, section 1.3](https://vt6.io/std/core/1.0/#section-1-3).
///
///This function can be used to validate module names.
///
///```
///# use vt6::core::is_identifier;
///assert_eq!(is_identifier("core"),          true);
///assert_eq!(is_identifier("core1"),         false);
///assert_eq!(is_identifier("scoped.ident"),  false);
///assert_eq!(is_identifier("what is: this"), false);
///assert_eq!(is_identifier("_what-is-this"), true);
///```
pub fn is_identifier(name: &str) -> bool {
  IDENT.is_match(name)
}

///Checks if the argument is a scoped identifier, as defined by
///[vt6/core1.0, section 1.3](https://vt6.io/std/core/1.0/#section-1-3).
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
///# use vt6::core::is_scoped_identifier;
///assert_eq!(is_scoped_identifier("core"),          None);
///assert_eq!(is_scoped_identifier("core.set"),      Some(("core", "set")));
///assert_eq!(is_scoped_identifier("what is: this"), None);
///assert_eq!(is_scoped_identifier("_what.is-this"), Some(("_what", "is-this")));
///assert_eq!(is_scoped_identifier("want"),          None);
///assert_eq!(is_scoped_identifier("have"),          None);
///assert_eq!(is_scoped_identifier("nope"),          None);
///```
pub fn is_scoped_identifier(name: &str) -> Option<(&str, &str)> {
  if SCOPED_IDENT.is_match(name) {
    Some(split_scoped_identifier(name))
  } else {
    None
  }
}

///Checks if the argument is a message type, as defined by
///[vt6/core1.0, section 1.3](https://vt6.io/std/core/1.0/#section-1-3).
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
///# use vt6::core::is_message_type;
///assert_eq!(is_message_type("core"),          None);
///assert_eq!(is_message_type("core.set"),      Some(("core", "set")));
///assert_eq!(is_message_type("what is: this"), None);
///assert_eq!(is_message_type("_what.is-this"), Some(("_what", "is-this")));
///assert_eq!(is_message_type("want"),          Some(("", "want")));
///assert_eq!(is_message_type("have"),          Some(("", "have")));
///assert_eq!(is_message_type("nope"),          Some(("", "nope")));
///```
pub fn is_message_type(name: &str) -> Option<(&str, &str)> {
  if SCOPED_IDENT.is_match(name) || name == "want" || name == "have" || name == "nope" {
    Some(split_scoped_identifier(name))
  } else {
    None
  }
}

fn split_scoped_identifier(name: &str) -> (&str, &str) {
    match name.find('.') {
        Some(idx) => (&name[0..idx], &name[idx+1..]),
        None      => ("", &name),
    }
}
