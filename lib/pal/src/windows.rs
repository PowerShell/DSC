// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//#[link(name = "ext-ms-win-cng-rng-l1-1-0")]
extern "C" {
    fn ProcessPrng(data: *mut u8, len: usize) -> u32;
}

/// # Safety
/// 
/// This function is unsafe because it dereferences a raw pointer.
/// 
/// # Panics
///
/// Will panic if the api returns 0
pub unsafe fn getrandom(data: *mut u8, len: usize) {
    assert!((ProcessPrng(data, len) != 0), "ProcessPrng failed");
}
