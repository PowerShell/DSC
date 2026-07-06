// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Generates minimal offline registry hive files for testing.
//! Usage: cargo run --example gen_test_hives -- <output_dir>

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use dsc_lib_registry::offreg::OfflineHive;

fn encode_sz(s: &str) -> Vec<u8> {
    let wide: Vec<u16> = OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect();
    wide.iter().flat_map(|&c| c.to_le_bytes()).collect()
}

fn main() {
    let test_dir = std::env::args().nth(1).expect("Usage: gen_test_hives <output_dir>");
    let test_dir = Path::new(&test_dir);
    std::fs::create_dir_all(test_dir).unwrap();

    // Create HKLM hive with a test key and value
    let hklm = OfflineHive::create().unwrap();
    let software_key = hklm.create_key("Software\\DSCTest").unwrap();
    let data = encode_sz("TestValue");
    hklm.set_value(&software_key, "TestString", 1, &data).unwrap(); // REG_SZ = 1
    let dword_data = 42u32.to_le_bytes();
    hklm.set_value(&software_key, "TestDword", 4, &dword_data).unwrap(); // REG_DWORD = 4
    hklm.close_key(software_key);
    hklm.save(&test_dir.join("HKLM.hiv")).unwrap();
    println!("Created {}", test_dir.join("HKLM.hiv").display());

    // Create HKCU hive with a test key and value
    let hkcu = OfflineHive::create().unwrap();
    let user_key = hkcu.create_key("Software\\DSCUserTest").unwrap();
    let data2 = encode_sz("UserValue");
    hkcu.set_value(&user_key, "UserString", 1, &data2).unwrap(); // REG_SZ = 1
    hkcu.close_key(user_key);
    hkcu.save(&test_dir.join("HKCU.hiv")).unwrap();
    println!("Created {}", test_dir.join("HKCU.hiv").display());

    println!("Test hives created successfully in {}", test_dir.display());
}
