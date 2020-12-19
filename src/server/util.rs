/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

///Choose a useful default for the `socket_path` argument that Dispatch constructors usually take.
///
///Right now, this always chooses "$XDG_RUNTIME_DIR/vt6/$PID", so an error is returned if the
///XDG_RUNTIME_DIR environment variable is not set. The XDG Base Directory Specification instructs
///applications to "fall back to a replacement directory with similar capabilities and print a
///warning message" if XDG_RUNTIME_DIR is not set, so if you know a suitable replacement directory
///for Unixes where XDG_RUNTIME_DIR is not set by the login manager, please send a patch.
pub fn default_socket_path() -> std::io::Result<std::path::PathBuf> {
    use std::io::{Error, ErrorKind};

    //we need XDG_RUNTIME_DIR as the base for our socket path
    let mut runtime_dir = match std::env::var_os("XDG_RUNTIME_DIR") {
        Some(s) => std::path::PathBuf::from(s),
        None => {
            let msg = "XDG_RUNTIME_DIR not set";
            return Err(Error::new(ErrorKind::InvalidInput, msg));
        }
    };
    if !runtime_dir.is_dir() {
        let msg = format!(
            "XDG_RUNTIME_DIR ({}) is not a directory or not accessible",
            runtime_dir.to_string_lossy()
        );
        return Err(Error::new(ErrorKind::InvalidInput, msg));
    }

    //we put our sockets in "$XDG_RUNTIME_DIR/vt6"
    runtime_dir.push("vt6");
    std::fs::create_dir_all(&runtime_dir)?;
    Ok(runtime_dir.join(std::process::id().to_string()))
}
