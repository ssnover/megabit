#![cfg_attr(not(feature = "with-std"), no_std)]
#![allow(unsafe_op_in_unsafe_fn)]

pub mod flash;
pub mod image;
pub mod nvs;

pub use flash::FlashStorage;
