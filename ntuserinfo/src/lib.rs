use ntapi::ntpsapi::NtCurrentProcess;
use ntapi::ntrtl::{RtlConvertSidToUnicodeString};
use ntapi::ntseapi::{self};
use ntapi::winapi::ctypes::c_void;
use ntapi::winapi::shared::ntdef::{HANDLE, NT_SUCCESS, NTSTATUS, UNICODE_STRING};
use ntapi::winapi::shared::ntstatus::{STATUS_BUFFER_TOO_SMALL};
use ntapi::winapi::um::winnt::{SID, TOKEN_QUERY, TOKEN_USER, TOKEN_QUERY_SOURCE, TokenUser};
use std::ptr::null_mut;

use ntstatuserror::{NtStatusError};

const MAX_SID_LENGTH: u16 = 256;

/// Struct representing the curret user.
pub struct NtCurrentUserInfo {
    /// The SID of the current user.
    pub sid: String,
}

impl NtCurrentUserInfo {
    /// Create a new NtCurrentUserInfo.
    /// 
    /// # Example
    /// 
    /// ```
    /// # use ntuserinfo::NtCurrentUserInfo;
    /// let user_info = NtCurrentUserInfo::new();
    /// assert!(user_info.is_ok());
    /// let user_info = user_info.unwrap();
    /// assert!(user_info.sid.len() > 0);
    /// ```
    pub fn new() -> Result<Self, NtStatusError> {
        let mut token: HANDLE = null_mut();
        let mut status: NTSTATUS = unsafe {
            ntseapi::NtOpenProcessToken(
                NtCurrentProcess,
                TOKEN_QUERY | TOKEN_QUERY_SOURCE,
                &mut token
            )
        };

        if !NT_SUCCESS(status) {
            return Err(NtStatusError::new(status, "Failed to open process token"));
        }

        let mut token_information: Vec<u8> = vec![0; 0];
        let mut result_length: u32 = 0;
        status = unsafe {
            ntseapi::NtQueryInformationToken(
                token,
                TokenUser,
                token_information.as_mut_ptr() as *mut c_void,
                token_information.len() as u32,
                &mut result_length
            )
        };

        if status != STATUS_BUFFER_TOO_SMALL {
            return Err(NtStatusError::new(status, "Failed to query token information for size"));
        }

        token_information.resize(result_length as usize, 0);
        status = unsafe {
            ntseapi::NtQueryInformationToken(
                token,
                TokenUser,
                token_information.as_mut_ptr() as *mut c_void,
                token_information.len() as u32,
                &mut result_length
            )
        };

        if !NT_SUCCESS(status) {
            return Err(NtStatusError::new(status, "Failed to query token information"));
        }

        let token_user: *const TOKEN_USER = token_information.as_ptr() as *const TOKEN_USER;
        let mut sid_string_buffer: Vec<u16> = vec![0; MAX_SID_LENGTH as usize];
        let mut sid_string: UNICODE_STRING = UNICODE_STRING {
            Length: 0,
            MaximumLength: MAX_SID_LENGTH,
            Buffer: sid_string_buffer.as_mut_ptr()
        };

        status = unsafe {
            RtlConvertSidToUnicodeString(
                &mut sid_string,
                (*token_user).User.Sid as *const SID as *mut c_void,
                0
            )
        };

        if !NT_SUCCESS(status) {
            return Err(NtStatusError::new(status, "Failed to convert SID to string"));
        }

        let sid_string = unsafe {
            String::from_utf16(std::slice::from_raw_parts(
                sid_string.Buffer,
                sid_string.Length as usize / 2
            )).unwrap()
        };

        Ok(
            NtCurrentUserInfo {
                sid: sid_string
            }
        )
    }
}
