// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

extern crate ntuserinfo;
use ntuserinfo::NtCurrentUserInfo;

#[cfg(test)]

#[test]
fn test_get_current_user_sid() {
    let user = NtCurrentUserInfo::new();
    assert!(user.is_ok());
    let user = user.unwrap();
    assert!(user.sid.starts_with("S-"));
    // validate it's in the form of a SID
    assert!(user.sid.split('-').count() > 2);
    // validate the last part is a number
    assert!(user.sid.split('-').last().unwrap().parse::<u32>().is_ok());
}
