// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::System::Com::*,
    Win32::System::UpdateAgent::*,
};

use crate::windows_update::types::{UpdateList, UpdateInfo, extract_update_info};

pub fn handle_set(input: &str) -> Result<String> {
    // Parse input as UpdateList
    let update_list: UpdateList = serde_json::from_str(input)
        .map_err(|e| Error::new(E_INVALIDARG, format!("Failed to parse input: {}", e)))?;
    
    if update_list.updates.is_empty() {
        return Err(Error::new(E_INVALIDARG, "Updates array cannot be empty for set operation"));
    }

    // Get the first filter
    let update_input = &update_list.updates[0];
    
    // Validate that at least one search criterion is provided
    if update_input.title.is_none() && update_input.id.is_none() {
        return Err(Error::new(E_INVALIDARG, "At least one of 'title' or 'id' must be specified for set operation"));
    }
    
    // Initialize COM
    let com_initialized = unsafe {
        CoInitializeEx(Some(std::ptr::null()), COINIT_MULTITHREADED).is_ok()
    };

    let result: Result<UpdateInfo> = unsafe {
        // Create update session
        let update_session: IUpdateSession = CoCreateInstance(
            &UpdateSession,
            None,
            CLSCTX_INPROC_SERVER,
        )?;

        // Create update searcher
        let searcher = update_session.CreateUpdateSearcher()?;

        // Search for all updates (installed and not installed)
        let search_result = searcher.Search(&BSTR::from("IsInstalled=0 or IsInstalled=1"))?;

        // Get updates collection
        let updates = search_result.Updates()?;
        let count = updates.Count()?;

        // Find the update by title or id
        let mut found_update: Option<(IUpdate, bool)> = None;
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
            let matches = title_match && id_match;

            if matches {
                let is_installed = update.IsInstalled()?.as_bool();
                found_update = Some((update.clone(), is_installed));
                break;
            }
        }

        if let Some((update, is_installed)) = found_update {
            // Extract info regardless of whether we need to install
            if is_installed {
                // Already installed, just return current state
                extract_update_info(&update)
            } else {
                // Not installed - proceed with installation
                // Create update collection for download/install
                let updates_to_install: IUpdateCollection = CoCreateInstance(
                    &UpdateCollection,
                    None,
                    CLSCTX_INPROC_SERVER,
                )?;
                updates_to_install.Add(&update)?;

                // Download the update if needed
                if !update.IsDownloaded()?.as_bool() {
                    let downloader = update_session.CreateUpdateDownloader()?;
                    downloader.SetUpdates(&updates_to_install)?;
                    let download_result = downloader.Download()?;
                    
                    use windows::Win32::System::UpdateAgent::OperationResultCode;
                    let result_code = download_result.ResultCode()?;
                    // Check if download was successful (orcSucceeded = 2)
                    if result_code != OperationResultCode(2) {
                        let hresult = download_result.HResult()?;
                        return Err(Error::new(HRESULT(hresult), format!("Failed to download update. Result code: {}", result_code.0)));
                    }
                }

                // Install the update
                let installer = update_session.CreateUpdateInstaller()?;
                installer.SetUpdates(&updates_to_install)?;
                let install_result = installer.Install()?;
                
                use windows::Win32::System::UpdateAgent::OperationResultCode;
                let result_code = install_result.ResultCode()?;
                // Check if installation was successful (orcSucceeded = 2)
                if result_code != OperationResultCode(2) {
                    let hresult = install_result.HResult()?;
                    return Err(Error::new(HRESULT(hresult), format!("Failed to install update. Result code: {}", result_code.0)));
                }
                
                // Get full details now that it's installed
                extract_update_info(&update)
            }
        } else {
            let search_criteria = if let Some(title) = &update_input.title {
                format!("title '{}'", title)
            } else if let Some(id) = &update_input.id {
                format!("id '{}'", id)
            } else {
                "no criteria specified".to_string()
            };
            Err(Error::new(E_FAIL, format!("Update with {} not found", search_criteria)))
        }
    };

    // Ensure COM is uninitialized if it was initialized
    if com_initialized {
        unsafe {
            CoUninitialize();
        }
    }

    match result {
        Ok(update_info) => {
            let results = UpdateList {
                updates: vec![update_info]
            };
            serde_json::to_string(&results)
                .map_err(|e| Error::new(E_FAIL, format!("Failed to serialize output: {}", e)))
        }
        Err(e) => Err(e),
    }
}
