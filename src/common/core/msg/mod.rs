/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::common::core::{DecodeArgument, MessageType};

mod format;
pub use format::*;
mod traits;
pub use traits::*;

#[cfg(test)]
mod tests;

///An error type that is returned by
///[`MessageFormatter::finalize()`](struct.MessageFormatter.html). It indicates
///that the target buffer was too small to contain the formatted message. The
///contained `usize` value indicates how many bytes could not be written into
///the target buffer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BufferTooSmallError(pub usize);

////////////////////////////////////////////////////////////////////////////////
// struct ParseError

///Enumeration of the kinds of errors that [`Message::parse()`](struct.Message.html) can
///return. See [struct ParseError](struct.ParseError.html) for details.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseErrorKind {
    ///The end of the buffer was encountered before parsing was completed.
    UnexpectedEOF,
    ///Found an unexpected character where there should be a message opener (`{`).
    ExpectedMessageOpener,
    ///Found an unexpected character where there should be a message closer (`}`).
    ExpectedMessageCloser,
    ///Found a non-digit character where there should be a decimal number.
    ExpectedDecimalNumber,
    ///Found a decimal number that is too large to fit in `usize`.
    DecimalNumberTooLarge,
    ///Found a decimal number with leading zeroes, which is not allowed.
    DecimalNumberHasLeadingZeroes,
    ///Found an unexpected character where there should be a list sigil (`|`).
    ExpectedListSigil,
    ///Found an unexpected character where there should be a string sigil (`:`).
    ExpectedStringSigil,
    ///Found an unexpected character where there should be a string closer (`,`).
    ExpectedStringCloser,
    ///Encountered a message without any bytestrings after the list sigil, not even a message type.
    ExpectedMessageType,
    ///Encountered a message whose first bytestring after the list sigil is not a valid message type.
    InvalidMessageType,
}

use self::ParseErrorKind::*;

impl ParseErrorKind {
    ///Returns a human-readable name for this kind.
    pub fn to_str(&self) -> &'static str {
        match *self {
            UnexpectedEOF => "unexpected EOF",
            ExpectedMessageOpener => "expected message opener",
            ExpectedMessageCloser => "expected message closer",
            ExpectedDecimalNumber => "expected decimal number",
            DecimalNumberTooLarge => "decimal number too large",
            DecimalNumberHasLeadingZeroes => "decimal number has leading zeroes",
            ExpectedListSigil => "expected list sigil",
            ExpectedStringSigil => "expected string sigil",
            ExpectedStringCloser => "expected string closer",
            ExpectedMessageType => "expected message type",
            InvalidMessageType => "invalid message type",
        }
    }
}

impl core::fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str(self.to_str())
    }
}

///An error type that is returned by [`Message::parse`](struct.Message.html).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError<'s> {
    ///The original bytestring that was given as input to the message parser.
    pub buffer: &'s [u8],
    ///The position within that bytestring where the error was encountered.
    pub offset: usize,
    ///The kind of parse error that was encountered.
    pub kind: ParseErrorKind,
}

impl<'s> core::fmt::Display for ParseError<'s> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Parse error at offset {}: {}", self.offset, self.kind)
    }
}

#[cfg(any(test, feature = "use_std"))]
impl<'s> std::error::Error for ParseError<'s> {
    fn description(&self) -> &str {
        self.kind.to_str()
    }
}

////////////////////////////////////////////////////////////////////////////////
// struct Cursor

///This holds the state used by the parsing functions in this module.
#[derive(Debug, Clone)]
struct Cursor<'s> {
    ///The byte string whose contents (or parts thereof) shall be parsed. This
    ///is only read, never modified during parsing.
    buffer: &'s [u8],
    ///Points to the next char that will be read (initially 0, up to
    ///`buffer.len()` after all characters have been consumed). This will move
    ///forward during parsing.
    offset: usize,
}

impl<'s> Cursor<'s> {
    ///Constructs a new ParserState pointing to the front of `buffer`.
    fn new(buffer: &'s [u8]) -> Self {
        Cursor { buffer, offset: 0 }
    }

    //assorted helper methods to make the parsing functions shorter
    fn error<T>(&self, kind: ParseErrorKind) -> Result<T, ParseError<'s>> {
        Err(ParseError {
            buffer: self.buffer,
            offset: self.offset,
            kind,
        })
    }
    fn current(&self) -> Result<u8, ParseError<'s>> {
        if self.offset < self.buffer.len() {
            Ok(self.buffer[self.offset])
        } else {
            self.error(UnexpectedEOF)
        }
    }

    fn consume_char(&mut self, c: u8, kind: ParseErrorKind) -> Result<(), ParseError<'s>> {
        if self.current()? != c {
            return self.error(kind);
        }
        self.offset += 1;
        Ok(())
    }

    fn consume_message_opener(&mut self) -> Result<(), ParseError<'s>> {
        self.consume_char(b'{', ExpectedMessageOpener)
    }

    fn consume_message_closer(&mut self) -> Result<(), ParseError<'s>> {
        self.consume_char(b'}', ExpectedMessageCloser)
    }

    fn consume_decimal(&mut self) -> Result<usize, ParseError<'s>> {
        //exit early if cursor is at EOF already
        self.current()?;

        //find end of string of digits
        let start_cursor = self.offset;
        loop {
            match self.current() {
                Ok(c) if isnum(c) => {
                    self.offset += 1;
                }
                _ => break, //EOF or some non-digit character
            }
        }

        //did we find any digits?
        if start_cursor == self.offset {
            self.error(ExpectedDecimalNumber)
        } else {
            let digit_str = unsafe {
                //this is safe because we verified above that this range of
                //bytes matches /[0-9]*/ and thus is in ASCII
                core::str::from_utf8_unchecked(&self.buffer[start_cursor..self.offset])
            };

            //check that there are no leading zeroes
            if digit_str.len() > 1 && digit_str.as_bytes()[0] == b'0' {
                return self.error(DecimalNumberHasLeadingZeroes);
            }

            match digit_str.parse() {
                Ok(val) => Ok(val),
                Err(_) => self.error(DecimalNumberTooLarge),
            }
        }
    }

    fn consume_list_sigil(&mut self) -> Result<(), ParseError<'s>> {
        self.consume_char(b'|', ExpectedListSigil)
    }

    fn consume_string_sigil(&mut self) -> Result<(), ParseError<'s>> {
        self.consume_char(b':', ExpectedStringSigil)
    }

    fn consume_string_contents(&mut self, count: usize) -> Result<&'s [u8], ParseError<'s>> {
        let new_offset = self.offset.wrapping_add(count);
        //check for integer overflow, buffer overflow
        if new_offset < self.offset || new_offset > self.buffer.len() {
            self.offset = self.buffer.len();
            self.error(UnexpectedEOF)
        } else {
            let result = &self.buffer[self.offset..new_offset];
            self.offset = new_offset;
            Ok(result)
        }
    }

    fn consume_string_closer(&mut self) -> Result<(), ParseError<'s>> {
        self.consume_char(b',', ExpectedStringCloser)
    }
}

fn isnum(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

////////////////////////////////////////////////////////////////////////////////
// struct MessageIterator

///An iterator over the list of bytestrings in a message. Messages are defined
///in [vt6/foundation, section 3.1](https://vt6.io/std/foundation/#section-3-1).
///
///The lifetime argument is the lifetime of the buffer from which the
///message containing this list was parsed.
#[derive(Clone)]
pub struct MessageIterator<'s> {
    cursor: Cursor<'s>,
    remaining_items: usize,
}

impl<'s> MessageIterator<'s> {
    fn make(cursor: Cursor<'s>, items: usize) -> Self {
        MessageIterator {
            cursor,
            remaining_items: items,
        }
    }

    //Implementation notes: There are two distinct phases in message parsing.
    //
    //* Validation phase: During Message::parse(), the initial MessageIterator for
    //  the message's top-level list is constructed, and consume_and_validate()
    //  is called on a clone of it. This is required because Message::parse()
    //  needs a cursor pointing to the end of the list to parse the message
    //  closer (`}`).
    //
    //* Usage phase: When the user receives the MessageIterator for the message's
    //  arguments, the validation phase has already proven that the message
    //  parses successfully. We don't retain an AST because we want to avoid
    //  heap allocations, but we know that when the user iterates through the
    //  message's arguments, no parse errors can occur. The public
    //  MessageIterator::next() method can therefore safely ignore parse errors.
    fn next_or_error(&mut self) -> Result<Option<&'s [u8]>, ParseError<'s>> {
        if self.remaining_items == 0 {
            return Ok(None);
        }
        self.remaining_items -= 1;

        //self.cursor is at the start of the bytestring, i.e. on its length
        let count = self.cursor.consume_decimal()?;
        self.cursor.consume_string_sigil()?;
        let s = self.cursor.consume_string_contents(count)?;
        self.cursor.consume_string_closer()?;
        Ok(Some(s))
    }

    fn consume_and_validate(mut self) -> Result<Cursor<'s>, ParseError<'s>> {
        loop {
            if self.next_or_error()?.is_none() {
                return Ok(self.cursor);
            }
        }
    }

    //This is `pub(crate)` only for now because I want to gain experience with this API first.
    //When it goes `pub`, it will probably be on an `IteratorExt`-like trait.
    pub(crate) fn exactly1<A>(mut self) -> Option<A>
    where
        A: DecodeArgument<'s>,
    {
        if self.remaining_items != 1 {
            return None;
        }
        Some(A::decode_argument(self.next()?)?)
    }

    //This is `pub(crate)` only for now because I want to gain experience with this API first.
    //When it goes `pub`, it will probably be on an `IteratorExt`-like trait.
    pub(crate) fn exactly2<A, B>(mut self) -> Option<(A, B)>
    where
        A: DecodeArgument<'s>,
        B: DecodeArgument<'s>,
    {
        if self.remaining_items != 2 {
            return None;
        }
        let a = A::decode_argument(self.next()?)?;
        let b = B::decode_argument(self.next()?)?;
        Some((a, b))
    }

    //This is `pub(crate)` only for now because I want to gain experience with this API first.
    //When it goes `pub`, it will probably be on an `IteratorExt`-like trait.
    pub(crate) fn exactly4<A, B, C, D>(mut self) -> Option<(A, B, C, D)>
    where
        A: DecodeArgument<'s>,
        B: DecodeArgument<'s>,
        C: DecodeArgument<'s>,
        D: DecodeArgument<'s>,
    {
        if self.remaining_items != 4 {
            return None;
        }
        let a = A::decode_argument(self.next()?)?;
        let b = B::decode_argument(self.next()?)?;
        let c = C::decode_argument(self.next()?)?;
        let d = D::decode_argument(self.next()?)?;
        Some((a, b, c, d))
    }
}

impl<'s> Iterator for MessageIterator<'s> {
    type Item = &'s [u8];

    fn next(&mut self) -> Option<Self::Item> {
        self.next_or_error().unwrap_or(None)
    }
}

impl<'s> core::iter::ExactSizeIterator for MessageIterator<'s> {
    fn len(&self) -> usize {
        self.remaining_items
    }
}

impl<'s> core::fmt::Debug for MessageIterator<'s> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "<{} items>", self.len())
    }
}

////////////////////////////////////////////////////////////////////////////////
// struct Message

///A VT6 message, as defined in
///[vt6/foundation, section 3.1](https://vt6.io/std/foundation/#section-3-1).
///
///The lifetime argument is the lifetime of the bytestring from which the
///message containing this list was parsed.
///
///This type is only used for reading received messages. To build a message for
///sending, use [struct MessageFormatter](struct.MessageFormatter.html) instead.
///
///The implementation of Display prints the human-readable representation as defined by
///[vt6/foundation, section 3.1.3](https://vt6.io/std/foundation/#section-3-1-3).
///
///```
///# use vt6::common::core::msg::*;
///let (msg, _) = Message::parse(b"{3|9:core1.set,13:example.title,11:hello world,}").unwrap();
///assert_eq!(format!("{}", msg), r#"(core1.set example.title "hello world")"#);
///```
#[derive(Debug)]
pub struct Message<'s> {
    parsed_type: MessageType<'s>,
    arguments: MessageIterator<'s>,
}

impl<'s> Message<'s> {
    ///Parses a message from `buffer`. The first byte, `buffer[0]`, must be the
    ///message opener (`{`). The success value is a pair of the message and the
    ///number of bytes it makes up. Callers can use the byte count to discard
    ///the message from the buffer after it has been processed. The byte count
    ///includes the message opener and closer, so `buffer[byte_count - 1] ==
    ///b'}'`.
    pub fn parse(buffer: &'s [u8]) -> Result<(Message<'s>, usize), ParseError<'s>> {
        let mut cursor = Cursor::new(buffer);
        cursor.consume_message_opener()?;

        let count_items = cursor.consume_decimal()?;

        cursor.consume_list_sigil()?;
        let mut iter = MessageIterator::make(cursor, count_items);

        //extract the first item to check if it's a message type
        let msg_type = match iter.next_or_error()? {
            None => return iter.cursor.error(ExpectedMessageType),
            Some(s) => match core::str::from_utf8(s).ok().and_then(MessageType::parse) {
                Some(mt) => mt,
                None => return iter.cursor.error(InvalidMessageType),
            },
        };

        //validate the rest of the argument list
        cursor = iter.clone().consume_and_validate()?;
        cursor.consume_message_closer()?;

        let msg = Message {
            parsed_type: msg_type,
            arguments: iter,
        };
        Ok((msg, cursor.offset))
    }

    ///Returns the parsed message type.
    ///
    ///```
    ///# use vt6::common::core::msg::Message;
    ///let (msg, _) = Message::parse(b"{2|9:core1.sub,8:foo3.bar,}").unwrap();
    ///    // (core1.sub foo3.bar)
    ///assert_eq!(msg.parsed_type().as_str(), "core1.sub");
    ///
    ///let (msg, _) = Message::parse(b"{2|4:want,5:core1,}").unwrap();
    ///    // (want core1)
    ///assert_eq!(msg.parsed_type().as_str(), "want");
    ///```
    pub fn parsed_type(&'s self) -> MessageType<'s> {
        self.parsed_type.clone()
    }

    ///Returns an iterator over the arguments of this message. (This does not
    ///include the message type name.)
    ///
    ///```
    ///# use vt6::common::core::msg::Message;
    ///let (msg, _) = Message::parse(b"{3|9:core1.set,13:example.title,11:hello world,}").unwrap();
    ///    // (core1.set example.title "hello world")
    ///let mut iter = msg.arguments();
    ///assert_eq!(iter.next(), Some(b"example.title" as &[u8]));
    ///assert_eq!(iter.next(), Some(b"hello world" as &[u8]));
    ///assert_eq!(iter.next(), None);
    ///```
    pub fn arguments(&self) -> MessageIterator<'s> {
        self.arguments.clone()
    }
}

impl<'s> core::fmt::Display for Message<'s> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "({}", self.parsed_type)?;
        for arg in self.arguments.clone() {
            let escaped = arg.is_empty() || arg.iter().any(|&x| char_needs_escaping(x));
            f.write_str(if escaped { " \"" } else { " " })?;
            for byte in arg.iter().flat_map(|&b| core::ascii::escape_default(b)) {
                (byte as char).fmt(f)?;
            }
            if escaped {
                f.write_str("\"")?;
            }
        }
        f.write_str(")")
    }
}

fn char_needs_escaping(ch: u8) -> bool {
    //vt6/foundation, sect. 3.1.3:
    //> Bytestrings whose value matches the regular expression `^[A-Za-z0-9._-]*$` are represented
    //> directly by their value.
    !((ch >= b'A' && ch <= b'Z')
        || (ch >= b'a' && ch <= b'z')
        || (ch >= b'0' && ch <= b'9')
        || ch == b'.'
        || ch == b'_'
        || ch == b'-')
}
