// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::System::Com::*,
    Win32::System::UpdateAgent::*,
};

use crate::windows_update::types::{UpdateInput, UpdateInfo, MsrcSeverity, UpdateType};

pub fn handle_get(input: &str) -> Result<String> {
    // Parse input
    let update_input: UpdateInput = serde_json::from_str(input)
        .map_err(|e| Error::new(E_INVALIDARG, format!("Failed to parse input: {}", e)))?;
    
    // Initialize COM
    unsafe {
        CoInitializeEx(Some(std::ptr::null()), COINIT_MULTITHREADED).ok()?;
    }

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
        let mut found_update: Option<UpdateInfo> = None;
        for i in 0..count {
            let update = updates.get_Item(i)?;
            let title = update.Title()?.to_string();
            let identity = update.Identity()?;
            let update_id = identity.UpdateID()?.to_string();

            let matches = if let Some(search_title) = &update_input.title {
                title.eq_ignore_ascii_case(search_title)
            } else if let Some(search_id) = &update_input.id {
                update_id.eq_ignore_ascii_case(search_id)
            } else {
                false
            };

            if matches {
                // Extract update information
                let is_installed = update.IsInstalled()?.as_bool();
                let description = update.Description()?.to_string();
                let id = update_id;
                let is_uninstallable = update.IsUninstallable()?.as_bool();

                // Get KB Article IDs
                let kb_articles = update.KBArticleIDs()?;
                let kb_count = kb_articles.Count()?;
                let mut kb_article_ids = Vec::new();
                for j in 0..kb_count {
                    if let Ok(kb_str) = kb_articles.get_Item(j) {
                        kb_article_ids.push(kb_str.to_string());
                    }
                }

                // Get max download size (DECIMAL type - complex to convert, using 0 for now)
                // Windows Update API returns DECIMAL which would require complex conversion
                let max_download_size = 0i64;

                // Get MSRC Severity
                let msrc_severity = if let Ok(severity_str) = update.MsrcSeverity() {
                    match severity_str.to_string().as_str() {
                        "Critical" => Some(MsrcSeverity::Critical),
                        "Important" => Some(MsrcSeverity::Important),
                        "Moderate" => Some(MsrcSeverity::Moderate),
                        "Low" => Some(MsrcSeverity::Low),
                        _ => None,
                    }
                } else {
                    None
                };

                // Get Security Bulletin IDs
                let security_bulletins = update.SecurityBulletinIDs()?;
                let bulletin_count = security_bulletins.Count()?;
                let mut security_bulletin_ids = Vec::new();
                for j in 0..bulletin_count {
                    if let Ok(bulletin_str) = security_bulletins.get_Item(j) {
                        security_bulletin_ids.push(bulletin_str.to_string());
                    }
                }

                // Determine update type
                let update_type = {
                    use windows::Win32::System::UpdateAgent::UpdateType as WinUpdateType;
                    match update.Type()? {
                        WinUpdateType(2) => UpdateType::Driver, // utDriver = 2
                        _ => UpdateType::Software,
                    }
                };

                found_update = Some(UpdateInfo {
                    title,
                    is_installed,
                    description,
                    id,
                    is_uninstallable,
                    kb_article_ids,
                    max_download_size,
                    msrc_severity,
                    security_bulletin_ids,
                    update_type,
                });
                break;
            }
        }

        found_update
    };

    unsafe {
        CoUninitialize();
    }

    match result {
        Some(update_info) => serde_json::to_string(&update_info)
            .map_err(|e| Error::new(E_FAIL, format!("Failed to serialize output: {}", e))),
        None => {
            let search_criteria = if let Some(title) = &update_input.title {
                format!("title '{}'", title)
            } else if let Some(id) = &update_input.id {
                format!("id '{}'", id)
            } else {
                "no criteria specified".to_string()
            };
            Err(Error::new(E_FAIL, format!("Update with {} not found", search_criteria)))
        }
    }
}
