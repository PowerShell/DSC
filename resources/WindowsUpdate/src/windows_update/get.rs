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
        let all_updates = search_result.Updates()?;
        let count = all_updates.Count()?;

        // Process each input filter
        let mut matched_updates = Vec::new();
        
        for update_input in &update_list.updates {
            // Validate that at least one search criterion is provided
            if update_input.title.is_none() 
                && update_input.id.is_none() 
                && update_input.kb_article_ids.is_none() 
                && update_input.is_installed.is_none() 
                && update_input.update_type.is_none() 
                && update_input.msrc_severity.is_none() {
                return Err(Error::new(E_INVALIDARG, "At least one search criterion must be specified for get operation"));
            }

            // Find the update matching ALL provided criteria (logical AND)
            let mut found_update = None;
            for i in 0..count {
                let update = all_updates.get_Item(i)?;
                
                // Check title match
                if let Some(search_title) = &update_input.title {
                    let title = update.Title()?.to_string();
                    if !title.eq_ignore_ascii_case(search_title) {
                        continue; // Title doesn't match, skip this update
                    }
                }

                // Check id match
                if let Some(search_id) = &update_input.id {
                    let identity = update.Identity()?;
                    let update_id = identity.UpdateID()?.to_string();
                    if !update_id.eq_ignore_ascii_case(search_id) {
                        continue; // ID doesn't match, skip this update
                    }
                }

                // Check is_installed match
                if let Some(search_installed) = update_input.is_installed {
                    let is_installed = update.IsInstalled()?.as_bool();
                    if is_installed != search_installed {
                        continue; // Installation state doesn't match, skip this update
                    }
                }

                // Check KB article IDs match
                if let Some(search_kb_ids) = &update_input.kb_article_ids {
                    let kb_articles = update.KBArticleIDs()?;
                    let kb_count = kb_articles.Count()?;
                    let mut kb_article_ids = Vec::new();
                    for j in 0..kb_count {
                        if let Ok(kb_str) = kb_articles.get_Item(j) {
                            kb_article_ids.push(kb_str.to_string());
                        }
                    }
                    
                    // Check if all search KB IDs are present
                    let mut all_match = true;
                    for search_kb in search_kb_ids {
                        if !kb_article_ids.iter().any(|kb| kb.eq_ignore_ascii_case(search_kb)) {
                            all_match = false;
                            break;
                        }
                    }
                    if !all_match {
                        continue; // KB articles don't match, skip this update
                    }
                }

                // Check update type match
                if let Some(search_type) = &update_input.update_type {
                    use windows::Win32::System::UpdateAgent::UpdateType as WinUpdateType;
                    let ut = update.Type()?;
                    let update_type = if ut == WinUpdateType(2) {
                        crate::windows_update::types::UpdateType::Driver
                    } else {
                        crate::windows_update::types::UpdateType::Software
                    };
                    
                    if &update_type != search_type {
                        continue; // Update type doesn't match, skip this update
                    }
                }

                // Check MSRC severity match
                if let Some(search_severity) = &update_input.msrc_severity {
                    let msrc_severity = if let Ok(severity_str) = update.MsrcSeverity() {
                        match severity_str.to_string().as_str() {
                            "Critical" => Some(crate::windows_update::types::MsrcSeverity::Critical),
                            "Important" => Some(crate::windows_update::types::MsrcSeverity::Important),
                            "Moderate" => Some(crate::windows_update::types::MsrcSeverity::Moderate),
                            "Low" => Some(crate::windows_update::types::MsrcSeverity::Low),
                            _ => None,
                        }
                    } else {
                        None
                    };
                    
                    if msrc_severity.as_ref() != Some(search_severity) {
                        continue; // Severity doesn't match, skip this update
                    }
                }

                // All criteria matched - extract and store the update
                found_update = Some(extract_update_info(&update)?);
                break;
            }

            if let Some(update_info) = found_update {
                matched_updates.push(update_info);
            } else {
                // No match found for this input - construct error message and return
                let mut criteria_parts = Vec::new();
                if let Some(title) = &update_input.title {
                    criteria_parts.push(format!("title '{}'", title));
                }
                if let Some(id) = &update_input.id {
                    criteria_parts.push(format!("id '{}'", id));
                }
                if let Some(is_installed) = update_input.is_installed {
                    criteria_parts.push(format!("is_installed {}", is_installed));
                }
                if let Some(kb_ids) = &update_input.kb_article_ids {
                    criteria_parts.push(format!("kb_article_ids {:?}", kb_ids));
                }
                if let Some(update_type) = &update_input.update_type {
                    criteria_parts.push(format!("update_type {:?}", update_type));
                }
                if let Some(severity) = &update_input.msrc_severity {
                    criteria_parts.push(format!("msrc_severity {:?}", severity));
                }
                
                let criteria_str = criteria_parts.join(", ");
                let error_msg = format!("No matching update found for criteria: {}", criteria_str);
                
                // Emit JSON error to stderr
                eprintln!("{{\"error\":\"{}\"}}", error_msg);
                
                return Err(Error::new(E_FAIL, error_msg));
            }
        }

        Ok(matched_updates)
    };

    // Ensure COM is uninitialized if it was initialized
    if com_initialized {
        unsafe {
            CoUninitialize();
        }
    }

    match result {
        Ok(updates) => {
            let result = UpdateList {
                updates
            };
            serde_json::to_string(&result)
                .map_err(|e| Error::new(E_FAIL, format!("Failed to serialize output: {}", e)))
        }
        Err(e) => Err(e),
    }
}
