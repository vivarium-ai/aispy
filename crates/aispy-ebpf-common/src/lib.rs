#![no_std]
#![deny(unsafe_code)]

mod event;

pub use event::*;

#[cfg(test)]
mod tests;
