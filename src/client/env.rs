/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::client::AsyncRuntime;
use crate::common::core::msg;
use crate::common::core::msg::DecodeMessage;
use crate::msg::posix::ParentHello;
use core::fmt;
use std::fs::File;
use std::io::Read;
use std::marker::PhantomData;
use std::os::unix::io::FromRawFd;

///General information about the current client process.
///
///VT6 clients usually hold a singleton of this, e.g. through `lazy_static` or `once_cell`:
///
///```ignore
///use vt6::client::Environment;
///
///lazy_static! {
///    static ref VT6_ENV: Environment = Environment::discover().unwrap();
///}
///
///fn main() {
///    let env = VT6_ENV.parse().unwrap();
///    // ... use `env` ..
///}
///```
///
///This type is carefully designed for use with no_std, but currently requires std mostly because
///libcore does not have any IO facilities, cf. <https://github.com/rust-lang/rfcs/issues/2262>.
pub struct Environment {
    ///Read buffer for receiving the ParentHello message.
    buf: [u8; 1024],
    ///How many bytes of `self.buf` is filled (counting from the beginning).
    filled: usize,
    ///Whether FD 60 exists.
    has_vt6_terminal: bool,
}

impl Environment {
    ///Constructs an instance of `Environment` by reading from file descriptor 60 as defined in
    ///[\[vt6/posix1.0, section 2.2\]](https://vt6.io/std/posix/1.0/#section-2-2). This function
    ///only reports IO errors. All further errors will be reported by `parse()`.
    ///
    ///File descriptor 60 will be closed after this, so this operation will only work once. As
    ///described in the documentation on `struct Environment`, the resulting Environment instance
    ///should be held as a singleton, either in `main()` or through `lazy_static!` or similar
    ///facilities.
    pub fn discover() -> std::io::Result<Self> {
        let mut env = Self {
            buf: [0u8; 1024],
            filled: 0,
            has_vt6_terminal: true,
        };

        //SAFETY: we need to call this unsafe trait method to obtain a File handle
        let mut f = unsafe { File::from_raw_fd(60) };
        //NOTE: `impl Drop for File` will close FD 60 when this method returns.

        //the first read on FD 60 decides if we are on a VT6 terminal or not
        match f.read(&mut env.buf) {
            Ok(filled) => env.filled = filled,
            Err(e) => {
                if matches!(e.raw_os_error(), Some(errno) if errno == libc::EBADF || errno == libc::EINVAL)
                {
                    env.has_vt6_terminal = false;
                    return Ok(env);
                } else {
                    return Err(e);
                }
            }
        }

        //continue reading until we have a full parent-hello message or EOF or parse error
        while matches!(msg::Message::parse(&env.buf[0..env.filled]), Err(e) if e.is_incomplete()) {
            let filled = f.read(&mut env.buf[env.filled..])?;
            env.filled += filled;
            if filled == 0 {
                //we reached EOF on FD 60, so no more reads necessary
                break;
            }
        }
        Ok(env)
    }

    ///Parses the data that was read during `discover()` into an instance of `EnvironmentRef`. This
    ///operation can be repeated as many times as necessary. If `EnvironmentRef` instances are
    ///needed in multiple threads, each thread can run `parse()` on its own.
    pub fn parse<R: AsyncRuntime>(&self) -> Result<EnvironmentRef<'_, R>, EnvironmentError<'_>> {
        use EnvironmentError::*;
        if !self.has_vt6_terminal {
            return Err(NoVT6Terminal);
        }
        let (m, _) = msg::Message::parse(&self.buf[0..self.filled]).map_err(CorruptParentHello)?;
        if let Some(hello) = ParentHello::decode_message(&m) {
            return Ok(EnvironmentRef::<R> {
                _phantom: PhantomData,
                hello,
            });
        }
        Err(InvalidParentHello(m))
    }
}

///Provides access to the data contained in [struct Environment](struct.Environment.html).
///
///Unlike `Environment`, which is a singleton by design, instances of this type can be cloned and
///passed around freely.
#[derive(Clone, Debug)]
pub struct EnvironmentRef<'a, R: AsyncRuntime> {
    _phantom: PhantomData<R>,
    hello: ParentHello<'a>,
}

impl<R: AsyncRuntime> EnvironmentRef<'_, R> {
    ///Returns the filesystem path of the terminal's server socket.
    pub fn server_socket_path(&self) -> &std::path::Path {
        self.hello.server_socket_path
    }
}

///Error type returned from [`Environment::parse`](struct.Environment.html).
#[derive(Debug)]
pub enum EnvironmentError<'a> {
    ///The client is not connected to a VT6-capable terminal.
    NoVT6Terminal,
    ///The ParentHello message received by this client during discovery was not a valid VT6 message.
    CorruptParentHello(msg::ParseError<'a>),
    ///The server socket path from the ParentHello message is not a valid path.
    InvalidParentHello(msg::Message<'a>),
}

impl fmt::Display for EnvironmentError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::NoVT6Terminal => write!(f, "not connected to a VT6-capable terminal"),
            Self::CorruptParentHello(ref e) => write!(f, "cannot parse ParentHello: {}", e),
            Self::InvalidParentHello(ref msg) => write!(f, "invalid ParentHello: {}", msg),
        }
    }
}
