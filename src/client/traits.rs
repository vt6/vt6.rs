/*******************************************************************************
* Copyright 2021 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::client::{Connection, Poller};
use crate::common::core::msg::{EncodeMessage, Message};
use futures::channel::oneshot;
use futures::io::{AsyncRead, AsyncWrite};

///A trait for messages that a client can send as a request to the terminal.
pub trait Request: EncodeMessage {
    ///Checks whether the given message is a response to this request. The Connection uses this to
    ///distinguish our request's response from any delayed responses that may arrive while we wait
    ///for our response.
    fn is_response(&self, msg: &Message<'_>) -> bool;

    //NOTE: Right now, responses are usually run through `DecodeMessage` twice, once in
    //`is_response()`, once in the method that receives the response from
    //`Connection::request().await`. It would be nicer to have a trait method like
    //
    //    fn match_response<'m>(&'_ self, msg: &Message<'m>) -> Option<ResponseType<'m>>
    //
    //instead, where `ResponseType` is an associated type, but we cannot express this until
    //Generic Associated Types are stabilized <https://github.com/rust-lang/rust/issues/44265>.
}

///TODO doc
pub trait AsyncRuntime: Clone + Send {
    type StreamReader: AsyncRead;
    type StreamWriter: AsyncWrite;
    fn spawn_poller<D: DelayedResponseHandler>(&self, p: Poller<Self, D>);
}

///TODO doc
pub trait DelayedResponseHandler: Send {
    fn handle(&mut self, msg: &Message<'_>);
}
