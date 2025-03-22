use std::borrow::Cow;

use scuffle_amf0::Amf0Object;

use crate::command_messages::CommandResultLevel;

// It is not very clear if the onStatus message should be part of the NetConnection or NetStream commands.
// The legacy RTMP spec makes it look like it should be part of the NetStream commands while the enhanced-rtmp-v2 spec
// is very clear that it should be part of the NetConnection commands.
// In reality, it is used as a response message to both NetConnection and NetStream commands.
// This is why we have decided to put it in its own module.

pub mod writer;

#[derive(Debug, Clone, PartialEq)]
pub struct OnStatus<'a> {
    pub code: Cow<'a, str>,
    pub description: Option<Cow<'a, str>>,
    pub level: CommandResultLevel,
    pub others: Option<Amf0Object<'a>>,
}

// We can't use a nutype enum here because it would have to wrap a Cow<'a, str>.
#[allow(unused)]
pub mod codes {
    pub const NET_CONNECTION_CALL_FAILED: &str = "NetConnection.Call.Failed";
    pub const NET_CONNECTION_CONNECT_APP_SHUTDOWN: &str = "NetConnection.Connect.AppShutdown";
    pub const NET_CONNECTION_CONNECT_CLOSED: &str = "NetConnection.Connect.Closed";
    pub const NET_CONNECTION_CONNECT_FAILED: &str = "NetConnection.Connect.Failed";
    pub const NET_CONNECTION_CONNECT_REJECTED: &str = "NetConnection.Connect.Rejected";
    pub const NET_CONNECTION_CONNECT_SUCCESS: &str = "NetConnection.Connect.Success";
    pub const NET_CONNECTION_CONNECT_RECONNECT_REQUEST: &str = "NetConnection.Connect.ReconnectRequest";
    pub const NET_CONNECTION_PROXY_NOT_RESPONDING: &str = "NetConnection.Proxy.NotResponding";

    pub const NET_STREAM_PUBLISH_START: &str = "NetStream.Publish.Start";
    pub const NET_STREAM_DELETE_STREAM_SUCCESS: &str = "NetStream.DeleteStream.Suceess";
}
