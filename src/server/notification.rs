/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

///A notification that originates somewhere within this module. Notifications are sent to
///application-level code through the notify() function on [trait Dispatch](trait.Dispatch.html)
///where they can be logged or displayed to the user. Notifications are used only for informational
///messages and non-fatal errors.
///
///## Compatibility warning
///
///New versions of this library can add new variants to this enum at any time. Applications should
///always have a catch-all branch when matching on variants of this enum.
///
#[derive(Debug)]
pub enum Notification<'a> {
    ///A new client connection was accepted.
    ConnectionOpened,
    ///A client connection encountered an IO error.
    ConnectionIOError(Box<dyn std::error::Error>),
    ///A client connection was closed.
    ConnectionClosed,
    ///The referenced bytestring is about to be discarded from a receive buffer to recover from a
    ///parse error. This notification is always sent immediately after IncomingParseError.
    IncomingBytesDiscarded(&'a [u8]),
    //TODO Note to self: Before 1.0, check which variants have been obsoleted by proper APIs
    //elsewhere.
}

impl<'a> Notification<'a> {
    ///Returns whether this notification is an error or an informational message.
    pub fn is_error(&self) -> bool {
        match self {
            Self::ConnectionOpened => false,
            Self::ConnectionIOError(_) => true,
            Self::ConnectionClosed => false,
            Self::IncomingBytesDiscarded(_) => false,
        }
    }
}

impl<'a> std::fmt::Display for Notification<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConnectionOpened => {
                write!(f, "client connection opened")
            }
            Self::ConnectionIOError(e) => {
                write!(f, "client connection encountered IO error: {}", e)
            }
            Self::ConnectionClosed => {
                write!(f, "client connection closed")
            }
            Self::IncomingBytesDiscarded(buf) => {
                write!(
                    f,
                    "discarded invalid input: {:?}",
                    std::string::String::from_utf8_lossy(buf)
                )
            }
        }
    }
}
