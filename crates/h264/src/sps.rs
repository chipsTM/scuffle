use std::io;

use byteorder::{BigEndian, ReadBytesExt};
use bytes::Bytes;
use scuffle_bytes_util::BitReader;
use scuffle_expgolomb::BitReaderExpGolombExt;

use crate::NALUnitType;

#[derive(Debug, Clone, PartialEq)]
/// The Sequence Parameter Set.
/// ISO/IEC-14496-10-2022 - 7.3.2
pub struct Sps {
    /// The `forbidden_zero_bit` is a single bit that must be set to 0. Otherwise
    /// `parse()` will return an error. ISO/IEC-14496-10-2022 - 7.4.1
    pub forbidden_zero_bit: bool,

    /// The `nal_ref_idc` is comprised of 2 bits.
    ///
    /// A nonzero value means the NAL unit has any of the following: SPS, SPS extension,
    /// subset SPS, PPS, slice of a reference picture, slice of a data partition of a reference picture,
    /// or a prefix NAL unit preceeding a slice of a reference picture.
    ///
    /// 0 means that the stream is decoded using the process from Clauses 2-9 (ISO/IEC-14496-10-2022)
    /// that the slice or slice data partition is part of a non-reference picture.
    /// Additionally, if `nal_ref_idc` is 0 for a NAL unit with `nal_unit_type`
    /// ranging from \[1, 4\] then `nal_ref_idc` must be 0 for all NAL units with `nal_unit_type` between [1, 4].
    ///
    /// If the `nal_unit_type` is 5, then the `nal_ref_idc` cannot be 0.
    ///
    /// If `nal_unit_type` is 6, 9, 10, 11, or 12, then the `nal_ref_idc` must be 0.
    ///
    /// ISO/IEC-14496-10-2022 - 7.4.1
    ///
    /// Note that this crate does NOT use the `nal_ref_idc` field for decoding.
    /// The parse function will still read the values if passed in correctly.
    /// If you have an application that requires this feature, please reach out to us at <https://scuffle.cloud>.
    pub nal_ref_idc: u8,

    /// The `nal_unit_type` is comprised of 5 bits. See the NALUnitType nutype enum for more info.
    pub nal_unit_type: NALUnitType,

    /// The `profile_idc` of the coded video sequence as a u8.
    ///
    /// It is comprised of 8 bits or 1 byte. ISO/IEC-14496-10-2022 - 7.4.2.1.1
    pub profile_idc: u8,

    /// `constraint_set0_flag`: `1` if it abides by the constraints in A.2.1, `0` if unsure or otherwise.
    ///
    /// It is a single bit. ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// Note that this crate does NOT use the `constraint_set0_flag` field for decoding.
    /// The parse function will still read the values if passed in correctly.
    /// If you have an application that requires this feature, please reach out to us at <https://scuffle.cloud>.
    pub constraint_set0_flag: bool,

    /// `constraint_set1_flag`: `1` if it abides by the constraints in A.2.2, `0` if unsure or otherwise.
    ///
    /// It is a single bit. ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// Note that this crate does NOT use the `constraint_set1_flag` field for decoding.
    /// The parse function will still read the values if passed in correctly.
    /// If you have an application that requires this feature, please reach out to us at <https://scuffle.cloud>.
    pub constraint_set1_flag: bool,

    /// `constraint_set2_flag`: `1` if it abides by the constraints in A.2.3, `0` if unsure or otherwise.
    ///
    /// It is a single bit. ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// Note that this crate does NOT use the `constraint_set2_flag` field for decoding.
    /// The parse function will still read the values if passed in correctly.
    /// If you have an application that requires this feature, please reach out to us at <https://scuffle.cloud>.
    pub constraint_set2_flag: bool,

    /// `constraint_set3_flag`:
    /// ```text
    ///     if (profile_idc == 66, 77, or 88) AND (level_idc == 11):
    ///         1 if it abides by the constraints in Annex A for level 1b
    ///         0 if it abides by the constraints in Annex A for level 1.1
    ///     elif profile_idc == 100 or 110:
    ///         1 if it abides by the constraints for the "High 10 Intra profile"
    ///         0 if unsure or otherwise
    ///     elif profile_idc == 122:
    ///         1 if it abides by the constraints in Annex A for the "High 4:2:2 Intra profile"
    ///         0 if unsure or otherwise
    ///     elif profile_idc == 44:
    ///         1 by default
    ///         0 is not possible.
    ///     elif profile_idc == 244:
    ///         1 if it abides by the constraints in Annex A for the "High 4:4:4 Intra profile"
    ///         0 if unsure or otherwise
    ///     else:
    ///         1 is reserved for future use
    ///         0 otherwise
    /// ```
    ///
    /// It is a single bit. ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// Note that this crate does NOT use the `constraint_set3_flag` field for decoding.
    /// The parse function will still read the values if passed in correctly.
    /// If you have an application that requires this feature, please reach out to us at <https://scuffle.cloud>.
    pub constraint_set3_flag: bool,

    /// `constraint_set4_flag`:
    /// ```text
    ///     if (profile_idc == 77, 88, 100, or 110):
    ///         1 if frame_mbs_only_flag == 1
    ///         0 if unsure or otherwise
    ///     elif (profile_idc == 118, 128, or 134):
    ///         1 if it abides by the constraints in G.6.1.1
    ///         0 if unsure or otherwise
    ///     else:
    ///         1 is reserved for future use
    ///         0 otherwise
    /// ```
    ///
    /// It is a single bit. ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// Note that this crate does NOT use the `constraint_set4_flag` field for decoding.
    /// The parse function will still read the values if passed in correctly.
    /// If you have an application that requires this feature, please reach out to us at <https://scuffle.cloud>.
    pub constraint_set4_flag: bool,

    /// `constraint_set5_flag`:
    /// ```text
    ///     if (profile_idc == 77, 88, or 100):
    ///         1 if there are no B slice types
    ///         0 if unsure or otherwise
    ///     elif profile_idc == 118:
    ///         1 if it abides by the constraints in G.6.1.2
    ///         0 if unsure or otherwise
    ///     else:
    ///         1 is reserved for future use
    ///         0 otherwise
    /// ```
    ///
    /// It is a single bit. ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// Note that this crate does NOT use the `constraint_set5_flag` field for decoding.
    /// The parse function will still read the values if passed in correctly.
    /// If you have an application that requires this feature, please reach out to us at <https://scuffle.cloud>.
    pub constraint_set5_flag: bool,

    /// The `level_idc` of the coded video sequence as a u8.
    ///
    /// It is comprised of 8 bits or 1 byte. ISO/IEC-14496-10-2022 - 7.4.2.1.1
    pub level_idc: u8,

    /// The `seq_parameter_set_id` is the id of the SPS referred to by the PPS (picture parameter set).
    ///
    /// The value of this ranges from \[0, 31\].
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// The smallest encoding would be for `0` which is encoded as `1`, which is a single bit.
    /// The largest encoding would be for `31` which is encoded as `000 0010 0000`, which is 11 bits.
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    ///
    /// Note that this crate does NOT use `seq_parameter_set_id` field for decoding.
    /// The parse function will still read the values if passed in correctly.
    /// If you have an application that requires this feature, please reach out to us at <https://scuffle.cloud>.
    pub seq_parameter_set_id: u16,

    /// An optional `SpsExtended`. Refer to the SpsExtended struct for more info.
    ///
    /// This will be parsed if `profile_idc` is equal to any of the following values:
    /// 44, 83, 86, 100, 110, 118, 122, 128, 134, 135, 138, 139, or 244.
    pub ext: Option<SpsExtended>,

    /// The `log2_max_frame_num_minus4` is the value used when deriving MaxFrameNum from the equation:
    /// `MaxFrameNum` = 2^(`log2_max_frame_num_minus4` + 4)
    ///
    /// The value of this ranges from \[0, 12\].
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// The smallest encoding would be for `0` which is encoded as `1`, which is a single bit.
    /// The largest encoding would be for `12` which is encoded as `000 1101`, which is 7 bits.
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    ///
    /// Note that this crate does NOT use the `log2_max_frame_num_minus4` field for decoding.
    /// The parse function will still read the values if passed in correctly.
    /// If you have an application that requires this feature, please reach out to us at <https://scuffle.cloud>.
    pub log2_max_frame_num_minus4: u8,

    /// The `pic_order_cnt_type` specifies how to decode the picture order count in subclause 8.2.1.
    ///
    /// The value of this ranges from \[0, 2\].
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// The smallest encoding would be for `0` which is encoded as `1`, which is a single bit.
    /// The largest encoding would be for `2` which is encoded as `011`, which is 3 bits.
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    ///
    /// There are a few subsequent fields that are read if `pic_order_cnt_type` is 0 or 1.
    ///
    /// In the case of 1, `log2_max_pic_order_cnt_lsb_minus4` is read as an exp golomb (unsigned).
    ///
    /// In the case of 2, `delta_pic_order_always_zero_flag`, `offset_for_non_ref_pic`,
    /// `offset_for_top_to_bottom_field`, `num_ref_frames_in_pic_order_cnt_cycle` and
    /// `offset_for_ref_frame` will be read.
    ///
    /// Note that this crate does NOT use the aforementioned fields for decoding.
    /// The parse function will still read the values if passed in correctly,
    /// but this struct does not store them at this time.
    /// If you have an application that requires this feature, please reach out to us at <https://scuffle.cloud>.
    pub pic_order_cnt_type: u8,

    /// The `max_num_ref_frames` is the max short-term and long-term reference frames,
    /// complementary reference field pairs, and non-paired reference fields that
    /// can be used by the decoder for inter-prediction of pictures in the coded video.
    ///
    /// The value of this ranges from \[0, `MaxDpbFrames`\], which is specified in subclause A.3.1 or A.3.2.
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// The smallest encoding would be for `0` which is encoded as `1`, which is a single bit.
    /// The largest encoding would be for `14` which is encoded as `000 1111`, which is 7 bits.
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    ///
    /// Note that this crate does NOT use the `max_num_ref_frames` field for decoding.
    /// The parse function will still read the values if passed in correctly (up to a u8).
    /// If you have an application that requires this feature, please reach out to us at <https://scuffle.cloud>.
    pub max_num_ref_frames: u8,

    /// The `gaps_in_frame_num_value_allowed_flag` is a single bit.
    ///
    /// The value specifies the allowed values of `frame_num` from subclause 7.4.3 and the decoding process
    /// if there is an inferred gap between the values of `frame_num` from subclause 8.2.5.2.
    ///
    /// Note that this crate does NOT use the `gaps_in_frame_num_value_allowed_flag` field for decoding.
    /// The parse function will still read the values if passed in correctly.
    /// If you have an application that requires this feature, please reach out to us at <https://scuffle.cloud>.
    pub gaps_in_frame_num_value_allowed_flag: bool,

    /// The `pic_width_in_mbs_minus1` is the width of each decoded picture in macroblocks as a u64.
    ///
    /// We then use this (along with the left and right frame crop offsets) to calculate the width as:
    ///
    /// `width = ((pic_width_in_mbs_minus1 + 1) * 16) - frame_crop_right_offset * 2 - frame_crop_left_offset * 2`
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub pic_width_in_mbs_minus1: u64,

    /// The `pic_height_in_map_units_minus1` is the height of each decoded frame in slice group map units as a u64.
    ///
    /// We then use this (along with the bottom and top frame crop offsets) to calculate the height as:
    ///
    /// `height = ((2 - frame_mbs_only_flag as u64) * (pic_height_in_map_units_minus1 + 1) * 16) -
    /// frame_crop_bottom_offset * 2 - frame_crop_top_offset * 2`
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub pic_height_in_map_units_minus1: u64,

    /// The `frame_mbs_only_flag` is a single bit.
    ///
    /// 0 means the coded pictures of the coded video sequence are either coded fields or coded frames
    /// and we will read the `mb_adaptive_frame_field_flag`.
    /// 1 means every coded picture of the coded video sequence is a coded frame with only frame macroblocks.
    ///
    /// We then use this to calculate the height as:
    ///
    /// `height = ((2 - frame_mbs_only_flag as u64) * (pic_height_in_map_units_minus1 + 1) * 16) -
    /// frame_crop_bottom_offset * 2 - frame_crop_top_offset * 2`
    ///
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// Note that this crate does NOT use the `mb_adaptive_frame_field_flag` field for decoding.
    /// The parse function will still read the values if passed in correctly,
    /// but this struct does not store them at this time.
    /// If you have an application that requires this feature, please reach out to us at <https://scuffle.cloud>.
    pub frame_mbs_only_flag: bool,

    /// The `direct_8x8_inference_flag` specifies the method used to derive the luma motion
    /// vectors for B_Skip, B_Direct_8x8 and B_Direct_16x16 from subclause 8.4.1.2, and is a single bit.
    ///
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// Note that this crate does NOT use the `direct_8x8_inference_flag` field for decoding.
    /// The parse function will still read the values if passed in correctly.
    /// If you have an application that requires this feature, please reach out to us at <https://scuffle.cloud>.
    pub direct_8x8_inference_flag: bool,

    /// The `frame_cropping_flag` is a single bit.
    ///
    /// 0 means the width and height aren't cropped.
    /// 1 means that we will parse the 4 frame crop offsets and use them to calculate the width and height.
    ///
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    pub frame_cropping_flag: bool,

    /// The `frame_crop_left_offset` is the the left crop offset which is used to compute the width:
    ///
    /// `width = ((pic_width_in_mbs_minus1 + 1) * 16) - frame_crop_right_offset * 2 - frame_crop_left_offset * 2`
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub frame_crop_left_offset: u64,

    /// The `frame_crop_right_offset` is the the right crop offset which is used to compute the width:
    ///
    /// `width = ((pic_width_in_mbs_minus1 + 1) * 16) - frame_crop_right_offset * 2 - frame_crop_left_offset * 2`
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub frame_crop_right_offset: u64,

    /// The `frame_crop_top_offset` is the the top crop offset which is used to compute the height:
    ///
    /// `height = ((2 - frame_mbs_only_flag as u64) * (pic_height_in_map_units_minus1 + 1) * 16)
    /// - frame_crop_bottom_offset * 2 - frame_crop_top_offset * 2`
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub frame_crop_top_offset: u64,

    /// The `frame_crop_bottom_offset` is the the bottom crop offset which is used to compute the height:
    ///
    /// `height = ((2 - frame_mbs_only_flag as u64) * (pic_height_in_map_units_minus1 + 1) * 16)
    /// - frame_crop_bottom_offset * 2 - frame_crop_top_offset * 2`
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub frame_crop_bottom_offset: u64,

    /// The width as a u64. This is computed from other fields, and isn't directly set.
    ///
    /// `width = ((pic_width_in_mbs_minus1 + 1) * 16) - frame_crop_right_offset * 2 - frame_crop_left_offset * 2`
    pub width: u64,

    /// The height as a u64. This is computed from other fields, and isn't directly set.
    ///
    /// `height = ((2 - frame_mbs_only_flag as u64) * (pic_height_in_map_units_minus1 + 1) * 16)
    /// - frame_crop_bottom_offset * 2 - frame_crop_top_offset * 2``
    pub height: u64,

    /// The `vui_parameters_present_flag` is a single bit. ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// 0 means we have a frame rate of 0 and we do not have a color config.
    /// 1 means we will compute the framerate (it may be 0) and the color config.
    ///
    /// If this is set, the parse function will read the `aspect_ratio_info_present_flag`, `aspect_ratio_idc`,
    /// `sar_width`, `sar_height`, `overscan_info_present_flag`, `overscan_appropriate_flag`,
    /// `video_format`, `chroma_loc_info_present_flag`, `chroma_sample_loc_type_top_field`,
    /// and `chroma_sample_loc_type_bottom_field` fields.
    ///
    /// Note that this crate does NOT use the aforementioned fields for decoding.
    /// The parse function will still read the values if passed in correctly,
    /// but this struct does not store them at this time.
    /// If you have an application that requires this feature, please reach out to us at <https://scuffle.cloud>.
    pub vui_parameters_present_flag: bool,

    /// The `video_signal_type_present_flag` is a single bit that determines whether we compute the `ColorConfig`.
    /// ISO/IEC-14496-10-2022 - E.2.1
    pub video_signal_type_present_flag: bool,

    /// The `color_description_present_flag` is a single bit that determines whether we read
    /// values to set for the `color_primaries`, `transfer_characteristics`, and `matrix_coefficients`
    /// all of which are passed into the `ColorConfig`.
    ///
    /// 1 means we read u8s (1 u8 per field) to store into the above fields.
    /// 0 means we set each of the fields to 2, which means unspecified.
    pub color_description_present_flag: bool,

    /// An optional `ColorConfig`. This is computed from other fields, and isn't directly set.
    ///
    /// If `video_signal_type_present_flag` is set, then the `ColorConfig` will be computed, and
    /// if the `color_description_present_flag` is set, then the `ColorConfig` will be
    /// comprised of the `video_full_range_flag` (1 bit), `color_primaries` (1 byte as a u8),
    /// `transfer_characteristics` (1 byte as a u8), and `matrix_coefficients` (1 byte as a u8).
    ///
    /// Refer to the ColorConfig struct for more info.
    pub color_config: Option<ColorConfig>,

    /// The `timing_info_present_flag` is a single bit that determines whether we read
    /// values to set for the `num_units_in_tick`, `time_scale`, used to compute
    /// the frame rate.
    ///
    /// 0 means we don't compute the frame rate (defaults to 0.0).
    /// 1 means we read 2 u32's (big endian) to then compute the frame rate as long as
    /// `num_units_in_tick` is nonzero.
    ///
    /// ISO/IEC-14496-10-2022 - E.2.1
    ///
    /// Both `num_units_in_tick` and `time_scale` are read the values if passed in correctly,
    /// but this struct does not store them at this time.
    /// If you have an application that requires this feature, please reach out to us at <https://scuffle.cloud>.
    pub timing_info_present_flag: bool,

    /// The framerate as a f64. This is computed from other fields, and isn't directly set.
    ///
    /// If `timing_info_present_flag` is set, then the `frame_rate` will be computed, and
    /// if `num_units_in_tick` is nonzero, then the framerate will be:
    /// `frame_rate = time_scale as f64 / (2.0 * num_units_in_tick as f64)`
    pub frame_rate: f64,
}

#[derive(Debug, Clone, PartialEq)]
/// The color config for SPS. ISO/IEC-14496-10-2022 - E.2.1
///
/// There are a few fields from the spec that we read but do not store, namely:
/// `aspect_ratio_info_present_flag`, `aspect_ratio_idc`, `sar_width`, `sar_height`,
/// `overscan_info_present_flag`, `overscan_appropriate_flag`, `video_format`,
/// `chroma_loc_info_present_flag`, `chroma_sample_loc_type_top_field`,
/// and `chroma_sample_loc_type_bottom_field`.
///
/// This crate does NOT use the aforementioned fields for decoding.
/// The parse function will still read the values if passed in correctly,
/// but this struct does not store them at this time.
/// If you have an application that requires this feature OR if we are missing any other h264 features
/// that you need, please reach out to us at <https://scuffle.cloud>.
pub struct ColorConfig {
    /// The `video_full_range_flag` is a single bit indicating the black level and range of
    /// luma and chroma signals.
    ///
    /// This field is passed into the `ColorConfig`.
    /// ISO/IEC-14496-10-2022 - E.2.1
    pub video_full_range_flag: bool,

    /// The `colour_primaries` byte as a u8. If `color_description_present_flag` is not set,
    /// the value defaults to 2. ISO/IEC-14496-10-2022 - E.2.1 Table E-3
    pub color_primaries: u8,

    /// The `transfer_characteristics` byte as a u8. If `color_description_present_flag` is not set,
    /// the value defaults to 2. ISO/IEC-14496-10-2022 - E.2.1 Table E-4
    pub transfer_characteristics: u8,

    /// The `matrix_coefficients` byte as a u8. If `color_description_present_flag` is not set,
    /// the value defaults to 2. ISO/IEC-14496-10-2022 - E.2.1 Table E-5
    pub matrix_coefficients: u8,
}

impl Sps {
    /// Parses an SPS from the input bytes.
    /// Returns an `Sps` struct.
    pub fn parse(data: Bytes) -> io::Result<Self> {
        // Returns an error if there aren't enough bytes.
        if data.len() < 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Insufficient data: SPS must be at least 4 bytes long",
            ));
        }

        let mut vec = Vec::with_capacity(data.len());

        // We need to remove the emulation prevention byte
        // This is BARELY documented in the spec, but it's there.
        // ISO/IEC-14496-10-2022 - 3.1.48
        let mut i = 0;
        while i < data.len() {
            if i + 2 < data.len() && data[i] == 0x00 && data[i + 1] == 0x00 && data[i + 2] == 0x03 {
                vec.push(0x00);
                vec.push(0x00);
                i += 3; // Skip the emulation prevention byte.
            } else {
                vec.push(data[i]);
                i += 1;
            }
        }

        let mut bit_reader = BitReader::new_from_slice(vec);

        let forbidden_zero_bit = bit_reader.read_bit()?;
        if forbidden_zero_bit {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Forbidden zero bit is set"));
        }

        let nal_ref_idc = bit_reader.read_bits(2)? as u8;
        let nal_unit_type = bit_reader.read_bits(5)? as u8;
        if nal_unit_type != 7 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "NAL unit type is not SPS"));
        }

        let profile_idc = bit_reader.read_u8()?;

        let constraint_set0_flag = bit_reader.read_bit()?;
        let constraint_set1_flag = bit_reader.read_bit()?;
        let constraint_set2_flag = bit_reader.read_bit()?;
        let constraint_set3_flag = bit_reader.read_bit()?;
        let constraint_set4_flag = bit_reader.read_bit()?;
        let constraint_set5_flag = bit_reader.read_bit()?;
        // reserved_zero_2bits
        bit_reader.seek_bits(2)?;

        let level_idc = bit_reader.read_u8()?;
        let seq_parameter_set_id = bit_reader.read_exp_golomb()? as u16;

        let sps_ext = match profile_idc {
            100 | 110 | 122 | 244 | 44 | 83 | 86 | 118 | 128 | 138 | 139 | 134 | 135 => {
                Some(SpsExtended::parse(&mut bit_reader)?)
            }
            _ => None,
        };

        let log2_max_frame_num_minus4 = bit_reader.read_exp_golomb()? as u8;
        let pic_order_cnt_type = bit_reader.read_exp_golomb()? as u8;
        if pic_order_cnt_type == 0 {
            bit_reader.read_exp_golomb()?; // log2_max_pic_order_cnt_lsb_minus4
        } else if pic_order_cnt_type == 1 {
            bit_reader.seek_bits(1)?; // delta_pic_order_always_zero_flag
            bit_reader.read_signed_exp_golomb()?; // offset_for_non_ref_pic
            bit_reader.read_signed_exp_golomb()?; // offset_for_top_to_bottom_field
            let num_ref_frames_in_pic_order_cnt_cycle = bit_reader.read_exp_golomb()?;
            for _ in 0..num_ref_frames_in_pic_order_cnt_cycle {
                bit_reader.read_signed_exp_golomb()?; // offset_for_ref_frame
            }
        }

        let max_num_ref_frames = bit_reader.read_exp_golomb()? as u8;
        let gaps_in_frame_num_value_allowed_flag = bit_reader.read_bit()?;
        let pic_width_in_mbs_minus1 = bit_reader.read_exp_golomb()?;
        let pic_height_in_map_units_minus1 = bit_reader.read_exp_golomb()?;
        let frame_mbs_only_flag = bit_reader.read_bit()?;
        if !frame_mbs_only_flag {
            bit_reader.seek_bits(1)?; // mb_adaptive_frame_field_flag
        }

        let direct_8x8_inference_flag = bit_reader.read_bit()?;

        let mut frame_crop_left_offset = 0;
        let mut frame_crop_right_offset = 0;
        let mut frame_crop_top_offset = 0;
        let mut frame_crop_bottom_offset = 0;

        let frame_cropping_flag = bit_reader.read_bit()?;
        if frame_cropping_flag {
            frame_crop_left_offset = bit_reader.read_exp_golomb()?; // frame_crop_left_offset
            frame_crop_right_offset = bit_reader.read_exp_golomb()?; // frame_crop_right_offset
            frame_crop_top_offset = bit_reader.read_exp_golomb()?; // frame_crop_top_offset
            frame_crop_bottom_offset = bit_reader.read_exp_golomb()?; // frame_crop_bottom_offset
        }

        let width = ((pic_width_in_mbs_minus1 + 1) * 16) - frame_crop_right_offset * 2 - frame_crop_left_offset * 2;
        let height = ((2 - frame_mbs_only_flag as u64) * (pic_height_in_map_units_minus1 + 1) * 16)
            - frame_crop_bottom_offset * 2
            - frame_crop_top_offset * 2;

        let mut frame_rate = 0.0;

        let vui_parameters_present_flag = bit_reader.read_bit()?;

        let mut video_signal_type_present_flag = false;
        let mut color_description_present_flag = false;
        let mut color_config = None;

        let mut timing_info_present_flag = false;

        if vui_parameters_present_flag {
            // We read the VUI parameters to get the frame rate.

            // aspect_ratio_info_present_flag
            if bit_reader.read_bit()? {
                let aspect_ratio_idc = bit_reader.read_u8()?;
                if aspect_ratio_idc == 255 {
                    bit_reader.seek_bits(16)?; // sar_width
                    bit_reader.seek_bits(16)?; // sar_height
                }
            }

            // overscan_info_present_flag
            if bit_reader.read_bit()? {
                bit_reader.seek_bits(1)?; // overscan_appropriate_flag
            }

            video_signal_type_present_flag = bit_reader.read_bit()?;
            if video_signal_type_present_flag {
                bit_reader.seek_bits(3)?; // video_format E.2.1 Table E-2
                let video_full_range_flag = bit_reader.read_bit()?;

                let color_primaries;
                let transfer_characteristics;
                let matrix_coefficients;

                color_description_present_flag = bit_reader.read_bit()?;
                if color_description_present_flag {
                    color_primaries = bit_reader.read_u8()?;
                    transfer_characteristics = bit_reader.read_u8()?;
                    matrix_coefficients = bit_reader.read_u8()?;
                } else {
                    color_primaries = 2; // UNSPECIFIED
                    transfer_characteristics = 2; // UNSPECIFIED
                    matrix_coefficients = 2; // UNSPECIFIED
                }

                color_config = Some(ColorConfig {
                    video_full_range_flag,
                    color_primaries,
                    transfer_characteristics,
                    matrix_coefficients,
                });
            }

            // chroma_loc_info_present_flag
            if bit_reader.read_bit()? {
                bit_reader.read_exp_golomb()?; // chroma_sample_loc_type_top_field
                bit_reader.read_exp_golomb()?; // chroma_sample_loc_type_bottom_field
            }

            timing_info_present_flag = bit_reader.read_bit()?;
            if timing_info_present_flag {
                let num_units_in_tick = bit_reader.read_u32::<BigEndian>()?;
                let time_scale = bit_reader.read_u32::<BigEndian>()?;

                if num_units_in_tick == 0 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "num_units_in_tick cannot be zero"));
                }

                frame_rate = time_scale as f64 / (2.0 * num_units_in_tick as f64);
            }
        }

        Ok(Sps {
            forbidden_zero_bit,
            nal_ref_idc,
            nal_unit_type: crate::NALUnitType(nal_unit_type),
            profile_idc,
            constraint_set0_flag,
            constraint_set1_flag,
            constraint_set2_flag,
            constraint_set3_flag,
            constraint_set4_flag,
            constraint_set5_flag,
            level_idc,
            seq_parameter_set_id,
            ext: sps_ext,
            log2_max_frame_num_minus4,
            pic_order_cnt_type,
            max_num_ref_frames,
            gaps_in_frame_num_value_allowed_flag,
            pic_width_in_mbs_minus1,
            pic_height_in_map_units_minus1,
            frame_mbs_only_flag,
            direct_8x8_inference_flag,
            frame_cropping_flag,
            frame_crop_left_offset,
            frame_crop_right_offset,
            frame_crop_top_offset,
            frame_crop_bottom_offset,
            width,
            height,
            vui_parameters_present_flag,
            video_signal_type_present_flag,
            color_description_present_flag,
            color_config,
            timing_info_present_flag,
            frame_rate,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
/// The Sequence Parameter Set extension.
/// ISO/IEC-14496-10-2022 - 7.3.2
pub struct SpsExtended {
    /// The `chroma_format_idc` as a u8. This is the chroma sampling relative
    /// to the luma sampling specified in subclause 6.2.
    ///
    /// The value of this ranges from \[0, 3\].
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// The smallest encoding would be for `0` which is encoded as `1`, which is a single bit.
    /// The largest encoding would be for `3` which is encoded as `0 0100`, which is 5 bits.
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub chroma_format_idc: u8,

    /// The `separate_colour_plane_flag` is a single bit.
    ///
    /// 0 means the the color components aren't coded separately and `ChromaArrayType` is set to `chroma_format_idc`.
    /// 1 means the 3 color components of the 4:4:4 chroma format are coded separately and `ChromaArrayType` is set to 0.
    ///
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    pub separate_color_plane_flag: bool,

    /// The `bit_depth_luma_minus8` as a u8. This is the chroma sampling relative
    /// to the luma sampling specified in subclause 6.2.
    ///
    /// The value of this ranges from \[0, 6\].
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// The smallest encoding would be for `0` which is encoded as `1`, which is a single bit.
    /// The largest encoding would be for `6` which is encoded as `0 0111`, which is 5 bits.
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub bit_depth_luma_minus8: u8,

    /// The `bit_depth_chroma_minus8` as a u8. This is the chroma sampling
    /// relative to the luma sampling specified in subclause 6.2.
    ///
    /// The value of this ranges from \[0, 6\].
    ///
    /// This is a variable number of bits as it is encoded by an exp golomb (unsigned).
    /// The smallest encoding would be for `0` which is encoded as `1`, which is a single bit.
    /// The largest encoding would be for `6` which is encoded as `0 0111`, which is 5 bits.
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// For more information:
    ///
    /// <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
    pub bit_depth_chroma_minus8: u8,

    /// The `qpprime_y_zero_transform_bypass_flag` is a single bit.
    ///
    /// 0 means the transform coefficient decoding and picture construction processes wont
    /// use the transform bypass operation.
    /// 1 means that when QP'_Y is 0 then a transform bypass operation for the transform
    /// coefficient decoding and picture construction processes will be applied before
    /// the deblocking filter process from subclause 8.5.
    ///
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    pub qpprime_y_zero_transform_bypass_flag: bool,

    /// The `seq_scaling_matrix_present_flag` is a single bit.
    ///
    /// 0 means the flags are NOT present.
    /// 1 means the flags `seq_scaling_matrix_present_flag[i]` for i values \[0, 7\] or \[0, 11\] are set.
    ///
    /// ISO/IEC-14496-10-2022 - 7.4.2.1.1
    ///
    /// Note that this crate does NOT use scaling matricies for decoding.
    /// The parse function will still read the values if passed in correctly,
    /// but this struct does not store them at this time.
    /// If you have an application that requires this feature, please reach out to us at <https://scuffle.cloud>.
    pub seq_scaling_matrix_present_flag: bool,
}

impl SpsExtended {
    /// Parses an extended SPS from a bitstream.
    /// Returns an `SpsExtended` struct.
    pub fn parse<T: io::Read>(reader: &mut BitReader<T>) -> io::Result<Self> {
        let chroma_format_idc = reader.read_exp_golomb()? as u8;
        // Defaults to false: ISO/IEC-14496-10-2022 - 7.4.2.1.1
        let mut separate_color_plane_flag = false;
        if chroma_format_idc == 3 {
            separate_color_plane_flag = reader.read_bit()?;
        }

        let bit_depth_luma_minus8 = reader.read_exp_golomb()? as u8;
        let bit_depth_chroma_minus8 = reader.read_exp_golomb()? as u8;
        let qpprime_y_zero_transform_bypass_flag = reader.read_bit()?;
        let seq_scaling_matrix_present_flag = reader.read_bit()?;

        if seq_scaling_matrix_present_flag {
            // We need to read the scaling matrices here, but we don't need them
            // for decoding, so we just skip them.
            let count = if chroma_format_idc != 3 { 8 } else { 12 };
            for i in 0..count {
                if reader.read_bit()? {
                    let size = if i < 6 { 16 } else { 64 };
                    let mut next_scale = 8;
                    for _ in 0..size {
                        let delta_scale = reader.read_signed_exp_golomb()?;
                        next_scale = (next_scale + delta_scale + 256) % 256;
                        if next_scale == 0 {
                            break;
                        }
                    }
                }
            }
        }

        Ok(SpsExtended {
            chroma_format_idc,
            separate_color_plane_flag,
            bit_depth_luma_minus8,
            bit_depth_chroma_minus8,
            qpprime_y_zero_transform_bypass_flag,
            seq_scaling_matrix_present_flag,
        })
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::io;

    use bytes::Bytes;
    use scuffle_bytes_util::BitReader;

    use crate::sps::{Sps, SpsExtended};

    #[test]
    fn test_parse_sps_insufficient_bytes_() {
        let sps = Bytes::from(vec![0xFF]);
        let result = Sps::parse(sps);

        assert!(result.is_err());
        let err = result.unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(err.to_string(), "Insufficient data: SPS must be at least 4 bytes long");
    }

    #[test]
    fn test_parse_sps_set_forbidden_bit() {
        let sps = Bytes::from(vec![
            0xFF, // forbidden bit is set
            0xFF, // dummy data
            0xFF, 0xFF,
        ]);
        let result = Sps::parse(sps);

        assert!(result.is_err());
        let err = result.unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(err.to_string(), "Forbidden zero bit is set");
    }

    #[test]
    fn test_parse_sps_invalid_nal() {
        let data = Bytes::from(vec![
            // NAL Header: forbidden_zero_bit (0) + nal_ref_idc (11) + nal_unit_type (5 = non-SPS)
            0x65, // 01100101 -> nal_unit_type = 5 (not 7, so invalid)
            0xFF, // dummy data
            0xFF, 0xFF,
        ]);
        let result = Sps::parse(data);

        assert!(result.is_err());
        let err = result.unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(err.to_string(), "NAL unit type is not SPS");
    }

    #[test]
    fn test_parse_sps_4k_60fps() {
        let sps = Bytes::from(vec![
            // NAL Header: forbidden_zero_bit (0), nal_ref_idc (11), nal_unit_type (7 = SPS)
            0x67, // Profile IDC (High Profile = 100)
            0x64, // Constraint flags and reserved bits
            0x00, // Level IDC (51)
            0x33, // Sequence Parameter Set ID, log2_max_frame_num_minus4, pic_order_cnt_type
            0xAC, 0xCA, 0x50, 0x0F, // Reserved bits and emulation prevention
            0x00, 0x10, 0xFB, 0x01, 0x10, // Frame dimensions: width = 3840, height = 2160
            0x00, 0x00, 0x03, 0x00, 0x10, 0x00, 0x00, 0x07, 0x88, 0xF1, 0x83, 0x19, 0x60,
        ]);

        let sps = Sps::parse(sps).unwrap();

        insta::assert_debug_snapshot!(sps, @r"
        Sps {
            forbidden_zero_bit: false,
            nal_ref_idc: 3,
            nal_unit_type: NALUnitType::SPS,
            profile_idc: 100,
            constraint_set0_flag: false,
            constraint_set1_flag: false,
            constraint_set2_flag: false,
            constraint_set3_flag: false,
            constraint_set4_flag: false,
            constraint_set5_flag: false,
            level_idc: 51,
            seq_parameter_set_id: 0,
            ext: Some(
                SpsExtended {
                    chroma_format_idc: 1,
                    separate_color_plane_flag: false,
                    bit_depth_luma_minus8: 0,
                    bit_depth_chroma_minus8: 0,
                    qpprime_y_zero_transform_bypass_flag: false,
                    seq_scaling_matrix_present_flag: false,
                },
            ),
            log2_max_frame_num_minus4: 0,
            pic_order_cnt_type: 0,
            max_num_ref_frames: 4,
            gaps_in_frame_num_value_allowed_flag: false,
            pic_width_in_mbs_minus1: 239,
            pic_height_in_map_units_minus1: 134,
            frame_mbs_only_flag: true,
            direct_8x8_inference_flag: true,
            frame_cropping_flag: false,
            frame_crop_left_offset: 0,
            frame_crop_right_offset: 0,
            frame_crop_top_offset: 0,
            frame_crop_bottom_offset: 0,
            width: 3840,
            height: 2160,
            vui_parameters_present_flag: true,
            video_signal_type_present_flag: false,
            color_description_present_flag: false,
            color_config: None,
            timing_info_present_flag: true,
            frame_rate: 60.0,
        }
        ");
    }

    #[test]
    fn test_parse_sps_480p_0fps() {
        let sps = Bytes::from(vec![
            // NAL Header: nal_unit_type (7 = SPS)
            0x67, // Profile IDC (Baseline = 66)
            0x42, // Constraint flags and reserved bits
            0xC0, // Level IDC (31)
            0x1F, // Sequence Parameter Set ID, log2_max_frame_num_minus4, pic_order_cnt_type
            0x8C, 0x8D, 0x40, 0x50, // Frame dimensions: width = 640, height = 480
            0x1E, 0x90, 0x0F, 0x08, 0x84, 0x6A,
        ]);

        let sps = Sps::parse(sps).unwrap();

        insta::assert_debug_snapshot!(sps, @r"
        Sps {
            forbidden_zero_bit: false,
            nal_ref_idc: 3,
            nal_unit_type: NALUnitType::SPS,
            profile_idc: 66,
            constraint_set0_flag: true,
            constraint_set1_flag: true,
            constraint_set2_flag: false,
            constraint_set3_flag: false,
            constraint_set4_flag: false,
            constraint_set5_flag: false,
            level_idc: 31,
            seq_parameter_set_id: 0,
            ext: None,
            log2_max_frame_num_minus4: 11,
            pic_order_cnt_type: 0,
            max_num_ref_frames: 1,
            gaps_in_frame_num_value_allowed_flag: false,
            pic_width_in_mbs_minus1: 39,
            pic_height_in_map_units_minus1: 29,
            frame_mbs_only_flag: true,
            direct_8x8_inference_flag: false,
            frame_cropping_flag: false,
            frame_crop_left_offset: 0,
            frame_crop_right_offset: 0,
            frame_crop_top_offset: 0,
            frame_crop_bottom_offset: 0,
            width: 640,
            height: 480,
            vui_parameters_present_flag: true,
            video_signal_type_present_flag: false,
            color_description_present_flag: false,
            color_config: None,
            timing_info_present_flag: false,
            frame_rate: 0.0,
        }
        ");
    }

    #[test]
    fn test_parse_sps_1080p_60fps_with_color_config() {
        let sps = Bytes::from(vec![
            // NAL Header: nal_unit_type (7 = SPS)
            0x67, // Profile IDC (High Profile = 100)
            0x64, // Constraint flags and reserved bits
            0x00, // Level IDC (42)
            0x2A, // Sequence Parameter Set ID, log2_max_frame_num_minus4, pic_order_cnt_type
            0xAC, 0xB2, 0x00, 0xF0, // Color configuration present
            0x04, 0x4F, 0xCB, 0x80, 0xB5, 0x01, 0x01, 0x01, 0x40,
            // Emulation prevention bytes removal and frame rate
            0x00, 0x00, 0x03, 0x00, 0x40, 0x00, 0x00, 0x1E, 0x23, 0xC6, 0x0C, 0x92,
        ]);

        let sps = Sps::parse(sps).unwrap();

        insta::assert_debug_snapshot!(sps, @r"
        Sps {
            forbidden_zero_bit: false,
            nal_ref_idc: 3,
            nal_unit_type: NALUnitType::SPS,
            profile_idc: 100,
            constraint_set0_flag: false,
            constraint_set1_flag: false,
            constraint_set2_flag: false,
            constraint_set3_flag: false,
            constraint_set4_flag: false,
            constraint_set5_flag: false,
            level_idc: 42,
            seq_parameter_set_id: 0,
            ext: Some(
                SpsExtended {
                    chroma_format_idc: 1,
                    separate_color_plane_flag: false,
                    bit_depth_luma_minus8: 0,
                    bit_depth_chroma_minus8: 0,
                    qpprime_y_zero_transform_bypass_flag: false,
                    seq_scaling_matrix_present_flag: false,
                },
            ),
            log2_max_frame_num_minus4: 0,
            pic_order_cnt_type: 2,
            max_num_ref_frames: 3,
            gaps_in_frame_num_value_allowed_flag: false,
            pic_width_in_mbs_minus1: 119,
            pic_height_in_map_units_minus1: 67,
            frame_mbs_only_flag: true,
            direct_8x8_inference_flag: true,
            frame_cropping_flag: true,
            frame_crop_left_offset: 0,
            frame_crop_right_offset: 0,
            frame_crop_top_offset: 0,
            frame_crop_bottom_offset: 4,
            width: 1920,
            height: 1080,
            vui_parameters_present_flag: true,
            video_signal_type_present_flag: true,
            color_description_present_flag: true,
            color_config: Some(
                ColorConfig {
                    video_full_range_flag: false,
                    color_primaries: 1,
                    transfer_characteristics: 1,
                    matrix_coefficients: 1,
                },
            ),
            timing_info_present_flag: true,
            frame_rate: 60.0,
        }
        ");
    }

    #[test]
    fn test_parse_sps_pic_order_cnt_type_set() {
        let sps = bytes::Bytes::from(vec![
            // NAL header, profile (66), constraint flags + reserved bits, level idc (31)
            0x67, 0x42, 0xC0, 0x1F, 0xD3, 0x58, // sps_id (0), log2_max_frame_num_minus4 (0)
            0x14, // pic_order_cnt_type (1)
            0x07, // delta_pic_order_always_zero_flag (0) and offset_for_non_ref_pic (0)
            // offset_for_top_to_bottom_field (0) and num_ref_frames... (1) and offset_for_ref_frame (0)
            0xB0,
            // max_num_ref_frames (0) and gaps_in_frame_num_value_allowed_flag (0) and begins pic_width_in_mbs_minus1 (39)
            0x1E, 0x90, // pic_width_in_mbs_minus1 encoding (39, so width = 40 * 16 = 640)
            0x0F, // pic_height_in_map_units_minus1 = 29 (so height = 30 * 16 = 480)
            0x08, // frame_mbs_only_flag = 1
            0x84, // direct_8x8_inference_flag = 1; frame_cropping_flag = 0
            0x6A, // vui_parameters_present_flag = 0; end of SPS data
        ]);

        let sps = crate::sps::Sps::parse(sps).unwrap();

        insta::assert_debug_snapshot!(sps, @r"
        Sps {
            forbidden_zero_bit: false,
            nal_ref_idc: 3,
            nal_unit_type: NALUnitType::SPS,
            profile_idc: 66,
            constraint_set0_flag: true,
            constraint_set1_flag: true,
            constraint_set2_flag: false,
            constraint_set3_flag: false,
            constraint_set4_flag: false,
            constraint_set5_flag: false,
            level_idc: 31,
            seq_parameter_set_id: 0,
            ext: None,
            log2_max_frame_num_minus4: 0,
            pic_order_cnt_type: 1,
            max_num_ref_frames: 0,
            gaps_in_frame_num_value_allowed_flag: false,
            pic_width_in_mbs_minus1: 39,
            pic_height_in_map_units_minus1: 29,
            frame_mbs_only_flag: true,
            direct_8x8_inference_flag: true,
            frame_cropping_flag: false,
            frame_crop_left_offset: 0,
            frame_crop_right_offset: 0,
            frame_crop_top_offset: 0,
            frame_crop_bottom_offset: 0,
            width: 640,
            height: 480,
            vui_parameters_present_flag: false,
            video_signal_type_present_flag: false,
            color_description_present_flag: false,
            color_config: None,
            timing_info_present_flag: false,
            frame_rate: 0.0,
        }
        ");
    }

    #[test]
    fn test_parse_sps_vui_and_interlaced() {
        let sps = bytes::Bytes::from(vec![
            // NAL header, profile idc = 66, constraint flags and reserved bits, level idc = 31
            0x67, 0x42, 0x00, 0x1F, 0xF8, // first bits of pic_width_in_mbs_minus1
            0x14, // next 8 bits of pic_width_in_mbs_minus1
            // remainder of pic_width_in_mbs_minus1 + first 7 bits of pic_height_in_map_units_minus1
            0x07,
            // last bits of pic_height_in_map_units_minus1 + flags (frame_mbs_only_flag, etc.) + VUI start bits
            0x8B, 0xFF, // aspect_ratio_idc = 255
            0x01, 0x23, // sar_width high byte (0x0123)
            0x04, 0x56, // sar_height high byte (0x0456)
            0xA0, // overscan and video signal type flags
            0xE0, // chroma loc info and timing flag (plus padding)
        ]);
        let sps = Sps::parse(sps).unwrap();

        insta::assert_debug_snapshot!(sps, @r"
        Sps {
            forbidden_zero_bit: false,
            nal_ref_idc: 3,
            nal_unit_type: NALUnitType::SPS,
            profile_idc: 66,
            constraint_set0_flag: false,
            constraint_set1_flag: false,
            constraint_set2_flag: false,
            constraint_set3_flag: false,
            constraint_set4_flag: false,
            constraint_set5_flag: false,
            level_idc: 31,
            seq_parameter_set_id: 0,
            ext: None,
            log2_max_frame_num_minus4: 0,
            pic_order_cnt_type: 0,
            max_num_ref_frames: 0,
            gaps_in_frame_num_value_allowed_flag: false,
            pic_width_in_mbs_minus1: 39,
            pic_height_in_map_units_minus1: 29,
            frame_mbs_only_flag: false,
            direct_8x8_inference_flag: true,
            frame_cropping_flag: false,
            frame_crop_left_offset: 0,
            frame_crop_right_offset: 0,
            frame_crop_top_offset: 0,
            frame_crop_bottom_offset: 0,
            width: 640,
            height: 960,
            vui_parameters_present_flag: true,
            video_signal_type_present_flag: true,
            color_description_present_flag: false,
            color_config: Some(
                ColorConfig {
                    video_full_range_flag: false,
                    color_primaries: 2,
                    transfer_characteristics: 2,
                    matrix_coefficients: 2,
                },
            ),
            timing_info_present_flag: false,
            frame_rate: 0.0,
        }
        ");
    }

    #[test]
    fn test_parse_sps_ext_chroma_format_3() {
        let sps = Bytes::from_static(&[
            0x67, 0x64, 0x00, 0x1F, // NAL/profile/constraints/level
            0x91, 0x9E, 0xF0, // chroma_format_idc=3
        ]);

        let result = Sps::parse(sps).expect("Failed to parse SPS");
        assert_eq!(result.profile_idc, 100);

        let ext = result.ext.expect("Expected SpsExtended, got None");
        assert_eq!(ext.chroma_format_idc, 3);

        assert_eq!(ext.bit_depth_luma_minus8, 0);
        assert_eq!(ext.bit_depth_chroma_minus8, 0);
    }

    #[test]
    fn test_parse_sps_ext_scaling_matrix() {
        let data = Bytes::from(vec![0x23, 0x7F, 0xFF, 0xE0, 0x00]);
        let mut reader = BitReader::new_from_slice(data);
        let ext = SpsExtended::parse(&mut reader).unwrap();

        insta::assert_debug_snapshot!(ext, @r"
        SpsExtended {
            chroma_format_idc: 3,
            separate_color_plane_flag: false,
            bit_depth_luma_minus8: 0,
            bit_depth_chroma_minus8: 0,
            qpprime_y_zero_transform_bypass_flag: false,
            seq_scaling_matrix_present_flag: true,
        }
        ");
    }

    #[test]
    fn test_parse_sps_ext_break() {
        let data = Bytes::from(vec![0x5B, 0x08, 0x80]);
        let mut reader = BitReader::new_from_slice(data);
        let ext = SpsExtended::parse(&mut reader).unwrap();

        insta::assert_debug_snapshot!(ext, @r"
        SpsExtended {
            chroma_format_idc: 1,
            separate_color_plane_flag: false,
            bit_depth_luma_minus8: 0,
            bit_depth_chroma_minus8: 0,
            qpprime_y_zero_transform_bypass_flag: false,
            seq_scaling_matrix_present_flag: true,
        }
        ");
    }
}
