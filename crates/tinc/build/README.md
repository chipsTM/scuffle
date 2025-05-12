# tinc-build

> [!WARNING]  
> This crate is under active development and may not be stable.

[![crates.io](https://img.shields.io/crates/v/tinc-build.svg)](https://crates.io/crates/tinc-build) [![docs.rs](https://img.shields.io/docsrs/tinc-build)](https://docs.rs/tinc-build)

---

The code generator for [`tinc`](https://crates.io/crates/tinc). 

## Usage

In your `build.rs`:

```rust
fn main() {
    tinc_build::Config::prost()
        .compile_protos(&["proto/test.proto"], &["proto"])
        .unwrap();
}
```

## Status

This crate is currently under development and is not yet stable.

Unit tests are not yet fully implemented. Use at your own risk.

## License

This project is licensed under the [MIT](./LICENSE.MIT) or [Apache-2.0](./LICENSE.Apache-2.0) license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
