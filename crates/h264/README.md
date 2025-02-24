# scuffle-h264

> [!WARNING]  
> This crate is under active development and may not be stable.

[![crates.io](https://img.shields.io/crates/v/scuffle-h264.svg)](https://crates.io/crates/scuffle-h264) [![docs.rs](https://img.shields.io/docsrs/scuffle-h264)](https://docs.rs/scuffle-h264)

---

A pure Rust implementation of the H.264 encoder and decoder.

This crate is designed to provide a simple and safe interface to encode and decode H.264 headers.

## Why do we need this?

This crate aims to provides a simple and safe interface for h264.

## How is this different from other h264 crates?

There are currently a handful of other active crates:

- Retina
  - This crate is heavily focused towards being as lightweight as possible since it's meant for security cameras. The downside of this is that it lacks some SPS fields. It is also lacking some documentation for the specific SPS fields.
- less-avc
  - This crate is focused on being a simple pure Rust implementation of H.264 but with only a subset of its features. IT lacks 4:4:4 support and some documentation for the specific SPS fields.
- openh264-rs
  - This crate is a set of bindings around the C implementation of H.264 (openh264). As a result, the crate contains a lot of unsafe code since it's just a lightweight rust wrapper.

## Notable features

This crate is a completely safe implementation of H264 encoding and decoding, which means there is no unsafe code!

When combined with [scuffle-expgolomb](https://crates.io/crates/scuffle-expgolomb) and [scuffle-bytes-util](https://crates.io/crates/scuffle-bytes-util), working with h264 has never been easier!

## Examples

### Writing an SPS for 4k@144fps:
```rust
use scuffle_h264::Sps;
use scuffle_bytes_util::BitWriter;
use scuffle_expgolomb::BitWriterExpGolombExt;

// Create a
let mut sps = Vec::new();
let mut writer = BitWriter::new(&mut sps);

// forbidden zero bit must be unset
let _ = writer.write_bit(false);
// nal_ref_idc is 0
let _ = writer.write_bits(0, 2);
// nal_unit_type must be 7
let _ = writer.write_bits(7, 5);

// profile_idc = 100
let _ = writer.write_bits(100, 8);
// constraint_setn_flags all false
let _ = writer.write_bits(0, 8);
// level_idc = 0
let _ = writer.write_bits(0, 8);

// seq_parameter_set_id is expg
let _ = writer.write_exp_golomb(0);

// branch to sps ext
// chroma_format_idc is expg
let _ = writer.write_exp_golomb(0);
// bit_depth_luma_minus8 is expg
let _ = writer.write_exp_golomb(0);
// bit_depth_chroma_minus8 is expg
let _ = writer.write_exp_golomb(0);
// qpprime
let _ = writer.write_bit(false);
// seq_scaling_matrix_present_flag
let _ = writer.write_bit(false);

// return to sps
// log2_max_frame_num_minus4 is expg
let _ = writer.write_exp_golomb(0);
// pic_order_cnt_type is expg
let _ = writer.write_exp_golomb(0);
// log2_max_pic_order_cnt_lsb_minus4 is expg
let _ = writer.write_exp_golomb(0);

// max_num_ref_frames is expg
let _ = writer.write_exp_golomb(0);
// gaps_in_frame_num_value_allowed_flag
let _ = writer.write_bit(false);
// 3840 width:
// 3840 = (p + 1) * 16 - 2 * offset1 - 2 * offset2
// we set offset1 and offset2 to both be 0 later
// 3840 = (p + 1) * 16
// p = 239
let _ = writer.write_exp_golomb(239);
// we want 2160 height:
// 2160 = ((2 - m) * (p + 1) * 16) - 2 * offset1 - 2 * offset2
// we set offset1 and offset2 to both be 0 later
// m is frame_mbs_only_flag which we set to 1 later
// 2160 = (2 - 1) * (p + 1) * 16
// 2160 = (p + 1) * 16
// p = 134
let _ = writer.write_exp_golomb(134);

// frame_mbs_only_flag
let _ = writer.write_bit(true);

// direct_8x8_inference_flag
let _ = writer.write_bit(false);
// frame_cropping_flag
let _ = writer.write_bit(false);

// vui_parameters_present_flag
let _ = writer.write_bit(true);

// enter vui to set the framerate
// aspect_ratio_info_present_flag
let _ = writer.write_bit(true);
// we want square (1:1) for 16:9 for 4k w/o overscan
// aspect_ratio_idc
let _ = writer.write_bits(1, 8);

// overscan_info_present_flag
let _ = writer.write_bit(true);
// we dont want overscan
// overscan_appropriate_flag
let _ = writer.write_bit(false);

// video_signal_type_present_flag
let _ = writer.write_bit(false);
// chroma_loc_info_present_flag
let _ = writer.write_bit(false);

// timing_info_present_flag
let _ = writer.write_bit(true);
// we can set this to 100 for example
// num_units_in_tick is a u32
let _ = writer.write_bits(100, 32);
// fps = time_scale / (2 * num_units_in_tick)
// since we want 144 fps:
// 144 = time_scale / (2 * 100)
// 28800 = time_scale
// time_scale is a u32
let _ = writer.write_bits(28800, 32);
let _ = writer.finish();

// Now result contains a complete SPS.
let result = Sps::parse(sps.into()).unwrap();
```
For more SPS examples, check out the tests in the source code for SPS.

### Demuxing

```rust
use std::io::{self, Write};

use bytes::Bytes;
use scuffle_bytes_util::BitWriter;

use scuffle_h264::{Sps, AVCDecoderConfigurationRecord};

let mut data = Vec::new();
let mut writer = BitWriter::new(&mut data);

// configuration_version
let _ = writer.write_bits(1, 8);
// profile_indication
let _ = writer.write_bits(100, 8);
// profile_compatibility
let _ = writer.write_bits(0, 8);
// level_indication
let _ = writer.write_bits(31, 8);
// length_size_minus_one
let _ = writer.write_bits(3, 8);

// num_of_sequence_parameter_sets
let _ = writer.write_bits(1, 8);
// sps_length
let _ = writer.write_bits(29, 16);
// You can even pass an SPS as a byte string!
// SPS
let _ = writer.write_all(b"gd\0\x1f\xac\xd9A\xe0m\xf9\xe6\xa0  (\0\0\x03\0\x08\0\0\x03\x01\xe0x\xc1\x8c\xb0");

// num_of_picture_parameter_sets
let _ = writer.write_bits(1, 8);
// pps_length
let _ = writer.write_bits(6, 16);
// You can also pass a PPS as a byte string!
// pps
let _ = writer.write_all(b"h\xeb\xe3\xcb\"\xc0\x00\x00");

// chroma_format_idc
let _ = writer.write_bits(1, 8);
// bit_depth_luma_minus8
let _ = writer.write_bits(0, 8);
// bit_depth_chroma_minus8
let _ = writer.write_bits(0, 8);
// number_of_sequence_parameter_set_ext
let _ = writer.write_bits(0, 8);
let _ = writer.finish();

// A demuxed config.
let result = AVCDecoderConfigurationRecord::demux(&mut io::Cursor::new(data.into())).unwrap();

// Do something with it!

// You can also access the sps bytestream and parse it:
let sps = &result.sps[0];
let sps = Sps::parse(sps.clone()).unwrap();
```

For more examples, check out the tests in the source code for the demux function.

### Muxing

```rust
use std::io::{self, Write};

use bytes::Bytes;
use scuffle_bytes_util::BitWriter;

use scuffle_h264::{Sps, AVCDecoderConfigurationRecord};

// Making a bytestream to demux
let data = Bytes::from(b"\x01d\0\x1f\xff\xe1\0\x1dgd\0\x1f\xac\xd9A\xe0m\xf9\xe6\xa0  (\0\0\x03\0\x08\0\0\x03\x01\xe0x\xc1\x8c\xb0\x01\0\x06h\xeb\xe3\xcb\"\xc0\xfd\xf8\xf8\0".to_vec());
// Demuxing
let config = AVCDecoderConfigurationRecord::demux(&mut io::Cursor::new(data.clone())).unwrap();

// Creating a buffer to store the muxed bytestream
let mut muxed = Vec::new();
// Muxing
config.mux(&mut muxed).unwrap();
// Do something!
```

For more examples, check out the tests in the source code for the mux function.

## Status

This crate is currently under development and is not yet stable.

Unit tests are not yet fully implemented. Use at your own risk.

## License

This project is licensed under the [MIT](./LICENSE.MIT) or [Apache-2.0](./LICENSE.Apache-2.0) license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
