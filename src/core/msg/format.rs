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
use core::EncodeArgument;

///A formatter for VT6 messages, as defined in
///[vt6/core1.0, section 2.1](https://vt6.io/std/core/1.0/#section-2-1).
///
///This type is only used for preparing messages for sending. To read received
///messages, use [struct Message](struct.Message.html) instead.
pub struct MessageFormatter<'b> {
    buffer: &'b mut [u8],
    cursor: usize,
    remaining_arguments: usize,
}

impl<'b> MessageFormatter<'b> {
    ///Create a new MessageFormatter. The number of arguments must be given at
    ///this point already because it gets encoded first.
    ///
    ///Most users will prefer `format()` over `new()`, see below.
    pub fn new(buffer: &'b mut [u8], type_name: &str, num_arguments: usize) -> MessageFormatter<'b> {
        //NOTE (majewsky): It's not strictly true that we need the number of
        //arguments at this point; we could also write the argument count in
        //finalize(). It would just involve an extra memmove() to make room for
        //the argument count. Would be nice, but I'm lazy right now. If you
        //would like to make that change, go ahead.

        let len = num_arguments + 1; // + 1 for the message type
        let mut f = MessageFormatter {
            buffer: buffer, cursor: 0, remaining_arguments: len,
        };
        f.add_char(b'{');
        f.encode(&len, len.get_size());
        f.add_char(b'|');
        f.add_argument(type_name);
        f
    }

    ///Adds an argument to the message that is being rendered.
    ///
    ///# Panics
    ///
    ///Panics if more arguments are being added than what has been announced in
    ///`new()` or `format()`.
    pub fn add_argument<T: EncodeArgument + ?Sized>(&mut self, arg: &T) {
        if self.remaining_arguments == 0 {
            panic!("vt6::core::msg::MessageFormatter::add_argument() called more often than expected");
        }
        self.remaining_arguments -= 1;

        let size = arg.get_size();
        self.encode(&size, size.get_size());
        self.add_char(b':');
        self.encode(arg, size);
        self.add_char(b',');
    }

    ///Finalizes the message that is being rendered. On success, returns the
    ///number of bytes that were rendered. In other words: If `Ok(size)` is
    ///returned, the final message can be retrieved from `&buffer[0..size]`,
    ///where `buffer` is the first argument passed to `new()`.
    ///
    ///# Panics
    ///
    ///Panics if `add_argument()` has not been called sufficiently often (as
    ///often as announced in `new()`) before this call.
    pub fn finalize(mut self) -> Result<usize, BufferTooSmallError> {
        if self.remaining_arguments != 0 {
            panic!("vt6::core::msg::MessageFormatter::finalize() called before all arguments were added");
        }
        self.add_char(b'}');
        if self.cursor > self.buffer.len() {
            Err(BufferTooSmallError(self.cursor - self.buffer.len()))
        } else {
            Ok(self.cursor)
        }
    }

    fn add_char(&mut self, c: u8) {
        if self.cursor < self.buffer.len() {
            self.buffer[self.cursor] = c;
        }
        if self.cursor == usize::max_value() {
            panic!("overflow in MessageFormatter.cursor :: usize");
        }
        self.cursor += 1;
    }

    //`size` must be the result of `arg.get_size()`. It is passed into here
    //manually to avoid duplicate get_size() calls.
    fn encode<T: EncodeArgument + ?Sized>(&mut self, arg: &T, size: usize) {
        let (new_cursor, overflow) = self.cursor.overflowing_add(size);
        if overflow {
            panic!("Integer overflow in MessageFormatter.cursor :: usize");
        }

        if new_cursor <= self.buffer.len() {
            arg.encode(&mut self.buffer[self.cursor .. new_cursor]);
        }
        self.cursor = new_cursor;
    }
}
