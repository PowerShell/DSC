// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::System::Com::*,
    Win32::System::UpdateAgent::*,
};

use crate::windows_update::types::{UpdateInput, UpdateInfo, MsrcSeverity, UpdateType};

pub fn handle_set(input: &str) -> Result<String> {
    // Parse input
    let update_input: UpdateInput = serde_json::from_str(input)
        .map_err(|e| Error::new(E_INVALIDARG, format!("Failed to parse input: {}", e)))?;
    
    // Initialize COM
    unsafe {
        CoInitializeEx(Some(std::ptr::null()), COINIT_MULTITHREADED).ok()?;
    }

    let result = unsafe {
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
        let mut found_update: Option<(IUpdate, UpdateInfo)> = None;
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
                let is_installed = update.IsInstalled()?.as_bool();
                
                // If already installed, return current state without installing
                if is_installed {
                    let description = update.Description()?.to_string();
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

                    let max_download_size = 0i64;

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

                    let security_bulletins = update.SecurityBulletinIDs()?;
                    let bulletin_count = security_bulletins.Count()?;
                    let mut security_bulletin_ids = Vec::new();
                    for j in 0..bulletin_count {
                        if let Ok(bulletin_str) = security_bulletins.get_Item(j) {
                            security_bulletin_ids.push(bulletin_str.to_string());
                        }
                    }

                    let update_type = {
                        use windows::Win32::System::UpdateAgent::UpdateType as WinUpdateType;
                        match update.Type()? {
                            WinUpdateType(2) => UpdateType::Driver,
                            _ => UpdateType::Software,
                        }
                    };

                    let info = UpdateInfo {
                        title,
                        is_installed: true,
                        description,
                        id: update_id,
                        is_uninstallable,
                        kb_article_ids,
                        max_download_size,
                        msrc_severity,
                        security_bulletin_ids,
                        update_type,
                    };

                    return Ok(serde_json::to_string(&info)
                        .map_err(|e| Error::new(E_FAIL, format!("Failed to serialize output: {}", e)))?);
                }
                
                // Not installed - proceed with installation
                found_update = Some((update.clone(), UpdateInfo {
                    title,
                    is_installed: false,
                    description: String::new(),
                    id: update_id,
                    is_uninstallable: false,
                    kb_article_ids: Vec::new(),
                    max_download_size: 0,
                    msrc_severity: None,
                    security_bulletin_ids: Vec::new(),
                    update_type: UpdateType::Software,
                }));
                break;
            }
        }

        if let Some((update, mut update_info)) = found_update {
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
                // Check if download was successful (orcSucceeded = 2)
                if download_result.ResultCode()? != OperationResultCode(2) {
                    return Err(Error::new(E_FAIL, "Failed to download update"));
                }
            }

            // Install the update
            let installer = update_session.CreateUpdateInstaller()?;
            installer.SetUpdates(&updates_to_install)?;
            let install_result = installer.Install()?;
            
            use windows::Win32::System::UpdateAgent::OperationResultCode;
            // Check if installation was successful (orcSucceeded = 2)
            if install_result.ResultCode()? != OperationResultCode(2) {
                return Err(Error::new(E_FAIL, "Failed to install update"));
            }

            // Update the info to reflect installed state
            update_info.is_installed = true;
            
            // Get full details now that it's installed
            let description = update.Description()?.to_string();
            let is_uninstallable = update.IsUninstallable()?.as_bool();

            let kb_articles = update.KBArticleIDs()?;
            let kb_count = kb_articles.Count()?;
            let mut kb_article_ids = Vec::new();
            for j in 0..kb_count {
                if let Ok(kb_str) = kb_articles.get_Item(j) {
                    kb_article_ids.push(kb_str.to_string());
                }
            }

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

            let security_bulletins = update.SecurityBulletinIDs()?;
            let bulletin_count = security_bulletins.Count()?;
            let mut security_bulletin_ids = Vec::new();
            for j in 0..bulletin_count {
                if let Ok(bulletin_str) = security_bulletins.get_Item(j) {
                    security_bulletin_ids.push(bulletin_str.to_string());
                }
            }

            let update_type = {
                use windows::Win32::System::UpdateAgent::UpdateType as WinUpdateType;
                match update.Type()? {
                    WinUpdateType(2) => UpdateType::Driver,
                    _ => UpdateType::Software,
                }
            };

            update_info.description = description;
            update_info.is_uninstallable = is_uninstallable;
            update_info.kb_article_ids = kb_article_ids;
            update_info.msrc_severity = msrc_severity;
            update_info.security_bulletin_ids = security_bulletin_ids;
            update_info.update_type = update_type;

            Ok(update_info)
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

    unsafe {
        CoUninitialize();
    }

    match result {
        Ok(update_info) => serde_json::to_string(&update_info)
            .map_err(|e| Error::new(E_FAIL, format!("Failed to serialize output: {}", e))),
        Err(e) => Err(e),
    }
}
