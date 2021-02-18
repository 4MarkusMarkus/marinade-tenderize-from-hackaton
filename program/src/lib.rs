// #![deny(missing_docs)]
#![forbid(unsafe_code)]

pub mod error;
pub mod processor;
pub mod instruction;

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;
