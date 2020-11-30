/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

mod decode_argument;
pub use self::decode_argument::*;
mod encode_argument;
pub use self::encode_argument::*;
mod identifiers;
pub use self::identifiers::*;

///Parsing and serializing of VT6 messages.
pub mod msg;
