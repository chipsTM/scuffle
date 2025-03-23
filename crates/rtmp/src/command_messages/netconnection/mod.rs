//! NetConnection command messages.

use std::borrow::Cow;

use scuffle_amf0::Amf0Object;

use super::on_status::codes::NET_CONNECTION_CONNECT_SUCCESS;
use crate::command_messages::CommandResultLevel;

pub mod reader;
pub mod writer;

/// NetConnection command `connect`.
///
/// Defined by:
/// - Legacy RTMP spec, 7.2.1.1
/// - Enhanced RTMP spec, page 36-37, Enhancing NetConnection connect Command
#[derive(Debug, Clone, PartialEq)]
pub struct NetConnectionCommandConnect<'a> {
    /// Tells the server application name the client is connected to.
    pub app: Cow<'a, str>,
    /// represents capability flags which can be combined via a
    /// Bitwise OR to indicate which extended set of capabilities (i.e.,
    /// beyond the legacy RTMP specification) are supported via E-RTMP.
    /// See enum [`CapsExMask`] for the enumerated values representing the
    /// assigned bits.
    pub caps_ex: Option<CapsExMask>,
    /// All other parameters.
    ///
    /// Defined by:
    /// - Legacy RTMP spec, page 30
    /// - Enhanced RTMP spec, page 36-37
    pub others: Amf0Object<'a>,
}

/// Extended capabilities mask used by the [enhanced connect command](NetConnectionCommandConnect).
#[bitmask_enum::bitmask(u8)]
pub enum CapsExMask {
    /// Support for reconnection
    Reconnect = 0x01,
    /// Support for multitrack
    Multitrack = 0x02,
    /// Can parse ModEx signal
    ModEx = 0x04,
    /// Support for nano offset
    TimestampNanoOffset = 0x08,
}

/// NetConnection command `connect` result.
///
/// Defined by:
/// - Legacy RTMP spec, 7.2.1.1
#[derive(Debug, Clone, PartialEq)]
pub struct NetConnectionCommandConnectResult<'a> {
    /// Flash Media Server version.
    ///
    /// Usually set to "FMS/3,0,1,123".
    fmsver: Cow<'a, str>,
    /// No idea what this means, but it is used by other media servers as well.
    ///
    /// Usually set to 31.0.
    capabilities: f64,
    /// Result level.
    level: CommandResultLevel,
    /// Result code.
    ///
    /// Usually set to [`NET_CONNECTION_CONNECT_SUCCESS`].
    code: Cow<'a, str>,
    /// Result description.
    ///
    /// Usually set to "Connection Succeeded.".
    description: Cow<'a, str>,
    /// Not sure what this means but it may stand for the AMF encoding version.
    ///
    /// Usually set to 0.0.
    encoding: f64,
}

impl Default for NetConnectionCommandConnectResult<'_> {
    fn default() -> Self {
        Self {
            fmsver: Cow::Borrowed("FMS/3,0,1,123"),
            capabilities: 31.0,
            level: CommandResultLevel::Status,
            code: Cow::Borrowed(NET_CONNECTION_CONNECT_SUCCESS),
            description: Cow::Borrowed("Connection Succeeded."),
            encoding: 0.0,
        }
    }
}

/// NetConnection commands as defined in 7.2.1.
#[derive(Debug, Clone, PartialEq)]
pub enum NetConnectionCommand<'a> {
    /// Connect command.
    Connect(NetConnectionCommandConnect<'a>),
    /// Connect result.
    ///
    /// Sent from server to client in response to [`NetConnectionCommand::Connect`].
    ConnectResult(NetConnectionCommandConnectResult<'a>),
    /// Call command.
    Call,
    /// Close command.
    Close,
    /// Create stream command.
    CreateStream,
    /// Create stream result.
    ///
    /// Sent from server to client in response to [`NetConnectionCommand::CreateStream`].
    CreateStreamResult {
        /// ID of the created stream.
        stream_id: f64,
    },
}
