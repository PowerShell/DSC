// Copyright (C) Microsoft Corporation. All rights reserved.

//! This crate provides a set of types that abstract over OS-specific platform
//! primitives. It is focused on IO- and wait-related functionality: events,
//! timers, and polling.
//!
//! As a convenience, it also exports some OS-specific functionality and some
//! general library functionality.

#[cfg(windows)]
pub mod windows;

#[cfg(windows)]
use windows as sys;

/// Writes cryptographically random bytes to `data` using the system RNG. Panics
/// on error, since the system RNG should always be available.
pub fn random(data: &mut [u8]) {
    unsafe { sys::getrandom(data.as_mut_ptr(), data.len()) }
}
