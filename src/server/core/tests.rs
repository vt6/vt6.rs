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

use std;

use common::core::{self, msg};
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

#[test]
fn test_property_handling() {
    let mut conn = TestConnection::new();

    //need to negotiate core before using core.set and core.sub
    assert_eq!(
        conn.handle_message("{3|4:want,4:core,1:1,}"),
        Some("{3|4:have,4:core,3:1.0,}".into()),
    );

    //check readonly properties in core
    assert_eq!(
        conn.handle_message("{2|8:core.sub,25:core.server-msg-bytes-max,}"),
        Some("{3|8:core.pub,25:core.server-msg-bytes-max,4:1024,}".into()),
    );
    assert_eq!(
        conn.handle_message("{2|8:core.sub,25:core.client-msg-bytes-max,}"),
        Some("{3|8:core.pub,25:core.client-msg-bytes-max,4:2048,}".into()),
    );

    //need to negotiate test before using test.title
    assert_eq!(
        conn.handle_message("{3|4:want,4:test,1:1,}"),
        Some("{3|4:have,4:test,3:1.3,}".into()),
    );

    //check core.sub
    assert_eq!(
        conn.handle_message("{2|8:core.sub,10:test.title,}"),
        Some("{3|8:core.pub,10:test.title,7:initial,}".into()),
    );
    //check core.set where the server accepts the suggested value
    assert_eq!(
        conn.handle_message("{3|8:core.set,10:test.title,0:,}"),
        Some("{3|8:core.pub,10:test.title,0:,}".into()),
    );
    assert_eq!(
        conn.handle_message("{3|8:core.set,10:test.title,19:Lorem ipsum, dolor.,}"),
        Some("{3|8:core.pub,10:test.title,19:Lorem ipsum, dolor.,}".into()),
    );
    //check core.set where the server normalizes the requested value
    assert_eq!(
        conn.handle_message("{3|8:core.set,10:test.title,28:Lorem ipsum, dolor sit amet.,}"),
        Some("{3|8:core.pub,10:test.title,20:Lorem ipsum, dolor s,}".into()),
    );
}

#[test]
fn test_property_handling_invalid_syntax() {
    let mut conn = TestConnection::new();

    //need to negotiate core before using core.set and core.sub
    assert_eq!(
        conn.handle_message("{3|4:want,4:core,1:1,}"),
        Some("{3|4:have,4:core,3:1.0,}".into()),
    );
    //need to negotiate test before using test.title
    assert_eq!(
        conn.handle_message("{3|4:want,4:test,1:1,}"),
        Some("{3|4:have,4:test,3:1.3,}".into()),
    );

    //core.sub: missing property name
    assert_eq!(
        conn.handle_message("{1|8:core.sub,}"),
        None,
    );
    //core.sub: multiple properties
    assert_eq!(
        conn.handle_message("{3|8:core.sub,10:test.title,25:core.server-msg-bytes-max,}"),
        None,
    );

    //core.set: missing property name
    assert_eq!(
        conn.handle_message("{1|8:core.set,}"),
        None,
    );
    //core.set: missing property value
    assert_eq!(
        conn.handle_message("{2|8:core.set,10:test.title,}"),
        None,
    );
    //core.set: unexpected extra arguments (an early draft of vt6/core1.0
    //allowed setting multiple properties at once like shown here)
    assert_eq!(
        conn.handle_message("{5|8:core.set,10:test.title,3:foo,25:core.server-msg-bytes-max,4:2048,}"),
        None,
    );
}

#[test]
fn test_property_handling_invalid_negotiation() {
    //cannot use core.sub or core.set without negotiating core first
    assert_eq!(
        TestConnection::handle_single_message("{2|8:core.sub,10:test.title,}"),
        None,
    );
    assert_eq!(
        TestConnection::handle_single_message("{3|8:core.set,10:test.title,3:foo,}"),
        None,
    );

    //cannot use test.title without negotiating title first
    let mut conn = TestConnection::new();
    assert_eq!(
        TestConnection::handle_single_message("{3|4:want,4:core,1:1,}"),
        Some("{3|4:have,4:core,3:1.0,}".into()),
    );
    assert_eq!(
        conn.handle_message("{2|8:core.sub,10:test.title,}"),
        None,
    );
    assert_eq!(
        conn.handle_message("{3|8:core.set,10:test.title,3:foo,}"),
        None,
    );
}

struct TestConnection {
    tracker: server::core::Tracker,
    title: String,
}

impl TestConnection {
    fn new() -> Self { TestConnection { tracker: server::core::Tracker::default(), title: "initial".into() } }

    fn handle_single_message(input: &str) -> Option<String> {
        Self::new().handle_message(input)
    }

    fn handle_message(&mut self, input: &str) -> Option<String> {
        let (message, _) = msg::Message::parse(input.as_bytes()).unwrap();
        let handler = server::core::Handler::new(TestHandler {});
        let mut send_buf = vec![0;1024];
        let bytes_written = handler.handle(&message, self, &mut send_buf)?;
        Some(std::str::from_utf8(&send_buf[0..bytes_written]).unwrap().into())
    }
}

impl server::Connection for TestConnection {
    fn max_server_message_length(&self) -> usize { 1024 }
    fn max_client_message_length(&self) -> usize { 2048 }

    fn enable_module(&mut self, name: &str, version: core::ModuleVersion) {
        self.tracker.enable_module(name, version)
    }
    fn is_module_enabled(&self, name: &str) -> Option<core::ModuleVersion> {
        self.tracker.is_module_enabled(name)
    }
}

////////////////////////////////////////////////////////////////////////////

struct TestHandler {} //NOTE: includes RejectHandler

impl server::Handler<TestConnection> for TestHandler {

    fn handle(&self, _msg: &msg::Message, _conn: &mut TestConnection, _send_buffer: &mut [u8]) -> Option<usize> {
        None
    }

    fn can_use_module(&self, name: &str, major_version: u16, _conn: &TestConnection) -> Option<u16> {
        match (name, major_version) {
            ("test", 1) => Some(3),
            ("test", 2) => Some(1),
            _ => None,
        }
    }

    fn handle_sub(&self, name: &str, conn: &mut TestConnection, send_buffer: &mut [u8]) -> Option<usize> {
        use common::core::msg::prerecorded::publish_property;
        use server::Connection;

        if name == "test.title" && conn.is_module_enabled("test").is_some() {
            publish_property(send_buffer, name, conn.title.as_str())
        } else {
            None
        }
    }

    fn handle_set(&self, name: &str, requested_value: &[u8], conn: &mut TestConnection, send_buffer: &mut [u8]) -> Option<usize> {
        use common::core::msg::prerecorded::publish_property;
        use server::Connection;

        //the "test.title" property accepts string values, but strings longer than 20 bytes are
        //truncated to fit (so that we can have testcases where `requested_value != new_value`)
        if name == "test.title" && conn.is_module_enabled("test").is_some() {
            if let Ok(mut new_str) = std::str::from_utf8(requested_value) {
                if new_str.len() > 20 {
                    let mut idx = 20;
                    while !new_str.is_char_boundary(idx) {
                        idx -= 1;
                    }
                    new_str = &new_str[0..idx];
                }
                conn.title = new_str.into();
            }
            publish_property(send_buffer, name, conn.title.as_str())
        } else {
            None
        }
    }

}
