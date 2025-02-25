use nutype_enum::nutype_enum;

use crate::ffi::*;

const _: () = {
    assert!(std::mem::size_of::<AVMediaType>() == std::mem::size_of_val(&AVMEDIA_TYPE_UNKNOWN));
};

nutype_enum! {
    /// Represents the different media types supported by FFmpeg.
    ///
    /// See FFmpeg's `AVMediaType` in the official documentation:
    /// <https://ffmpeg.org/doxygen/trunk/group__lavu__misc.html#ga9a84bba4713dfced21a1a56163be1f48>
    pub enum AVMediaType(i32) {
        /// Unknown media type. Used when the type cannot be determined.
        /// Corresponds to `AVMEDIA_TYPE_UNKNOWN`.
        Unknown = AVMEDIA_TYPE_UNKNOWN as _,

        /// Video media type. Used for visual content such as movies or streams.
        /// Corresponds to `AVMEDIA_TYPE_VIDEO`.
        Video = AVMEDIA_TYPE_VIDEO as _,

        /// Audio media type. Represents sound or music data.
        /// Corresponds to `AVMEDIA_TYPE_AUDIO`.
        Audio = AVMEDIA_TYPE_AUDIO as _,

        /// Data media type. Typically used for supplementary or non-media data.
        /// Corresponds to `AVMEDIA_TYPE_DATA`.
        Data = AVMEDIA_TYPE_DATA as _,

        /// Subtitle media type. Represents textual or graphical subtitles.
        /// Corresponds to `AVMEDIA_TYPE_SUBTITLE`.
        Subtitle = AVMEDIA_TYPE_SUBTITLE as _,

        /// Attachment media type. Used for files attached to a media container (e.g., fonts for subtitles).
        /// Corresponds to `AVMEDIA_TYPE_ATTACHMENT`.
        Attachment = AVMEDIA_TYPE_ATTACHMENT as _,

        /// Special enumeration value representing the number of media types.
        /// Not an actual media type.
        /// Corresponds to `AVMEDIA_TYPE_NB`.
        Nb = AVMEDIA_TYPE_NB as _,
    }
}

impl PartialEq<i32> for AVMediaType {
    fn eq(&self, other: &i32) -> bool {
        self.0 == *other
    }
}

impl From<u32> for AVMediaType {
    fn from(value: u32) -> Self {
        AVMediaType(value as i32)
    }
}

impl From<AVMediaType> for u32 {
    fn from(value: AVMediaType) -> Self {
        value.0 as u32
    }
}
