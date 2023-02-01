// Copyright (C) Microsoft Corporation. All rights reserved.

#![cfg(windows)]

//#[link(name = "ext-ms-win-cng-rng-l1-1-0")]
extern "C" {
    fn ProcessPrng(data: *mut u8, len: usize) -> u32;
}

/// # Safety
///
/// TODO
pub unsafe fn getrandom(data: *mut u8, len: usize) {
    if ProcessPrng(data, len) == 0 {
        panic!("ProcessPrng failed");
    }
}
