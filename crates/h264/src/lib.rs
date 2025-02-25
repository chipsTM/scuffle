//! A pure Rust implementation of the H.264 (header only) encoder and decoder.
//!
//! This crate is designed to provide a simple and safe interface to encode and decode H.264 headers.
//!
//! ## Why do we need this?
//!
//! This crate aims to provides a simple and safe interface for h264.
//!
//! ## How is this different from other h264 crates?
//!
//! This crate is only for encoding and decoding H.264 headers.
//!
//! ## Notable features
//!
//! This crate is a completely safe implementation of encoding and decoding H.264 headers.
//!
//! We mainly use this with scuffle-mp4 and scuffle-flv to work with mp4 and flv container formats respectively.
//!
//! ## Examples
//!
//! ### Demuxing
//!
//! ```rust
//! use std::io;
//!
//! use bytes::Bytes;
//!
//! use scuffle_h264::{Sps, AVCDecoderConfigurationRecord};
//!
//! // A sample h264 bytestream to demux
//! let data = Bytes::from(b"\x01d\0\x1f\xff\xe1\0\x1dgd\0\x1f\xac\xd9A\xe0m\xf9\xe6\xa0  (\0\0\x03\0\x08\0\0\x03\x01\xe0x\xc1\x8c\xb0\x01\0\x06h\xeb\xe3\xcb\"\xc0\xfd\xf8\xf8\0".to_vec());
//!
//! // Demuxing
//! let result = AVCDecoderConfigurationRecord::demux(&mut io::Cursor::new(data.into())).unwrap();
//!
//! // Do something with it!
//!
//! // You can also access the sps bytestream and parse it:
//! let sps = Sps::demux(&result.sps[0]).unwrap();
//! ```
//!
//! For more examples, check out the tests in the source code for the demux function.
//!
//! ### Muxing
//!
//! ```rust
//! use bytes::Bytes;
//!
//! use scuffle_h264::{AVCDecoderConfigurationRecord, AvccExtendedConfig};
//!
//! let extended_config = AvccExtendedConfig {
//!     chroma_format_idc: 1,
//!     bit_depth_luma_minus8: 0,
//!     bit_depth_chroma_minus8: 0,
//!     sequence_parameter_set_ext: vec![Bytes::from_static(b"extra")],
//! };
//! let config = AVCDecoderConfigurationRecord {
//!     configuration_version: 1,
//!     profile_indication: 100,
//!     profile_compatibility: 0,
//!     level_indication: 31,
//!     length_size_minus_one: 3,
//!     sps: vec![Bytes::from_static(b"spsdata")],
//!     pps: vec![Bytes::from_static(b"ppsdata")],
//!     extended_config: Some(extended_config),
//! };
//!
//! // Creating a buffer to store the muxed bytestream
//! let mut muxed = Vec::new();
//!
//! // Muxing
//! config.mux(&mut muxed).unwrap();
//!
//! // Do something with it!
//! ```
//!
//! For more examples, check out the tests in the source code for the mux function.
//!
//! ## Status
//!
//! This crate is currently under development and is not yet stable.
//!
//! Unit tests are not yet fully implemented. Use at your own risk.
//!
//! ## License
//!
//! This project is licensed under the [MIT](./LICENSE.MIT) or [Apache-2.0](./LICENSE.Apache-2.0) license.
//! You can choose between one of them if you use this work.
//!
//! `SPDX-License-Identifier: MIT OR Apache-2.0`
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![deny(unsafe_code)]

mod config;
mod enums;
mod sps;

pub use enums::*;

pub use self::config::{AVCDecoderConfigurationRecord, AvccExtendedConfig};
pub use self::sps::{ColorConfig, Sps, SpsExtended};
