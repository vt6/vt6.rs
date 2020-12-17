/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::common::core::{ClientID, OwnedClientID};
//TODO Once syntactical constraints on screen IDs are decided, add vt6::common::core::ScreenID. When we do, remove the `_screen_id` suffixes from method names where not necessary anymore.

///Information identifying a client.
///
///Clients are registered with the terminal through a message (such as `core1.client-make`) from
///the shell launching the client. The information from that announcement is stored as a
///ClientIdentity instance within the [Application](trait.Application.html). The client learns of
///its own identity by handshaking with the terminal: A successful msgio handshake will cause the
///terminal to send a message (such as `terminal-hello`) containing most of the data stored in the
///client's respective ClientIdentity instance.
///
///The [Application](trait.Application.html) usually holds on to ClientIdentity instances for their
///entire respective lifetime, to track which clients are currently alive.
#[derive(Clone, Debug)]
pub struct ClientIdentity {
    id: OwnedClientID,
    stdin_screen_id: Option<String>,
    stdout_screen_id: Option<String>,
    stderr_screen_id: Option<String>,
}

impl ClientIdentity {
    ///Constructs a new ClientIdentity. Further properties can be added via the `with_...` methods.
    ///
    ///```
    ///# use vt6::common::core::*;
    ///# use vt6::server::*;
    ///let identity = ClientIdentity::new(&ClientID::parse("example").unwrap())
    ///    .with_stdin("foo")
    ///    .with_stderr("bar");
    ///```
    pub fn new(id: &ClientID<'_>) -> Self {
        Self {
            id: id.into(),
            stdin_screen_id: None,
            stdout_screen_id: None,
            stderr_screen_id: None,
        }
    }

    ///Sets the `stdin_screen_id()` property on this ClientIdentity. Chain this after `new()` if
    ///and only if the client's stdin is connected to the terminal (instead of to a different type
    ///of file descriptor).
    pub fn with_stdin(self, screen_id: &str) -> ClientIdentity {
        ClientIdentity {
            stdin_screen_id: Some(screen_id.into()),
            ..self
        }
    }

    ///Sets the `stdout_screen_id()` property on this ClientIdentity. Chain this after `new()` if
    ///and only if the client's stdout is connected to the terminal (instead of to a different type
    ///of file descriptor).
    pub fn with_stdout(self, screen_id: &str) -> ClientIdentity {
        ClientIdentity {
            stdout_screen_id: Some(screen_id.into()),
            ..self
        }
    }

    ///Sets the `stderr_screen_id()` property on this ClientIdentity. Chain this after `new()` if
    ///and only if the client's stderr is connected to the terminal (instead of to a different type
    ///of file descriptor).
    pub fn with_stderr(self, screen_id: &str) -> ClientIdentity {
        ClientIdentity {
            stderr_screen_id: Some(screen_id.into()),
            ..self
        }
    }

    ///Returns the ID of this client.
    pub fn client_id(&self) -> ClientID<'_> {
        self.id.as_ref()
    }

    ///Returns the ID of the screen that this client's stdin is connected to, if any.
    pub fn stdin_screen_id(&self) -> Option<&str> {
        self.stdin_screen_id.as_ref().map(|s| s.as_ref())
    }

    ///Returns the ID of the screen that this client's stdout is connected to, if any.
    pub fn stdout_screen_id(&self) -> Option<&str> {
        self.stdout_screen_id.as_ref().map(|s| s.as_ref())
    }

    ///Returns the ID of the screen that this client's stderr is connected to, if any.
    pub fn stderr_screen_id(&self) -> Option<&str> {
        self.stderr_screen_id.as_ref().map(|s| s.as_ref())
    }
}

///Credentials issued for a client by the terminal.
#[derive(Clone, Debug)]
pub struct ClientCredentials {
    secret: String,
}

impl ClientCredentials {
    ///Generates a new ClientCredentials instance with a strongly random secret.
    pub fn generate() -> Self {
        Self {
            secret: generate_secret(),
        }
    }

    ///Returns the secret that this client can use to authenticate with the terminal.
    pub fn secret(&self) -> &str {
        &self.secret
    }
}

///Information identifying a screen.
///
///Screens are created either by the terminal itself (e.g. on startup) or in response to client
///messages. Either way, each screen is tracked as a ScreenIdentity instance (plus
///application-specific data) within the [Application](trait.Application.html).
#[derive(Clone, Debug)]
pub struct ScreenIdentity {
    id: String,
}

impl ScreenIdentity {
    ///Constructs a new ScreenIdentity.
    pub fn new(id: &str) -> Self {
        Self { id: id.into() }
    }

    ///Returns the ID of this screen.
    pub fn screen_id(&self) -> &str {
        &self.id
    }
}

///Credentials issued for a screen by the terminal.
#[derive(Clone, Debug)]
pub struct ScreenCredentials {
    stdin_secret: String,
    stdout_secret: String,
}

impl ScreenCredentials {
    ///Generates a new ClientCredentials instance with a strongly random secret.
    pub fn generate() -> Self {
        Self {
            stdin_secret: generate_secret(),
            stdout_secret: generate_secret(),
        }
    }

    ///Returns the secret that a client can use to attach to this screen's stdin.
    pub fn stdin_secret(&self) -> &str {
        &self.stdin_secret
    }

    ///Returns the secret that a client can use to attach to this screen's stdout.
    pub fn stdout_secret(&self) -> &str {
        &self.stdout_secret
    }
}

fn generate_secret() -> String {
    let mut buf1 = [0u8; 24];
    getrandom::getrandom(&mut buf1).unwrap();
    base64::encode_config(&buf1, base64::URL_SAFE)
}
