// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::System::Com::*,
    Win32::System::UpdateAgent::*,
};

use crate::windows_update::types::{UpdateList, extract_update_info};

pub fn handle_get(input: &str) -> Result<String> {
    // Parse input as UpdateList
    let update_list: UpdateList = serde_json::from_str(input)
        .map_err(|e| Error::new(E_INVALIDARG, format!("Failed to parse input: {}", e)))?;
    
    if update_list.updates.is_empty() {
        return Err(Error::new(E_INVALIDARG, "Updates array cannot be empty for get operation"));
    }

    // Get the first filter
    let update_input = &update_list.updates[0];
    
    // Validate that at least one search criterion is provided
    if update_input.title.is_none() && update_input.id.is_none() {
        return Err(Error::new(E_INVALIDARG, "At least one of 'title' or 'id' must be specified for get operation"));
    }
    
    // Initialize COM
    let com_initialized = unsafe {
        CoInitializeEx(Some(std::ptr::null()), COINIT_MULTITHREADED).is_ok()
    };

    let result = unsafe {
        // Create update session using the proper interface
        let update_session: IUpdateSession = CoCreateInstance(
            &UpdateSession,
            None,
            CLSCTX_INPROC_SERVER,
        )?;

        // Create update searcher
        let searcher = update_session.CreateUpdateSearcher()?;

        // Search for updates
        let search_result = searcher.Search(&BSTR::from("IsInstalled=0 or IsInstalled=1"))?;

        // Get updates collection
        let updates = search_result.Updates()?;
        let count = updates.Count()?;

        // Find the update by title or id
        let mut found_update = None;
        for i in 0..count {
            let update = updates.get_Item(i)?;
            let title = update.Title()?.to_string();
            let identity = update.Identity()?;
            let update_id = identity.UpdateID()?.to_string();

            let title_match = if let Some(search_title) = &update_input.title {
                title.eq_ignore_ascii_case(search_title)
            } else {
                true // No title filter, so it matches
            };

            let id_match = if let Some(search_id) = &update_input.id {
                update_id.eq_ignore_ascii_case(search_id)
            } else {
                true // No id filter, so it matches
            };

            // Both must match if both are provided
            if title_match && id_match {
                found_update = Some(extract_update_info(&update)?);
                break;
            }
        }

        Ok(found_update)
    };

    // Ensure COM is uninitialized if it was initialized
    if com_initialized {
        unsafe {
            CoUninitialize();
        }
    }

    match result {
        Ok(Some(update_info)) => {
            let result = UpdateList {
                updates: vec![update_info]
            };
            serde_json::to_string(&result)
                .map_err(|e| Error::new(E_FAIL, format!("Failed to serialize output: {}", e)))
        }
        Ok(None) => {
            let search_criteria = if let Some(title) = &update_input.title {
                format!("title '{}'", title)
            } else if let Some(id) = &update_input.id {
                format!("id '{}'", id)
            } else {
                "no criteria specified".to_string()
            };
            Err(Error::new(E_FAIL, format!("Update with {} not found", search_criteria)))
        }
        Err(e) => Err(e),
    }
}
