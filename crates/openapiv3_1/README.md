# openapiv3_1

> [!WARNING]  
> This crate is under active development and may not be stable.

[![crates.io](https://img.shields.io/crates/v/openapiv3_1.svg)](https://crates.io/crates/openapiv3_1) [![docs.rs](https://img.shields.io/docsrs/openapiv3_1)](https://docs.rs/openapiv3_1)

---

Rust implementation of OpenAPI Spec v3.1.x

A lof the code was taken from [`utoipa`](https://crates.io/crates/utoipa).

The main difference is the full JSON Schema 2020-12 Definitions.

## Features

- [`debug`]: Enables `derive(Debug)` on all the types.
- [`yaml`]: Enables `to_yaml` function.

## Alternatives

- [`openapiv3`](https://crates.io/crates/openapiv3): Implements the openapi v3.0.x spec, does not implement full json schema draft 2020-12 spec.
- [`utoipa`](https://crates.io/crates/utoipa): A fully fletched openapi-type-generator implementing some of the v3.1.x spec.
- [`schemars`](https://crates.io/crates/schemars): A fully fletched jsonschema-type-generator implementing some of the json schema draft 2020-12 spec.

## Status

This crate is currently under development and is not yet stable.

Unit tests are not yet fully implemented. Use at your own risk.

## License

This project is licensed under the [MIT](./LICENSE.MIT) or [Apache-2.0](./LICENSE.Apache-2.0) license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
