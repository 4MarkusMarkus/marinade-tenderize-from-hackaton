// #![deny(missing_docs)]
#![forbid(unsafe_code)]

pub mod error;
pub mod processor;

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;
