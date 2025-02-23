use nutype_enum::nutype_enum;

nutype_enum! {
    /// The `AspectRatioIdc` is a nutype enum for `aspect_ratio_idc` as defined in
    /// ISO/IEC-14496-10-2022 - E.2.1 Table E-1.
    ///
    /// Values 17..=254** are reserved (should be ignored if encountered)
    /// **Value 255 (`ExtendedSar`)** indicates that the aspect ratio is specified by
    ///   additional fields (`sar_width` and `sar_height`) in the bitstream.
    ///
    /// ## Examples of aspect_ratio_idc values:
    /// - `1` => 1:1 ("square")
    /// - `4` => 16:11
    /// - `14` => 4:3
    /// - `15` => 3:2
    /// - `16` => 2:1
    pub enum AspectRatioIdc(u8) {
        /// 0: Unspecified (not used in decoding)
        Unspecified = 0,

        /// 1: 1:1 (square)
        Square = 1,

        /// 2: 12:11
        Aspect12_11 = 2,

        /// 3: 10:11
        Aspect10_11 = 3,

        /// 4: 16:11
        Aspect16_11 = 4,

        /// 5: 40:33
        Aspect40_33 = 5,

        /// 6: 24:11
        Aspect24_11 = 6,

        /// 7: 20:11
        Aspect20_11 = 7,

        /// 8: 32:11
        Aspect32_11 = 8,

        /// 9: 80:33
        Aspect80_33 = 9,

        /// 10: 18:11
        Aspect18_11 = 10,

        /// 11: 15:11
        Aspect15_11 = 11,

        /// 12: 64:33
        Aspect64_33 = 12,

        /// 13: 160:99
        Aspect160_99 = 13,

        /// 14: 4:3
        Aspect4_3 = 14,

        /// 15: 3:2
        Aspect3_2 = 15,

        /// 16: 2:1
        Aspect2_1 = 16,

        /// 17..=254: Reserved (should be ignored)
        Reserved = 17,

        /// 255: Extended SAR (use `sar_width` & `sar_height` from bitstream)
        ExtendedSar = 255
    }
}
