use nutype_enum::nutype_enum;

use crate::ffi::*;

const _: () = {
    assert!(std::mem::size_of::<AVPictureType>() == std::mem::size_of_val(&AV_PICTURE_TYPE_NONE));
};

nutype_enum! {
    /// Picture types used in FFmpeg's `AVPictureType`.
    ///
    /// Picture types define the role of a frame within a video compression scheme.
    ///
    /// See the official FFmpeg documentation:
    /// <https://ffmpeg.org/doxygen/trunk/avutil_8h.html>
    pub enum AVPictureType(i32) {
        /// Undefined or unknown picture type.
        /// - **Used for**: Uninitialized or unspecified frames.
        /// - **Equivalent to**: `AV_PICTURE_TYPE_NONE`
        None = AV_PICTURE_TYPE_NONE as _,

        /// **Intra-frame (I-frame)**: A self-contained frame that does not reference others.
        /// - **Used for**: Keyframes in compressed video.
        /// - **Efficient for**: Random access (seeking).
        /// - **Equivalent to**: `AV_PICTURE_TYPE_I`
        Intra = AV_PICTURE_TYPE_I as _,

        /// **Predicted frame (P-frame)**: Encodes differences relative to previous frames.
        /// - **Used for**: Intermediate frames in video compression.
        /// - **Smaller than I-frames but requires previous frames for decoding.**
        /// - **Equivalent to**: `AV_PICTURE_TYPE_P`
        Predicted = AV_PICTURE_TYPE_P as _,

        /// **Bi-directional predicted frame (B-frame)**: Uses both past and future frames for prediction.
        /// - **Used for**: High compression efficiency in video encoding.
        /// - **Requires both previous and future frames for decoding.**
        /// - **Equivalent to**: `AV_PICTURE_TYPE_B`
        BiPredicted = AV_PICTURE_TYPE_B as _,

        /// **Sprite (S-GMC) VOP** in MPEG-4.
        /// - **Used for**: Global motion compensation (GMC) in older MPEG-4 video.
        /// - **Equivalent to**: `AV_PICTURE_TYPE_S`
        SpriteGmc = AV_PICTURE_TYPE_S as _,

        /// **Switching Intra-frame (SI-frame)**: A special type of I-frame.
        /// - **Used for**: Scalable video coding, ensuring smooth transitions.
        /// - **Equivalent to**: `AV_PICTURE_TYPE_SI`
        SwitchingIntra = AV_PICTURE_TYPE_SI as _,

        /// **Switching Predicted frame (SP-frame)**: A special type of P-frame.
        /// - **Used for**: Scalable video coding, allowing switching between streams.
        /// - **Equivalent to**: `AV_PICTURE_TYPE_SP`
        SwitchingPredicted = AV_PICTURE_TYPE_SP as _,

        /// **BI type frame**: Similar to a B-frame but has additional constraints.
        /// - **Used for**: Certain video codecs with different motion compensation.
        /// - **Equivalent to**: `AV_PICTURE_TYPE_BI`
        BiType = AV_PICTURE_TYPE_BI as _,
    }
}

impl PartialEq<i32> for AVPictureType {
    fn eq(&self, other: &i32) -> bool {
        self.0 == *other
    }
}

impl From<u32> for AVPictureType {
    fn from(value: u32) -> Self {
        AVPictureType(value as _)
    }
}

impl From<AVPictureType> for u32 {
    fn from(value: AVPictureType) -> Self {
        value.0 as u32
    }
}
