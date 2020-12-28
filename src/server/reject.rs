/*******************************************************************************
* Copyright 2020 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use crate::common::core::{msg, ModuleIdentifier};
use crate::server;

///A [Handler](trait.Handler.html) that just rejects everything as
///[UnknownMessageType](enum.HandlerResult.html).
///
///This handler is usually the last in every MessageHandler chain. Valid messages will be
///processeed by an earlier handler and never reach this handler.
#[derive(Default)]
pub struct RejectHandler;

impl<A: server::Application> server::HandshakeHandler<A> for RejectHandler {}

impl<A: server::Application> server::MessageHandler<A> for RejectHandler {
    fn get_supported_module_version(&self, _module: &ModuleIdentifier<'_>) -> Option<u16> {
        None
    }
}

impl<A: server::Application> server::Handler<A> for RejectHandler {
    fn handle<D: server::Dispatch<A>>(
        &self,
        _msg: &msg::Message,
        _conn: &mut server::Connection<A, D>,
    ) -> Result<(), server::HandlerError> {
        Err(server::HandlerError::UnknownMessageType)
    }

    fn handle_error<D: server::Dispatch<A>>(
        &self,
        _err: &msg::ParseError,
        _conn: &mut server::Connection<A, D>,
    ) {
    }
}

impl<A: server::Application> server::core::MessageHandlerExt<A> for RejectHandler {}
