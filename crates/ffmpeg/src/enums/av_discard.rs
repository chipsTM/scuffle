use nutype_enum::{bitwise_enum, nutype_enum};

use crate::ffi::*;

const _: () = {
    assert!(std::mem::size_of::<AVDiscard>() == std::mem::size_of_val(&AVDISCARD_NONE));
};

nutype_enum! {
    /// Discard levels used in FFmpeg's `AVDiscard`.
    ///
    /// These values specify how much of the input stream should be discarded.
    ///
    /// See the official FFmpeg documentation:
    /// <https://ffmpeg.org/doxygen/trunk/avcodec_8h.html>
    pub enum AVDiscard(i32) {
        /// **Discard nothing** (decode everything).
        /// - **Used for**: Keeping all packets.
        /// - **Binary representation**: `-0b10000`
        /// - **Equivalent to**: `AVDISCARD_NONE`
        None = AVDISCARD_NONE as _,

        /// **Discard useless packets** (e.g., zero-size packets in AVI).
        /// - **Used for**: Cleaning up unnecessary data.
        /// - **Binary representation**: `0b00000`
        /// - **Equivalent to**: `AVDISCARD_DEFAULT`
        Default = AVDISCARD_DEFAULT as _,

        /// **Discard all non-reference frames**.
        /// - **Used for**: Reducing decoding load while keeping keyframe accuracy.
        /// - **Binary representation**: `0b01000`
        /// - **Equivalent to**: `AVDISCARD_NONREF`
        NonRef = AVDISCARD_NONREF as _,

        /// **Discard all bidirectional (B) frames**.
        /// - **Used for**: Lower latency decoding, reducing memory usage.
        /// - **Binary representation**: `0b10000`
        /// - **Equivalent to**: `AVDISCARD_BIDIR`
        Bidir = AVDISCARD_BIDIR as _,

        /// **Discard all non-intra frames**.
        /// - **Used for**: Keeping only intra-coded frames (I-frames).
        /// - **Binary representation**: `0b11000`
        /// - **Equivalent to**: `AVDISCARD_NONINTRA`
        NonIntra = AVDISCARD_NONINTRA as _,

        /// **Discard all frames except keyframes**.
        /// - **Used for**: Extracting only keyframes from a stream.
        /// - **Binary representation**: `0b100000`
        /// - **Equivalent to**: `AVDISCARD_NONKEY`
        NonKey = AVDISCARD_NONKEY as _,

        /// **Discard all frames** (decode nothing).
        /// - **Used for**: Disabling decoding entirely.
        /// - **Binary representation**: `0b110000`
        /// - **Equivalent to**: `AVDISCARD_ALL`
        All = AVDISCARD_ALL as _,
    }
}

bitwise_enum!(AVDiscard);

impl PartialEq<i32> for AVDiscard {
    fn eq(&self, other: &i32) -> bool {
        self.0 == *other
    }
}

impl From<u32> for AVDiscard {
    fn from(value: u32) -> Self {
        AVDiscard(value as i32)
    }
}

impl From<AVDiscard> for u32 {
    fn from(value: AVDiscard) -> Self {
        value.0 as u32
    }
}
