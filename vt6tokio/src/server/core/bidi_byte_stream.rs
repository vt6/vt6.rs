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

use std;

use tokio::prelude::*;
use tokio::io::{ReadHalf, WriteHalf};
use tokio_uds::UnixStream;
use vt6::server as vt6s;
use vt6::common::core::msg;

use server::core::Connection;

pub(crate) struct BidiByteStream<C: Connection> {
    pub conn: C,
    recv: RecvBuffer<UnixStream>,
    send: SendBuffer<UnixStream>,
}

impl<C: Connection> Drop for BidiByteStream<C> {
    fn drop(&mut self) {
        info!("connection {}: terminated", self.conn.id());
    }
}

impl<C: Connection> BidiByteStream<C> {
    pub fn new(conn: C, stream: UnixStream) -> Self {
        trace!("connection {}: accepted", conn.id());
        let (reader, writer) = stream.split();

        let max_client_message_length = conn.max_client_message_length();
        let max_server_message_length = conn.max_server_message_length();

        BidiByteStream {
            conn: conn,
            recv: RecvBuffer::new(reader, max_client_message_length),
            send: SendBuffer::new(writer, max_server_message_length),
        }
    }

    pub fn poll<H: vt6s::EarlyHandler<C>>(&mut self, handler: &H) -> Poll<(), std::io::Error> {
        let recv_result = self.poll_recv(handler);

        if let Ok(Async::NotReady) = recv_result {
            //when self.recv.poll() returned "not ready", make sure that the
            //task also knows about our interest in writing to self.writer
            if self.send.can_write() {
                //note that this never returns Async::Ready
                return self.send.poll_write();
            }
        }
        recv_result
    }

    pub fn append_to_send_buffer(&mut self, bytes: &[u8]) {
       self.send.stdin.extend(bytes);
    }

    fn poll_recv<H: vt6s::EarlyHandler<C>>(&mut self, handler: &H) -> Poll<(), std::io::Error> {
        use vt6::server::core::StreamMode::*;
        match self.conn.stream_state().mode {
            Stdio   => self.poll_recv_stdio(),
            Message => {
                match try_ready!(self.poll_recv_messages(handler)) {
                    StreamModeChanged(false) => Ok(Async::Ready(())),
                    //restart call to branch into a different poll_recv_*()
                    //depending on the new stream mode
                    StreamModeChanged(true)  => self.poll_recv(handler),
                }
            },
        }
    }

    fn poll_recv_messages<H: vt6s::EarlyHandler<C>>(&mut self, handler: &H)
        -> Poll<StreamModeChanged, std::io::Error>
    {
        //spell it out to the borrow checker that we're *not* borrowing `self`
        //into the closure below
        let self_id = self.conn.id();
        let self_send = &mut self.send;
        let self_conn = &mut self.conn;

        self.recv.poll_messages(self_id, |msg| {
            trace!("message received on connection {}: {}", self_id, msg);

            //if the send buffer is getting full, try to empty it before
            //handling the message (we want to guarantee at least 1024 bytes in
            //the send buffer before trying to handle the message)
            while self_send.message_buffer.unfilled_len() < 1024 {
                try_ready!(self_send.poll_write());
            }

            //try to handle this message
            let result = handler.handle(msg, self_conn, self_send.message_buffer.unfilled_mut());
            match result {
                Some(bytes_written) => {
                    self_send.message_buffer.fill += bytes_written;
                    //TODO validate that self_send.fill < self_send.buf.len()
                },
                None => {
                    //message was either invalid or the send buffer was exceeded
                    //when trying to send a reply -> answer with (nope) instead
                    let result = msg::MessageFormatter::new(
                        self_send.message_buffer.unfilled_mut(),
                        "nope", 0,
                    ).finalize();
                    if let Ok(bytes_written) = result { // TODO otherwise log error
                        self_send.message_buffer.fill += bytes_written;
                    }
                },
            };

            use vt6::server::core::StreamMode::Message;
            let stream_mode_changed = self_conn.stream_state().mode == Message;
            Ok(Async::Ready(StreamModeChanged(stream_mode_changed)))
        })
    }

    fn poll_recv_stdio(&mut self) -> Poll<(), std::io::Error> {
        let mut stdout = Vec::new();
        let poll_result = self.recv.poll_stdout_into(&mut stdout);

        //even if poll returns error or EOF, we always want to process
        //the stdout that was received before the error or EOF
        if stdout.len() > 0 {
            self.conn.handle_standard_output(&stdout);
        }

        poll_result
    }
}

////////////////////////////////////////////////////////////////////////////////
// receiving direction

struct RecvBuffer<T: AsyncRead> {
    reader: ReadHalf<T>,
    buffer: Buffer,
}

//Result type used by RecvBuffer::poll_messages().
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
#[must_use]
struct StreamModeChanged(bool);

impl<T: AsyncRead> RecvBuffer<T> {
    fn new(reader: ReadHalf<T>, max_client_message_length: usize) -> Self {
        RecvBuffer {
            reader: reader,
            buffer: Buffer::new(max_client_message_length),
        }
    }

    fn poll_stdout_into(&mut self, result: &mut Vec<u8>) -> Poll<(), std::io::Error> {
        loop {
            //check the buffer for available stdout *before* poll_read because
            //there may be some leftovers in there from a previous poll_messages()
            result.extend(self.buffer.filled());
            let fill = self.buffer.fill;
            self.buffer.discard(fill);

            let bytes_read = try_ready!(
                self.reader.poll_read(self.buffer.unfilled_mut())
            );
            self.buffer.fill += bytes_read;
            if bytes_read == 0 {
                //EOF
                return Ok(Async::Ready(()));
            }
        }
    }

    //NOTE: When handle_message returns Ok(Ready(x)), then
    //x = StreamModeChanged(true) indicates that poll_messages() needs to break
    //its loop because the stream changed from message mode to something else,
    //and StreamModeChanged(false) indicates that processing can continue.
    //
    //In the return value, StreamModeChanged(true) is the same as above, and
    //StreamModeChanged(false) indicates EOF on `self.reader`.
    fn poll_messages<F>(&mut self, connection_id: u32, mut handle_message: F)
        -> Poll<StreamModeChanged, std::io::Error>
        where F: FnMut(&msg::Message) -> Poll<StreamModeChanged, std::io::Error>
    {
        use vt6::common::core::msg::ParseErrorKind::UnexpectedEOF;

        //NOTE: We cannot handle `bytes_to_discard` and `incomplete` directly
        //inside the match arms because the reference to `self.buffer.filled()`
        //needs to go out of scope first.
        let (bytes_to_discard, incomplete, stream_mode_changed)
                = match msg::Message::parse(self.buffer.filled()) {
            Ok((msg, bytes_consumed)) => {
                let result = try_ready!(handle_message(&msg));
                (bytes_consumed, false, result == StreamModeChanged(true))
            },
            Err(ref e) if e.kind == UnexpectedEOF && self.buffer.unfilled_len() > 0 => {
                (0, true, false)
            },
            Err(e) => {
                //parser error -> reset the stream parser [vt6/core1.0; sect. 2.3]
                let bytes_to_discard = self.buffer.buf.iter().skip(1).position(|&c| c == b'{')
                    .map(|x| x + 1).unwrap_or(self.buffer.fill);
                //^ The .skip(1) is necessary to ensure that bytes_to_discard > 0.
                //The .map() compensates the effect of .skip(1) on the index.
                let discarded = String::from_utf8_lossy(self.buffer.leading(bytes_to_discard));
                error!("input discarded on connection {}: {:?}", connection_id, discarded);
                error!("-> reason: {}", e);
                (bytes_to_discard, false, false)
            },
        };

        //we have read something (either a message or a definitive parser
        //error), so now we need to discard the bytes that were processed from
        //the recv buffer
        self.buffer.discard(bytes_to_discard);
        //do not continue when the stream mode has changed; the caller
        //(BidiByteStream) needs to switch to a different reading strategy
        if stream_mode_changed {
            return Ok(Async::Ready(StreamModeChanged(true)));
        }

        if incomplete {
            //it appears we have not read a full message yet
            if self.buffer.unfilled_len() > 0 {
                let bytes_read = try_ready!(self.reader.poll_read(self.buffer.unfilled_mut()));
                self.buffer.fill += bytes_read;
                if bytes_read == 0 {
                    //EOF - if we still have something in the buffer, it's an
                    //unfinished message -> complain
                    if self.buffer.fill > 0 {
                        let err = msg::Message::parse(self.buffer.filled()).unwrap_err();
                        let discarded = String::from_utf8_lossy(self.buffer.filled());
                        error!("input discarded on connection {}: {:?}", connection_id, discarded);
                        error!("-> reason: {}", err);
                    }
                    return Ok(Async::Ready(StreamModeChanged(false)));
                }
            }
            //restart handler with the new data
            return self.poll_messages(connection_id, handle_message);
        }

        //attempt to read the next message immediately
        self.poll_messages(connection_id, handle_message)
    }
}

////////////////////////////////////////////////////////////////////////////////
// sending direction

struct SendBuffer<T: AsyncWrite> {
    writer: WriteHalf<T>,
    //variable-size buffer for appending user input to
    stdin: Vec<u8>,
    //fixed-size buffer for rendering messages into
    message_buffer: Buffer,
}

impl<T: AsyncWrite> SendBuffer<T> {
    fn new(writer: WriteHalf<T>, max_server_message_length: usize) -> Self {
        SendBuffer {
            writer: writer,
            stdin: Vec::new(),
            //provide some extra space beyond max_server_message_length to allow
            //the handler to enqueue multiple messages if the stream is lacking
            //behind
            message_buffer: Buffer::new(max_server_message_length + 1024),
        }
    }

    fn can_write(&self) -> bool {
        self.stdin.len() > 0 || self.message_buffer.fill > 0
    }

    fn poll_write(&mut self) -> Poll<(), std::io::Error> {
        //check if we can send the client some input
        if self.stdin.len() > 0 {
            match self.writer.poll_write(&self.stdin[..]) {
                Err(e) => return Err(e),
                Ok(Async::NotReady) => {},
                Ok(Async::Ready(bytes_written)) => {
                    //remove the written bytes from the write buffer
                    self.stdin = self.stdin.split_off(bytes_written);
                    return self.poll_write(); //immediately try sending more
                },
            }
        }

        //check if we can send the client some messages
        let bytes_sent = try_ready!(
            self.writer.poll_write(self.message_buffer.filled()));
        self.message_buffer.discard(bytes_sent);
        Ok(Async::NotReady) //we can always add more stuff to the send buffer
    }
}

////////////////////////////////////////////////////////////////////////////////
// fixed-size buffer (used by both SendBuffer and RecvBuffer)

struct Buffer {
    buf: Vec<u8>,
    fill: usize,
}

impl Buffer {
    fn new(size: usize) -> Self {
        Self { buf: vec![0; size], fill: 0 }
    }

    //assorted helper methods
    fn unfilled_len(&self) -> usize { self.buf.len() - self.fill }
    fn leading(&self, bytes: usize) -> &[u8] { &self.buf[0 .. bytes] }
    fn filled(&self) -> &[u8] { self.leading(self.fill) }
    fn unfilled_mut(&mut self) -> &mut [u8] { &mut self.buf[self.fill ..] }

    ///Discards the given number of bytes from the buffer and shifts the
    ///remaining bytes to the left by that much.
    fn discard(&mut self, count: usize) {
        let remaining = self.fill - count;
        for idx in 0..remaining {
            self.buf[idx] = self.buf[idx + count];
        }
        for idx in remaining..self.buf.len() {
            self.buf[idx] = 0;
        }
        self.fill = remaining;
    }
}
