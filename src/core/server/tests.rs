/*******************************************************************************
*
* Copyright 2018 Stefan Majewsky <majewsky@gmx.net>
*
* This program is free software: you can redistribute it and/or modify it under
* the terms of the GNU General Public License as published by the Free Software
* Foundation, either version 3 of the License, or (at your option) any later
* version.
*
* This program is distributed in the hope that it will be useful, but WITHOUT ANY
* WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
* A PARTICULAR PURPOSE. See the GNU General Public License for more details.
*
* You should have received a copy of the GNU General Public License along with
* this program. If not, see <http://www.gnu.org/licenses/>.
*
*******************************************************************************/

use core::{self, msg};
use server::{self, EarlyHandler};

#[test]
fn test_wanthave_basic() {
    assert_eq!(
        TestConnection::handle_single_message("{3|4:want,4:test,1:1,}"),
        Some("{3|4:have,4:test,3:1.3,}".into()),
    );
    assert_eq!(
        TestConnection::handle_single_message("{3|4:want,4:test,1:2,}"),
        Some("{3|4:have,4:test,3:2.1,}".into()),
    );
    assert_eq!(
        TestConnection::handle_single_message("{3|4:want,4:test,1:3,}"),
        Some("{1|4:have,}".into()),
    );
    assert_eq!(
        TestConnection::handle_single_message("{4|4:want,4:test,1:1,1:2,}"),
        Some("{3|4:have,4:test,3:2.1,}".into()),
    );
    assert_eq!(
        TestConnection::handle_single_message("{4|4:want,4:test,1:1,1:3,}"),
        Some("{3|4:have,4:test,3:1.3,}".into()),
    );
    assert_eq!(
        TestConnection::handle_single_message("{4|4:want,4:test,1:3,1:4,}"),
        Some("{1|4:have,}".into()),
    );
}

#[test]
fn test_wanthave_replies_consistently() {
    let mut conn = TestConnection::new();

    assert_eq!(
        conn.handle_message("{3|4:want,4:test,1:1,}"),
        Some("{3|4:have,4:test,3:1.3,}".into()),
    );
    assert_eq!(
        conn.handle_message("{3|4:want,4:test,1:1,}"),
        Some("{3|4:have,4:test,3:1.3,}".into()),
    );
    assert_eq!(
        conn.handle_message("{3|4:want,4:test,1:2,}"),
        Some("{1|4:have,}".into()),
    );
    assert_eq!(
        conn.handle_message("{4|4:want,4:test,1:1,1:2,}"),
        Some("{3|4:have,4:test,3:1.3,}".into()),
    );
    assert_eq!(
        conn.handle_message("{4|4:want,4:test,1:1,1:3,}"),
        Some("{3|4:have,4:test,3:1.3,}".into()),
    );
    assert_eq!(
        conn.handle_message("{4|4:want,4:test,1:2,1:3,}"),
        Some("{1|4:have,}".into()),
    );
}

#[test]
fn test_invalid_wants() {
    //missing module name and major version
    assert_eq!(
        TestConnection::handle_single_message("{1|4:want,}"),
        None,
    );
    //missing major version
    assert_eq!(
        TestConnection::handle_single_message("{2|4:want,4:test,}"),
        None,
    );
    //missing module name
    assert_eq!(
        TestConnection::handle_single_message("{2|4:want,1:1,}"),
        None,
    );
    //wrong argument order
    assert_eq!(
        TestConnection::handle_single_message("{3|4:want,1:1,4:test,}"),
        None,
    );
    //malformed module name
    assert_eq!(
        TestConnection::handle_single_message("{3|4:want,7:foo.bar,1:1,}"),
        None,
    );
    //malformed major version
    assert_eq!(
        TestConnection::handle_single_message("{3|4:want,4:test,3:1.0,}"),
        None,
    );
    assert_eq!(
        TestConnection::handle_single_message("{3|4:want,4:test,1:0,}"),
        None,
    );
}

struct TestConnection {
    tracker: core::server::Tracker,
}

impl TestConnection {
    fn new() -> Self { TestConnection { tracker: core::server::Tracker::default() } }

    fn handle_single_message(input: &str) -> Option<String> {
        Self::new().handle_message(input)
    }

    fn handle_message(&mut self, input: &str) -> Option<String> {
        let (message, _) = msg::Message::parse(input.as_bytes()).unwrap();
        let handler = core::server::Handler::new(TestHandler {});
        let mut send_buf = vec![0;1024];
        let bytes_written = handler.handle(&message, self, &mut send_buf)?;
        use libcore::str;
        Some(str::from_utf8(&send_buf[0..bytes_written]).unwrap().into())
    }
}

impl server::Connection for TestConnection {
    fn max_server_message_length(&self) -> &usize { &1024 }
    fn max_client_message_length(&self) -> &usize { &1024 }

    fn enable_module(&mut self, name: &str, version: core::ModuleVersion) {
        self.tracker.enable_module(name, version)
    }
    fn is_module_enabled(&self, name: &str) -> Option<core::ModuleVersion> {
        self.tracker.is_module_enabled(name)
    }
}

////////////////////////////////////////////////////////////////////////////

struct TestHandler {} //NOTE: includes RejectHandler

impl<C: server::Connection> server::Handler<C> for TestHandler {

    fn handle(&self, _msg: &msg::Message, _conn: &mut C, _send_buffer: &mut [u8]) -> Option<usize> {
        None
    }

    fn can_use_module(&self, name: &str, major_version: u16, _conn: &C) -> Option<u16> {
        match (name, major_version) {
            ("test", 1) => Some(3),
            ("test", 2) => Some(1),
            _ => None,
        }
    }

    fn get_set_property<'c>(&self, _name: &str, _requested_value: Option<&[u8]>, _conn: &'c mut C) -> Option<&'c core::EncodeArgument> {
        None //TODO
    }

}
