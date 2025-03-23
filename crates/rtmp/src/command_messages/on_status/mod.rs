//! Types and functions for processing the `onStatus` command.
//!
//! It is not very clear if the onStatus command should be part of the NetConnection or NetStream set of commands.
//! The legacy RTMP spec makes it look like it should be part of the NetStream commands while the enhanced-rtmp-v2 spec
//! is very clear that it should be part of the NetConnection commands.
//! In reality, it is used as a response message to both NetConnection and NetStream commands received from the client.
//! This is why we have decided to put it in its own module.

use std::borrow::Cow;

use scuffle_amf0::Amf0Object;

use crate::command_messages::CommandResultLevel;

pub mod writer;

/// The `onStatus` command is used to send status information from the server to the client.
#[derive(Debug, Clone, PartialEq)]
pub struct OnStatus<'a> {
    /// The status code.
    ///
    /// See the [`codes`] module for common status codes.
    pub code: Cow<'a, str>,
    /// The description of the status update.
    pub description: Option<Cow<'a, str>>,
    /// The level of the status update.
    pub level: CommandResultLevel,
    /// Any other additional information that should be sent as part of the object.
    pub others: Option<Amf0Object<'a>>,
}

// We can't use a nutype enum here because it would have to wrap a Cow<'a, str>.
// TODO: CLOUD-90
/// Common status codes used in the `onStatus` command.
#[allow(unused)]
pub mod codes {
    /// The `NetConnection.call()` method was not able to invoke the server-side method or command.
    pub const NET_CONNECTION_CALL_FAILED: &str = "NetConnection.Call.Failed";
    /// The application has been shut down (for example, if the application is out of memory resources
    /// and must shut down to prevent the server from crashing) or the server has shut down.
    pub const NET_CONNECTION_CONNECT_APP_SHUTDOWN: &str = "NetConnection.Connect.AppShutdown";
    /// The connection was closed successfully.
    pub const NET_CONNECTION_CONNECT_CLOSED: &str = "NetConnection.Connect.Closed";
    /// The connection attempt failed.
    pub const NET_CONNECTION_CONNECT_FAILED: &str = "NetConnection.Connect.Failed";
    /// The client does not have permission to connect to the application.
    pub const NET_CONNECTION_CONNECT_REJECTED: &str = "NetConnection.Connect.Rejected";
    /// The connection attempt succeeded.
    pub const NET_CONNECTION_CONNECT_SUCCESS: &str = "NetConnection.Connect.Success";
    /// The server is requesting the client to reconnect.
    pub const NET_CONNECTION_CONNECT_RECONNECT_REQUEST: &str = "NetConnection.Connect.ReconnectRequest";
    /// The proxy server is not responding. See the ProxyStream class.
    pub const NET_CONNECTION_PROXY_NOT_RESPONDING: &str = "NetConnection.Proxy.NotResponding";

    /// Publishing has started.
    pub const NET_STREAM_PUBLISH_START: &str = "NetStream.Publish.Start";
    /// Stream was successfully deleted.
    pub const NET_STREAM_DELETE_STREAM_SUCCESS: &str = "NetStream.DeleteStream.Suceess";
}
