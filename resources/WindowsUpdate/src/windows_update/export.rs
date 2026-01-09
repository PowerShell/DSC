// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::System::Com::*,
    Win32::System::UpdateAgent::*,
};

use crate::windows_update::types::{UpdateInput, UpdateInfo, MsrcSeverity, UpdateType};

pub fn handle_export(input: &str) -> Result<String> {
    // Parse optional filter input
    let filter: UpdateInput = if input.trim().is_empty() {
        UpdateInput {
            title: None,
            id: None,
            is_installed: None,
            description: None,
            is_uninstallable: None,
            kb_article_ids: None,
            max_download_size: None,
            msrc_severity: None,
            security_bulletin_ids: None,
            update_type: None,
        }
    } else {
        serde_json::from_str(input)
            .map_err(|e| Error::new(E_INVALIDARG, format!("Failed to parse input: {}", e)))?
    };
    
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

        // Build search criteria based on filters
        let search_criteria = match filter.is_installed {
            Some(true) => "IsInstalled=1",
            Some(false) => "IsInstalled=0",
            None => "IsInstalled=0 or IsInstalled=1",
        };

        // Search for updates with optimized criteria
        let search_result = searcher.Search(&BSTR::from(search_criteria))?;

        // Get updates collection
        let updates = search_result.Updates()?;
        let count = updates.Count()?;

        // Collect all matching updates
        let mut found_updates: Vec<UpdateInfo> = Vec::new();
        for i in 0..count {
            let update = updates.get_Item(i)?;
            let title = update.Title()?.to_string();
            let identity = update.Identity()?;
            let update_id = identity.UpdateID()?.to_string();

            // Extract all update information first for filtering
            let is_installed = update.IsInstalled()?.as_bool();
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
                    WinUpdateType(2) => UpdateType::Driver,
                    _ => UpdateType::Software,
                }
            };

            // Apply all filters
            let mut matches = true;

            // Filter by title with wildcard support
            if let Some(title_filter) = &filter.title {
                matches = matches && matches_wildcard(&title, title_filter);
            }

            // Filter by id
            if let Some(id_filter) = &filter.id {
                matches = matches && update_id.eq_ignore_ascii_case(id_filter);
            }

            // Filter by description with wildcard support
            if let Some(desc_filter) = &filter.description {
                matches = matches && matches_wildcard(&description, desc_filter);
            }

            // Filter by is_uninstallable
            if let Some(uninstallable_filter) = filter.is_uninstallable {
                matches = matches && (is_uninstallable == uninstallable_filter);
            }

            // Filter by KB article IDs (match if any KB ID in the filter is present)
            if let Some(kb_filter) = &filter.kb_article_ids {
                if !kb_filter.is_empty() {
                    let kb_matches = kb_filter.iter().any(|filter_kb| {
                        kb_article_ids.iter().any(|update_kb| update_kb.eq_ignore_ascii_case(filter_kb))
                    });
                    matches = matches && kb_matches;
                }
            }

            // Filter by max_download_size (if specified, update size must be <= filter size)
            if let Some(size_filter) = filter.max_download_size {
                matches = matches && (max_download_size <= size_filter);
            }

            // Filter by MSRC severity
            if let Some(severity_filter) = &filter.msrc_severity {
                matches = matches && (msrc_severity.as_ref() == Some(severity_filter));
            }

            // Filter by security bulletin IDs (match if any bulletin ID in the filter is present)
            if let Some(bulletin_filter) = &filter.security_bulletin_ids {
                if !bulletin_filter.is_empty() {
                    let bulletin_matches = bulletin_filter.iter().any(|filter_bulletin| {
                        security_bulletin_ids.iter().any(|update_bulletin| update_bulletin.eq_ignore_ascii_case(filter_bulletin))
                    });
                    matches = matches && bulletin_matches;
                }
            }

            // Filter by update type
            if let Some(type_filter) = &filter.update_type {
                matches = matches && (&update_type == type_filter);
            }

            if matches {
                found_updates.push(UpdateInfo {
                    title,
                    is_installed,
                    description,
                    id: update_id,
                    is_uninstallable,
                    kb_article_ids,
                    max_download_size,
                    msrc_severity,
                    security_bulletin_ids,
                    update_type,
                });
            }
        }

        Ok(found_updates)
    };

    unsafe {
        CoUninitialize();
    }

    match result {
        Ok(updates) => serde_json::to_string(&updates)
            .map_err(|e| Error::new(E_FAIL, format!("Failed to serialize output: {}", e))),
        Err(e) => Err(e),
    }
}

// Helper function to match string against pattern with wildcard (*)
fn matches_wildcard(text: &str, pattern: &str) -> bool {
    let text_lower = text.to_lowercase();
    let pattern_lower = pattern.to_lowercase();
    
    // Split pattern by asterisks
    let parts: Vec<&str> = pattern_lower.split('*').collect();
    
    // If no wildcard, it's an exact match (case-insensitive)
    if parts.len() == 1 {
        return text_lower == pattern_lower;
    }
    
    // If pattern is just asterisk(s), match everything
    if parts.is_empty() {
        return true;
    }
    
    // Check if pattern starts with asterisk
    let starts_with_wildcard = pattern_lower.starts_with('*');
    // Check if pattern ends with asterisk
    let ends_with_wildcard = pattern_lower.ends_with('*');
    
    let mut pos = 0;
    
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        
        // For the first part, check if it should be at the start
        if i == 0 && !starts_with_wildcard {
            if !text_lower.starts_with(part) {
                return false;
            }
            pos = part.len();
        } else {
            // Find the part in the remaining text
            if let Some(found_pos) = text_lower[pos..].find(part) {
                pos += found_pos + part.len();
            } else {
                return false;
            }
        }
    }
    
    // For the last part, check if it should be at the end
    if !ends_with_wildcard && !parts.is_empty() {
        if let Some(last_part) = parts.last() {
            if !last_part.is_empty() && !text_lower.ends_with(last_part) {
                return false;
            }
        }
    }
    
    true
}
