/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use super::*;

//TODO Once <https://github.com/rust-lang/rust/issues/51999> stabilizes, make all the parse() functions const to enable 'static identifier values.

////////////////////////////////////////////////////////////////////////////////
// ClientID

///A client ID, as defined by
///[vt6/foundation, section 2.6](https://vt6.io/std/foundation/#section-2-6).
///
///Because of the associated lifetime, this type does not implement DecodeArgument. Use
///ClientID::parse() instead. Instances of this type can only be created through a successful
///ClientID::parse().
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClientID<'a>(&'a str);

//TODO impl Deref?

impl<'a> core::fmt::Debug for ClientID<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "ClientID::parse({:?})", self.0)
    }
}

impl<'a> core::fmt::Display for ClientID<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl<'a> EncodedArgument for ClientID<'a> {
    fn encoded(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl<'a> ClientID<'a> {
    ///Converts the given input string into an ClientID instance. Returns None if the input is
    ///not a valid identifier.
    ///
    ///```
    ///# use vt6::common::core::*;
    ///assert!(ClientID::parse("abc").is_some());
    ///assert!(ClientID::parse("123").is_some());
    ///assert!(ClientID::parse("A1B2").is_some());
    ///assert!(ClientID::parse("").is_none());
    ///assert!(ClientID::parse("a-b").is_none());
    ///assert!(ClientID::parse("core1.set").is_none());
    ///```
    pub fn parse(input: &'a str) -> Option<Self> {
        if input.is_empty() {
            return None;
        }
        if input.chars().all(is_client_id_char) {
            Some(ClientID(input))
        } else {
            None
        }
    }

    ///Returns the string value of this identifier. This is the same string that was originally
    ///passed into parse().
    pub fn as_str(&'_ self) -> &'a str {
        self.0
    }
}

fn is_client_id_char(ch: char) -> bool {
    (ch >= 'A' && ch <= 'Z') || (ch >= 'a' && ch <= 'z') || (ch >= '0' && ch <= '9')
}

////////////////////////////////////////////////////////////////////////////////
// Identifier

///An identifier, as defined by
///[vt6/foundation, section 2.1](https://vt6.io/std/foundation/#section-2-1).
///
///Because of the associated lifetime, this type does not implement DecodeArgument. Use
///Identifier::parse() instead. Instances of this type can only be created through a successful
///Identifier::parse().
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Identifier<'a>(&'a str);

//TODO impl Deref?

impl<'a> core::fmt::Debug for Identifier<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Identifier::parse({:?})", self.0)
    }
}

impl<'a> core::fmt::Display for Identifier<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl<'a> EncodedArgument for Identifier<'a> {
    fn encoded(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl<'a> Identifier<'a> {
    ///Converts the given input string into an Identifier instance. Returns None if the input is
    ///not a valid identifier.
    pub fn parse(input: &'a str) -> Option<Self> {
        match parse_ident_or_module_ident(input) {
            Some((name, None)) => Some(name),
            _ => None,
        }
    }

    ///Returns the string value of this identifier. This is the same string that was originally
    ///passed into parse().
    pub fn as_str(&'_ self) -> &'a str {
        self.0
    }
}

fn parse_ident_or_module_ident(input: &str) -> Option<(Identifier<'_>, Option<u16>)> {
    // returns one of:
    //     None                 -> neither identifier nor module identifier
    //     Some(str, None)      -> identifier
    //     Some(str, Some(num)) -> module identifier
    let mut iter = input.char_indices();

    //consume the leading character of the identifier
    match iter.next() {
        Some((_, ch)) if is_ident_leader(ch) => {}
        _ => return None,
    };

    //find the end of the identifier
    let start_idx_of_version = loop {
        match iter.next() {
            Some((_, ch)) if is_ident_char(ch) => continue,
            //found end of string while reading identifier -> no version follows -> just an identifier
            None => return Some((Identifier(input), None)),
            //found first digit of version -> stop here
            Some((idx, ch)) if is_digit(ch) && ch != '0' => break idx,
            //found unexpected character
            Some(_) => return None,
        }
    };

    //the rest of the input must be the major version
    let (name_str, version_str) = input.split_at(start_idx_of_version);
    let version = u16::decode(version_str.as_bytes())?;
    if version == 0 {
        return None;
    }
    Some((Identifier(name_str), Some(version)))
}

fn is_ident_leader(ch: char) -> bool {
    (ch >= 'A' && ch <= 'Z') || (ch >= 'a' && ch <= 'z') || ch == '_'
}

fn is_ident_char(ch: char) -> bool {
    (ch >= 'A' && ch <= 'Z') || (ch >= 'a' && ch <= 'z') || ch == '_' || ch == '-'
}

fn is_digit(ch: char) -> bool {
    ch >= '0' && ch <= '9'
}

////////////////////////////////////////////////////////////////////////////////
// ModuleIdentifier

///A module identifier is the first half (i.e., everything before the dot) of a scoped identifier
///as defined by [vt6/foundation, section 2.4](https://vt6.io/std/foundation/#section-2-4). For
///example, in the scoped identifier `core1.set`, the module identifier is `core1`.
///
///Because of the associated lifetime, this type does not implement DecodeArgument. Use
///ModuleIdentifier::parse() instead.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModuleIdentifier<'a> {
    source: &'a str,
    name: Identifier<'a>,
    major_version: u16,
}

impl<'a> core::fmt::Debug for ModuleIdentifier<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "ModuleIdentifier::parse({:?})", self.source)
    }
}

impl<'a> core::fmt::Display for ModuleIdentifier<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.source.fmt(f)
    }
}

impl<'a> EncodedArgument for ModuleIdentifier<'a> {
    fn encoded(&self) -> &[u8] {
        self.source.as_bytes()
    }
}

impl<'a> ModuleIdentifier<'a> {
    ///Parses the given input string into a ModuleIdentifier instance. Returns None if the input is
    ///not a valid module identifier.
    pub fn parse(input: &'a str) -> Option<Self> {
        match parse_ident_or_module_ident(input) {
            Some((name, Some(version))) => Some(ModuleIdentifier {
                source: input,
                name,
                major_version: version,
            }),
            _ => None,
        }
    }

    ///Returns the string representation of this module identifier. This is the same string that
    ///was originally passed into parse().
    pub fn as_str(&self) -> &str {
        self.source
    }

    ///Returns the name of the identified module, without the major version.
    ///
    ///```
    ///# use vt6::common::core::*;
    ///let module = ModuleIdentifier::parse("core3").unwrap();
    ///assert_eq!(module.name().as_str(), "core");
    ///```
    pub fn name(&'a self) -> Identifier<'a> {
        self.name
    }

    ///Returns the major version of the identified module.
    ///
    ///```
    ///# use vt6::common::core::*;
    ///let module = ModuleIdentifier::parse("core3").unwrap();
    ///assert_eq!(module.major_version(), 3);
    ///```
    pub fn major_version(&'a self) -> u16 {
        self.major_version
    }
}

////////////////////////////////////////////////////////////////////////////////
// ModuleVersion

///A module version is a module name with its full version, e.g. `term2.3`, as it appears in the
///argument of a positive `have` message, as defined by
///[vt6/foundation, section 4.2](https://vt6.io/std/foundation/#section-4-2).
///
///Because of the associated lifetime, this type does not implement DecodeArgument. Use
///ModuleIdentifier::parse() instead.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModuleVersion<'a> {
    source: &'a str,
    name: Identifier<'a>,
    major_version: u16,
    minor_version: u16,
}

impl<'a> core::fmt::Debug for ModuleVersion<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "ModuleVersion::parse({:?})", self.source)
    }
}

impl<'a> core::fmt::Display for ModuleVersion<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.source.fmt(f)
    }
}

impl<'a> EncodedArgument for ModuleVersion<'a> {
    fn encoded(&self) -> &[u8] {
        self.source.as_bytes()
    }
}

impl<'a> ModuleVersion<'a> {
    ///Parses the given input string into a ModuleVersion instance. Returns None if the input is
    ///not a valid module identifier followed by a full version.
    pub fn parse(input: &'a str) -> Option<Self> {
        let dot_idx = input.find('.')?;
        let (left, right) = (&input[0..dot_idx], &input[dot_idx + 1..]);
        let ident = ModuleIdentifier::parse(left)?;
        let minor = u16::decode(right.as_bytes())?;
        Some(ModuleVersion {
            source: input,
            name: ident.name,
            major_version: ident.major_version,
            minor_version: minor,
        })
    }

    ///Returns the string representation of this module identifier. This is the same string that
    ///was originally passed into parse().
    pub fn as_str(&self) -> &str {
        self.source
    }

    ///Returns the name of the identified module, without the major version.
    ///
    ///```
    ///# use vt6::common::core::*;
    ///let module = ModuleVersion::parse("core3.2").unwrap();
    ///assert_eq!(module.name().as_str(), "core");
    ///```
    pub fn name(&'a self) -> Identifier<'a> {
        self.name
    }

    ///Returns the major version of the identified module.
    ///
    ///```
    ///# use vt6::common::core::*;
    ///let module = ModuleVersion::parse("core3.2").unwrap();
    ///assert_eq!(module.major_version(), 3);
    ///```
    pub fn major_version(&'a self) -> u16 {
        self.major_version
    }

    ///Returns the minor version of the identified module.
    ///
    ///```
    ///# use vt6::common::core::*;
    ///let module = ModuleVersion::parse("core3.2").unwrap();
    ///assert_eq!(module.minor_version(), 2);
    ///```
    pub fn minor_version(&'a self) -> u16 {
        self.minor_version
    }
}

////////////////////////////////////////////////////////////////////////////////
// ScopedIdentifier

///A scoped identifier, as defined by
///[vt6/foundation, section 2.4](https://vt6.io/std/foundation/#section-2-4).
///
///Because of the associated lifetime, this type does not implement DecodeArgument. Use
///ScopedIdentifier::parse() instead.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScopedIdentifier<'a> {
    source: &'a str,
    module: ModuleIdentifier<'a>,
    member: Identifier<'a>,
}

impl<'a> core::fmt::Debug for ScopedIdentifier<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "ScopedIdentifier::parse({:?})", self.source)
    }
}

impl<'a> core::fmt::Display for ScopedIdentifier<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.source.fmt(f)
    }
}

impl<'a> EncodedArgument for ScopedIdentifier<'a> {
    fn encoded(&self) -> &[u8] {
        self.source.as_bytes()
    }
}

impl<'a> ScopedIdentifier<'a> {
    ///Parses the given input string into a ScopedIdentifier instance. Returns None if the input is
    ///not a valid scoped identifier.
    pub fn parse(input: &'a str) -> Option<Self> {
        let dot_idx = input.find('.')?;
        let (left, right) = (&input[0..dot_idx], &input[dot_idx + 1..]);
        Some(ScopedIdentifier {
            source: input,
            module: ModuleIdentifier::parse(left)?,
            member: Identifier::parse(right)?,
        })
    }

    ///Returns the string representation of this module identifier. This is the same string that
    ///was originally passed into parse().
    pub fn as_str(&self) -> &str {
        self.source
    }

    ///Returns the first half of this scoped identifier which contains the module name and major
    ///version.
    ///
    ///```
    ///# use vt6::common::core::*;
    ///let ident = ScopedIdentifier::parse("core1.set").unwrap();
    ///assert_eq!(ident.module().as_str(), "core1");
    ///```
    pub fn module(&'a self) -> ModuleIdentifier<'a> {
        self.module.clone()
    }

    ///Returns the second half of this scoped identifier which identifies the message type or
    ///property name within the module.
    ///
    ///```
    ///# use vt6::common::core::*;
    ///let ident = ScopedIdentifier::parse("core1.set").unwrap();
    ///assert_eq!(ident.member().as_str(), "set");
    ///```
    pub fn member(&'a self) -> Identifier<'a> {
        self.member
    }
}

////////////////////////////////////////////////////////////////////////////////
// MessageType

///A message type, as defined by
///[vt6/foundation, section 2.4](https://vt6.io/std/foundation/#section-2-4).
///
///Because of the associated lifetime, this type does not implement DecodeArgument. Use
///MessageType::parse() instead.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MessageType<'a> {
    Init,
    Want,
    Have,
    Nope,
    Scoped(ScopedIdentifier<'a>),
}
use self::MessageType::*;

impl<'a> core::fmt::Display for MessageType<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl<'a> EncodedArgument for MessageType<'a> {
    fn encoded(&self) -> &[u8] {
        self.as_str().as_bytes()
    }
}

impl<'a> MessageType<'a> {
    ///Parses the given input string into a MessageType instance. Returns None if the input is
    ///not a valid message type.
    pub fn parse(input: &'a str) -> Option<Self> {
        match input {
            "init" => Some(Init),
            "want" => Some(Want),
            "have" => Some(Have),
            "nope" => Some(Nope),
            _ => Some(Scoped(ScopedIdentifier::parse(input)?)),
        }
    }

    ///Returns the string representation of this module identifier. This is the same string that
    ///was originally passed into parse().
    pub fn as_str(&self) -> &str {
        match *self {
            Init => "init",
            Want => "want",
            Have => "have",
            Nope => "nope",
            Scoped(ref s) => s.as_str(),
        }
    }
}

#[cfg(test)]
mod tests {

    // NOTE: This does not really cover ClientID. When I added this type, this test already had enough
    // combinations that I didn't bother disambiguating even more check_*() functions. Instead, I
    // cover the simple cases (most importantly, the happy path) for ClientID::parse() in a doc
    // test above.

    use super::*;

    fn check_is_unrecognizable(input: &str) {
        assert_eq!(Identifier::parse(input), None);
        assert_eq!(ModuleIdentifier::parse(input), None);
        assert_eq!(ModuleVersion::parse(input), None);
        assert_eq!(ScopedIdentifier::parse(input), None);
        assert_eq!(MessageType::parse(input), None);
    }

    fn check_is_identifier(input: &str) {
        match Identifier::parse(input) {
            Some(ident) => assert_eq!(input, format!("{}", ident)),
            None => panic!("input {} was not recognized as identifier", input),
        };
        //plain identifiers can not be any other sort of identifier from this module
        //(except for eternal message types, for which we use check_is_eternal_message_type
        //instead)
        assert_eq!(ModuleIdentifier::parse(input), None);
        assert_eq!(ModuleVersion::parse(input), None);
        assert_eq!(ScopedIdentifier::parse(input), None);
        assert_eq!(MessageType::parse(input), None);
    }

    fn check_is_module_identifier(input: &str) {
        match ModuleIdentifier::parse(input) {
            Some(ident) => assert_eq!(input, format!("{}", ident)),
            None => panic!("input {} was not recognized as module identifier", input),
        };
        //module identifiers can not be any other sort of identifier from this module
        //(except maybe ClientID)
        assert_eq!(Identifier::parse(input), None);
        assert_eq!(ModuleVersion::parse(input), None);
        assert_eq!(ScopedIdentifier::parse(input), None);
        assert_eq!(MessageType::parse(input), None);
    }

    fn check_is_module_version(input: &str) {
        match ModuleVersion::parse(input) {
            Some(ident) => assert_eq!(input, format!("{}", ident)),
            None => panic!("input {} was not recognized as module version", input),
        };
        //module version can not be any other sort of identifier from this module
        assert_eq!(ClientID::parse(input), None);
        assert_eq!(Identifier::parse(input), None);
        assert_eq!(ModuleIdentifier::parse(input), None);
        assert_eq!(ScopedIdentifier::parse(input), None);
        assert_eq!(MessageType::parse(input), None);
    }

    fn check_is_scoped_identifier(input: &str) {
        match ScopedIdentifier::parse(input) {
            Some(ident) => assert_eq!(input, format!("{}", ident)),
            None => panic!("input {} was not recognized as scoped identifier", input),
        };
        //every scoped identifier is also a valid message type
        match MessageType::parse(input) {
            Some(Scoped(ident)) => assert_eq!(input, format!("{}", ident)),
            Some(msg_type) => panic!("input {} was misclassified as {:?}", input, msg_type),
            None => panic!("input {} was not recognized as message type", input),
        };
        //scoped identifiers are never plain identifiers or module identifiers
        assert_eq!(ClientID::parse(input), None);
        assert_eq!(Identifier::parse(input), None);
        assert_eq!(ModuleIdentifier::parse(input), None);
        assert_eq!(ModuleVersion::parse(input), None);
    }

    fn check_is_eternal_message_type(input: &str) {
        match MessageType::parse(input) {
            Some(ident) => assert_eq!(input, format!("{}", ident)),
            None => panic!("input {} was not recognized as message type", input),
        };
        //eternal message types are also valid client IDs and plain identifiers...
        match ClientID::parse(input) {
            Some(ident) => assert_eq!(input, format!("{}", ident)),
            None => panic!("input {} was not recognized as client ID", input),
        };
        match Identifier::parse(input) {
            Some(ident) => assert_eq!(input, format!("{}", ident)),
            None => panic!("input {} was not recognized as identifier", input),
        };
        //...but not any other sort of identifiers
        assert_eq!(ModuleIdentifier::parse(input), None);
        assert_eq!(ScopedIdentifier::parse(input), None);
        assert_eq!(ModuleVersion::parse(input), None);
    }

    #[test]
    fn test_parsing() {
        check_is_identifier("foo");
        check_is_identifier("foo_foo");
        check_is_identifier("foo-foo");
        check_is_identifier("_foo");
        check_is_identifier("_foo_foo");
        check_is_identifier("_foo-foo");
        //identifiers can never start with a dash
        check_is_unrecognizable("-foo");
        check_is_unrecognizable("-foo_foo");
        check_is_unrecognizable("-foo-foo");
        //other characters not allowed in an identifier
        check_is_unrecognizable("f0o");
        check_is_unrecognizable("f0o.bar");
        check_is_unrecognizable("fo=o");
        check_is_unrecognizable("what is this");

        check_is_module_identifier("foo1");
        check_is_module_identifier("foo_foo23");
        check_is_module_identifier("foo-foo4");
        check_is_module_identifier("_foo5");
        check_is_module_identifier("_foo_foo5");
        check_is_module_identifier("_foo-foo607");
        //major version 0 is not allowed
        check_is_unrecognizable("foo0");
        check_is_unrecognizable("foo_foo0");
        check_is_unrecognizable("foo-foo0");
        check_is_unrecognizable("_foo0");
        check_is_unrecognizable("_foo_foo0");
        check_is_unrecognizable("_foo-foo0");
        //other characters not allowed in an identifier
        check_is_unrecognizable("fo.1");
        check_is_unrecognizable("fo=1");
        check_is_unrecognizable("f.o1");
        check_is_unrecognizable("f=o1");

        check_is_module_version("foo1.0");
        check_is_module_version("foo_foo23.2");
        check_is_module_version("foo-foo4.0");
        check_is_module_version("_foo5.63");
        check_is_module_version("_foo_foo5.508");
        check_is_module_version("_foo-foo607.0");
        //require minimal encoding of 0
        check_is_unrecognizable("foo1.00");
        check_is_unrecognizable("foo_foo23.000");
        check_is_unrecognizable("foo-foo4.00");
        check_is_unrecognizable("_foo5.000");
        check_is_unrecognizable("_foo_foo5.00");
        check_is_unrecognizable("_foo-foo607.000");
        //major version 0 is not allowed
        check_is_unrecognizable("foo0.0");
        check_is_unrecognizable("foo_foo0.0");
        check_is_unrecognizable("foo-foo0.0");
        check_is_unrecognizable("_foo0.0");
        check_is_unrecognizable("_foo_foo0.0");
        check_is_unrecognizable("_foo-foo0.0");
        //other characters not allowed in an identifier
        check_is_unrecognizable("fo.1.0");
        check_is_unrecognizable("fo=1.0");
        check_is_unrecognizable("f.o1.0");
        check_is_unrecognizable("f=o1.0");
        check_is_unrecognizable("foo1.0a");
        check_is_unrecognizable("foo1.-0");
        check_is_unrecognizable("foo1.0-1");
        check_is_unrecognizable("foo1.0-");

        //scoped identifiers with different types of module identifiers
        check_is_scoped_identifier("foo1.bar");
        check_is_scoped_identifier("foo_foo23.bar");
        check_is_scoped_identifier("foo-foo4.bar");
        check_is_scoped_identifier("_foo5.bar");
        check_is_scoped_identifier("_foo_foo5.bar");
        check_is_scoped_identifier("_foo-foo607.bar");
        //scoped identifiers with different types of member identifiers
        check_is_scoped_identifier("foo23.bar");
        check_is_scoped_identifier("foo23.bar_bar");
        check_is_scoped_identifier("foo23.bar-bar");
        check_is_scoped_identifier("foo23._bar");
        check_is_scoped_identifier("foo23._bar_bar");
        check_is_scoped_identifier("foo23._bar-bar");
        //missing parts
        check_is_unrecognizable(".");
        check_is_unrecognizable("foo.");
        check_is_unrecognizable("foo1.");
        check_is_unrecognizable(".foo");

        //last but not least
        check_is_eternal_message_type("init");
        check_is_eternal_message_type("want");
        check_is_eternal_message_type("have");
        check_is_eternal_message_type("nope");
    }
}
