use nutype_enum::{bitwise_enum, nutype_enum};

use crate::ffi::*;

const _: () = {
    assert!(std::mem::size_of::<AVSeekFlag>() == std::mem::size_of_val(&AVSEEK_FLAG_BACKWARD));
};

nutype_enum! {
    /// Seek flags used in FFmpeg's `av_seek_frame` function.
    ///
    /// These flags modify how seeking is performed in media files.
    ///
    /// See the official FFmpeg documentation:
    /// <https://ffmpeg.org/doxygen/trunk/group__lavf__decoding.html#gaa59bdaec0590cc36300753c5cf6c9d49>
    pub enum AVSeekFlag(i32) {
        /// Seek to the closest keyframe before the specified timestamp.
        /// - **Used for**: Ensuring accurate decoding by seeking to a valid keyframe.
        /// - **Binary representation**: `0b0000000000000001`
        /// - **Equivalent to**: `AVSEEK_FLAG_BACKWARD`
        Backward = AVSEEK_FLAG_BACKWARD as _,

        /// Seek by byte position instead of timestamp.
        /// - **Used for**: Formats where byte offsets are more reliable than timestamps.
        /// - **Binary representation**: `0b0000000000000010`
        /// - **Equivalent to**: `AVSEEK_FLAG_BYTE`
        Byte = AVSEEK_FLAG_BYTE as _,

        /// Seek to any frame, not just keyframes.
        /// - **Used for**: Allowing finer seeking granularity at the cost of possible decoding artifacts.
        /// - **Binary representation**: `0b0000000000000100`
        /// - **Equivalent to**: `AVSEEK_FLAG_ANY`
        Any = AVSEEK_FLAG_ANY as _,

        /// Seek based on frame numbers rather than timestamps.
        /// - **Used for**: Direct frame-based seeking in formats that support it.
        /// - **Binary representation**: `0b0000000000001000`
        /// - **Equivalent to**: `AVSEEK_FLAG_FRAME`
        Frame = AVSEEK_FLAG_FRAME as _,
    }
}

bitwise_enum!(AVSeekFlag);

impl PartialEq<i32> for AVSeekFlag {
    fn eq(&self, other: &i32) -> bool {
        self.0 == *other
    }
}

impl From<u32> for AVSeekFlag {
    fn from(value: u32) -> Self {
        AVSeekFlag(value as _)
    }
}

impl From<AVSeekFlag> for u32 {
    fn from(value: AVSeekFlag) -> Self {
        value.0 as u32
    }
}

const _: () = {
    assert!(std::mem::size_of::<AVSeekWhence>() == std::mem::size_of_val(&SEEK_SET));
};

nutype_enum! {
    /// Seek flags used in FFmpeg's `av_seek_frame` function.
    ///
    /// These flags modify how seeking is performed in media files.
    ///
    /// See the official FFmpeg documentation:
    /// <https://ffmpeg.org/doxygen/trunk/group__lavf__decoding.html#gaa59bdaec0590cc36300753c5cf6c9d49>
    pub enum AVSeekWhence(i32) {
        /// Seek from the beginning of the file.
        /// - **Used for**: Seeking from the start of the file.
        /// - **Binary representation**: `0b0000000000000001`
        /// - **Equivalent to**: `SEEK_SET`
        Start = SEEK_SET as _,

        /// Seek from the current position.
        /// - **Used for**: Seeking from the current position.
        /// - **Binary representation**: `0b0000000000000010`
        /// - **Equivalent to**: `SEEK_CUR`
        Current = SEEK_CUR as _,

        /// Seek from the end of the file.
        /// - **Used for**: Seeking from the end of the file.
        /// - **Binary representation**: `0b0000000000000100`
        /// - **Equivalent to**: `SEEK_END`
        End = SEEK_END as _,

        /// Return the file size instead of performing a seek.
        /// - **Used for**: Querying the total file size.
        /// - **Binary representation**: `0b00000000000000010000000000000000`
        /// - **Equivalent to**: `AVSEEK_SIZE`
        Size = AVSEEK_SIZE as _,

        /// Force seeking, even if the demuxer does not indicate it supports it.
        /// - **Used for**: Forcing a seek operation when the demuxer might otherwise refuse.
        /// - **Binary representation**: `0b00000000000000100000000000000000`
        /// - **Equivalent to**: `AVSEEK_FORCE`
        Force = AVSEEK_FORCE as _,
    }
}

bitwise_enum!(AVSeekWhence);

impl PartialEq<i32> for AVSeekWhence {
    fn eq(&self, other: &i32) -> bool {
        self.0 == *other
    }
}

impl From<u32> for AVSeekWhence {
    fn from(value: u32) -> Self {
        AVSeekWhence(value as _)
    }
}

impl From<AVSeekWhence> for u32 {
    fn from(value: AVSeekWhence) -> Self {
        value.0 as u32
    }
}
