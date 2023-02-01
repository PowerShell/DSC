// Copyright (C) Microsoft Corporation. All rights reserved.

#![cfg(windows)]

/// Makes a byte slice from a pointer and length, where the pointer may be null.
///
/// # Safety
///
/// The caller must ensure `data..data+len` is a valid mutable memory region.
unsafe fn c_slice<'a>(data: *mut u8, len: u32) -> &'a mut [u8] {
    if len == 0 {
        // Special case this since `data` may be null in this case, which is
        // prohibited by `from_raw_parts_mut`.
        &mut []
    } else {
        // SAFETY: the caller guarantees that `data..data + len` is valid.
        std::slice::from_raw_parts_mut(data, len as usize)
    }
}

/// Rust calls RtlGenRandom (also known as SystemFunction036) as part of hashmap
/// initialization. Redirect this to the CRNG of our choice to avoid a dependency
/// on advapi32.dll.
///
/// # Safety
///
/// - `data` must point to a buffer at least `len` bytes in size.
#[no_mangle]
pub unsafe extern "system" fn SystemFunction036(data: *mut u8, len: u32) -> u8 {
    // SAFETY: the caller guarantees that `data..data + len` is valid.
    let data = c_slice(data, len);
    pal::random(data);
    1
}

/// If a call to SystemFunction036 is marked as a dllimport, then it may be an indirect call
/// through __imp_SystemFunction036 instead.
#[no_mangle]
pub static __imp_SystemFunction036: unsafe extern "system" fn(*mut u8, u32) -> u8 = SystemFunction036;

/// Rust calls BCryptGenRandom as part of hashmap initialization. Redirect this
/// to the CRNG of our choice to avoid a dependency on bcrypt.dll.
///
/// # Safety
///
/// - `data` must point to a buffer at least `len` bytes in size.
#[no_mangle]
pub unsafe extern "system" fn BCryptGenRandom(
    algorithm: usize,
    data: *mut u8,
    len: u32,
    flags: u32,
) -> u32 {
    const BCRYPT_RNG_USE_ENTROPY_IN_BUFFER: u32 = 1; // ignored in Win8+
    const BCRYPT_USE_SYSTEM_PREFERRED_RNG: u32 = 2;
    if algorithm != 0
        || flags & !BCRYPT_RNG_USE_ENTROPY_IN_BUFFER != BCRYPT_USE_SYSTEM_PREFERRED_RNG
    {
        unimplemented!("unsupported options passed to BCryptGenRandom");
    }
    // SAFETY: the caller guarantees that `data..data + len` is valid.
    let data = c_slice(data, len);
    pal::random(data);
    0
}

/// If a call to BCryptGenRandom is marked as a dllimport, then it may be an indirect call
/// through __imp_BCryptGenRandom instead.
#[no_mangle]
pub static __imp_BCryptGenRandom: unsafe extern "system" fn(usize, *mut u8, u32, u32) -> u32 = BCryptGenRandom;
