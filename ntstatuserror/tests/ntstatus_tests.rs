// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

extern crate ntstatuserror;
use ntapi::winapi::shared::ntstatus::{STATUS_OBJECT_NAME_NOT_FOUND};
use ntstatuserror::NtStatusError;

#[cfg(test)]

#[test]
fn test_from_ntstatus() {
    let error = NtStatusError::new(STATUS_OBJECT_NAME_NOT_FOUND, "Test");
    assert_eq!(error.status, ntstatuserror::NtStatusErrorKind::ObjectNameNotFound);
    assert_eq!(error.message, "Test".to_string());
}
