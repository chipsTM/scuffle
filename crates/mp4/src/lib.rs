mod boxes;

pub mod codec;

pub use boxes::{BoxType, DynBox, header, types};

#[cfg(test)]
mod tests;
