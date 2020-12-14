/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::common::core::msg::*;

#[test]
fn test_message_parsing() {
    //simple happy cases
    expect_parses(
        b"{4|4:want,4:core,1:1,1:2,}",
        "want",
        &[b"core", b"1", b"2"],
    );
    expect_parses(b"{1|10:sig1.claim,}", "sig1.claim", &[]);

    //argument that is not valid UTF-8
    expect_parses(
        b"{3|9:core1.set,13:example.bytes,3:\xA0+\xC3,}",
        "core1.set",
        &[b"example.bytes", b"\xA0+\xC3"],
    );

    //lower bounds for integers (many of those are errors, but the
    //errors occur *after* the integer parsing, so the integers were parsed
    //correctly)
    expect_parse_fails(b"{0|}", 3, ExpectedMessageType);
    expect_parse_fails(b"{1|0:,}", 6, InvalidMessageType);
    expect_parses(b"{2|4:want,0:,}", "want", &[b""]);

    //upper bounds for integers (the numbers are usize::max_value() - 1 for
    //usize == u16, usize == u32 and usize == u64; so at least one of those
    //should move the cursor backwards in the buffer when wrapping integer
    //arithmetic is used without proper checks)
    expect_parse_incomplete(b"{2|4:want,65535:x,}");
    expect_parse_incomplete(b"{2|4:want,4294967295:x,}");
    expect_parse_incomplete(b"{2|4:want,18446744073709551201:x,}");

    //various UnexpectedEOF scenarios
    expect_parse_incomplete(b"{4|4:want,4:core,1:1,1:2,");
    expect_parse_incomplete(b"{4|4:want,4:core,1:1,1:2");
    expect_parse_incomplete(b"{4|4:want,4:co");
    expect_parse_incomplete(b"{4|4:want,4:");
    expect_parse_incomplete(b"{4|4:want,4");
    expect_parse_incomplete(b"{4|");
    expect_parse_incomplete(b"{4");
    expect_parse_incomplete(b"{");
    expect_parse_fails(b"{4|4:want,4:core,1:1,}", 21, ExpectedDecimalNumber);

    //unexpected characters in various situations
    expect_parse_fails(b"{4|4:want,4:core,1:1,1:2,#", 25, ExpectedMessageCloser);
    expect_parse_fails(b"{4|4:want,4:core,1:1,1:2#", 24, ExpectedStringCloser);
    expect_parse_fails(b"{4|4:want,4:core,1:1,1#", 22, ExpectedStringSigil);
    expect_parse_fails(b"{4#", 2, ExpectedListSigil);
    expect_parse_fails(b"{#", 1, ExpectedDecimalNumber);
    expect_parse_fails(b"#", 0, ExpectedMessageOpener);

    //various other situations
    expect_parse_fails(b"{10000000000000000000000000000", 30, DecimalNumberTooLarge);
    expect_parse_fails(b"{01|10:sig1.claim,}", 3, DecimalNumberHasLeadingZeroes);
    expect_parse_fails(b"{1|010:sig1.claim,}", 6, DecimalNumberHasLeadingZeroes);
}

fn expect_parses(input: &[u8], message_type: &str, args: &[&[u8]]) {
    let (msg, offset) = Message::parse(input).unwrap();
    //`input` should not contain extraneous characters
    assert_eq!(input.len(), offset);
    assert_eq!(format!("{}", msg.parsed_type()), message_type);
    let mut iter = msg.arguments();
    for expected in args {
        assert_eq!(iter.next(), Some(*expected));
    }
    assert_eq!(iter.next(), None);
}

fn expect_parse_fails(input: &[u8], offset: usize, kind: ParseErrorKind) {
    let err = Message::parse(input).unwrap_err();
    assert_eq!(err.kind, kind);
    assert_eq!(err.offset, offset);
}

fn expect_parse_incomplete(input: &[u8]) {
    expect_parse_fails(input, input.len(), UnexpectedEOF);
}

#[test]
fn test_message_fmt_debug_display() {
    let (msg, _) = Message::parse(b"{2|4:want,5:core1,}").unwrap();
    assert_eq!(format!("{}", msg), "(want core1)");
    assert_eq!(
        format!("{:?}", msg),
        r#"Message { parsed_type: Want, arguments: <1 items> }"#
    );

    let (msg, _) = Message::parse(b"{1|10:sig1.claim,}").unwrap();
    assert_eq!(format!("{}", msg), "(sig1.claim)");
    assert_eq!(
        format!("{:?}", msg),
        r#"Message { parsed_type: Scoped(ScopedIdentifier::parse("sig1.claim")), arguments: <0 items> }"#
    );

    let (msg, _) = Message::parse(b"{3|9:core1.set,13:example.bytes,5:\xA0a\"a\xC3,}").unwrap();
    assert_eq!(
        format!("{}", msg),
        r#"(core1.set example.bytes "\xa0a\"a\xc3")"#
    );
    assert_eq!(
        format!("{:?}", msg),
        r#"Message { parsed_type: Scoped(ScopedIdentifier::parse("core1.set")), arguments: <2 items> }"#
    );
}

#[test]
fn test_message_formatting() {
    let mut buf = vec![0; 4096];
    let required_size = make_example_message(&mut buf).unwrap();
    assert_eq!(&buf[0..required_size], b"{2|4:want,5:core1,}" as &[u8]);

    //test that MessageFormatter correctly aborts when encountering too-small
    //buffers of various sizes
    for size in 0..(2 * required_size) {
        let mut buf = vec![0; size];
        let result = make_example_message(&mut buf);
        if size < required_size {
            assert_eq!(result, Err(BufferTooSmallError(required_size - size)));
        } else {
            assert_eq!(result, Ok(required_size));
        }
    }

    //test a message without arguments
    let size = MessageFormatter::new(&mut buf, "sig.claim", 0)
        .finalize()
        .unwrap();
    assert_eq!(&buf[0..size], b"{1|9:sig.claim,}" as &[u8]);

    //test a message with a large number of arguments
    let size = {
        let mut f = MessageFormatter::new(&mut buf, "foo.bar", 250);
        for _ in 0..250 {
            f.add_argument(&0);
        }
        f.finalize().unwrap()
    };
    //prefix "{251|7:foo.bar;" and suffix "}" have 16 bytes in total, and each
    //argument "1:0;" has 4 bytes
    assert_eq!(size, 1016);

    //test a message exceeding the hard 1024-byte limit
    let mut f = MessageFormatter::new(&mut buf, "foo.bar", 500);
    for _ in 0..500 {
        f.add_argument(&0);
    }
    let required_size = 16 + 4 * 500;
    assert_eq!(f.finalize(), Err(BufferTooSmallError(required_size - 1024)));
}

fn make_example_message(buf: &mut [u8]) -> Result<usize, BufferTooSmallError> {
    let mut f = MessageFormatter::new(buf, "want", 1);
    f.add_argument("core1");
    f.finalize()
}
