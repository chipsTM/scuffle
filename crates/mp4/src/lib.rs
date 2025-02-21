#![deny(missing_docs)]
#![deny(unsafe_code)]

mod boxes;

pub mod codec;

pub use boxes::{BoxType, DynBox, header, types};

#[cfg(test)]
mod tests;
