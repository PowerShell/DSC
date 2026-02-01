// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::System::Com::*,
    Win32::System::UpdateAgent::*,
};

use std::collections::HashSet;
use crate::windows_update::types::{UpdateList, UpdateInfo, extract_update_info};

pub fn handle_export(input: &str) -> Result<String> {
    // Parse optional filter input as UpdateList
    let update_list: UpdateList = if input.trim().is_empty() {
        UpdateList {
            metadata: None,
            updates: vec![UpdateInfo {
                description: None,
                id: None,
                installation_behavior: None,
                is_installed: None,
                is_uninstallable: None,
                kb_article_ids: None,
                recommended_hard_disk_space: None,
                msrc_severity: None,
                security_bulletin_ids: None,
                title: None,
                update_type: None,
            }]
        }
    } else {
        serde_json::from_str(input)
            .map_err(|e| Error::new(E_INVALIDARG.into(), t!("export.failedParseInput", err = e.to_string()).to_string()))?
    };
    
    let filters = &update_list.updates;
    
    // Initialize COM
    let com_initialized = unsafe {
        CoInitializeEx(Some(std::ptr::null()), COINIT_MULTITHREADED).is_ok()
    };

    let result = unsafe {
        // Create update session
        let update_session: IUpdateSession = CoCreateInstance(
            &UpdateSession,
            None,
            CLSCTX_INPROC_SERVER,
        )?;

        // Create update searcher
        let searcher = update_session.CreateUpdateSearcher()?;

        // Use the broadest search criteria to get all updates once
        // We'll filter in-memory for each filter in the array
        let search_result = searcher.Search(&BSTR::from("IsInstalled=0 or IsInstalled=1"))?;

        // Get updates collection
        let updates = search_result.Updates()?;
        let count = updates.Count()?;

        // Use HashSet to track unique update IDs (for OR logic across filters)
        let mut matched_update_ids: HashSet<String> = HashSet::new();
        let mut all_found_updates: Vec<UpdateInfo> = Vec::new();

        // Process each filter in the array (OR logic between filters)
        for (filter_index, filter) in filters.iter().enumerate() {
            let mut filter_found_match = false;
            
            // Collect matching updates for this specific filter
            for i in 0..count {
                let update = updates.get_Item(i)?;
                let identity = update.Identity()?;
                let update_id = identity.UpdateID()?.to_string();

                // Extract all update information for filtering
                let update_info = extract_update_info(&update)?;

                // Apply all filters (AND logic within a single filter)
                let mut matches = true;

                // Filter by is_installed
                if let Some(installed_filter) = filter.is_installed {
                    matches = matches && (update_info.is_installed == Some(installed_filter));
                }

                // Filter by title with wildcard support
                if let Some(title_filter) = &filter.title {
                    if let Some(ref title) = update_info.title {
                        matches = matches && matches_wildcard(title, title_filter);
                    } else {
                        matches = false;
                    }
                }

                // Filter by id
                if let Some(id_filter) = &filter.id {
                    if let Some(ref id) = update_info.id {
                        matches = matches && id.eq_ignore_ascii_case(id_filter);
                    } else {
                        matches = false;
                    }
                }

                // Filter by description with wildcard support
                if let Some(desc_filter) = &filter.description {
                    if let Some(ref description) = update_info.description {
                        matches = matches && matches_wildcard(description, desc_filter);
                    } else {
                        matches = false;
                    }
                }

                // Filter by is_uninstallable
                if let Some(uninstallable_filter) = filter.is_uninstallable {
                    matches = matches && (update_info.is_uninstallable == Some(uninstallable_filter));
                }

                // Filter by KB article IDs (match if any KB ID in the filter is present)
                if let Some(kb_filter) = &filter.kb_article_ids {
                    if !kb_filter.is_empty() {
                        if let Some(ref kb_article_ids) = update_info.kb_article_ids {
                            let kb_matches = kb_filter.iter().any(|filter_kb| {
                                kb_article_ids.iter().any(|update_kb| update_kb.eq_ignore_ascii_case(filter_kb))
                            });
                            matches = matches && kb_matches;
                        } else {
                            matches = false;
                        }
                    }
                }

                // Filter by recommended_hard_disk_space (if specified, update space must be >= filter space)
                if let Some(space_filter) = filter.recommended_hard_disk_space {
                    if let Some(recommended_hard_disk_space) = update_info.recommended_hard_disk_space {
                        matches = matches && (recommended_hard_disk_space >= space_filter);
                    } else {
                        matches = false;
                    }
                }

                // Filter by MSRC severity
                if let Some(severity_filter) = &filter.msrc_severity {
                    matches = matches && (update_info.msrc_severity.as_ref() == Some(severity_filter));
                }

                // Filter by security bulletin IDs (match if any bulletin ID in the filter is present)
                if let Some(bulletin_filter) = &filter.security_bulletin_ids {
                    if !bulletin_filter.is_empty() {
                        if let Some(ref security_bulletin_ids) = update_info.security_bulletin_ids {
                            let bulletin_matches = bulletin_filter.iter().any(|filter_bulletin| {
                                security_bulletin_ids.iter().any(|update_bulletin| update_bulletin.eq_ignore_ascii_case(filter_bulletin))
                            });
                            matches = matches && bulletin_matches;
                        } else {
                            matches = false;
                        }
                    }
                }

                // Filter by update type
                if let Some(type_filter) = &filter.update_type {
                    matches = matches && (update_info.update_type.as_ref() == Some(type_filter));
                }

                if matches {
                    filter_found_match = true;
                    // Only add to results if we haven't seen this update ID before
                    if !matched_update_ids.contains(&update_id) {
                        matched_update_ids.insert(update_id);
                        all_found_updates.push(update_info);
                    }
                }
            }
            
            // Check if this filter found at least one match
            if !filter_found_match {
                // Only check if the filter has at least one criterion specified
                let has_criteria = filter.title.is_some()
                    || filter.id.is_some()
                    || filter.is_installed.is_some()
                    || filter.description.is_some()
                    || filter.is_uninstallable.is_some()
                    || filter.kb_article_ids.is_some()
                    || filter.recommended_hard_disk_space.is_some()
                    || filter.msrc_severity.is_some()
                    || filter.security_bulletin_ids.is_some()
                    || filter.update_type.is_some();
                
                if has_criteria {
                    // Construct error message with filter criteria
                    let mut criteria_parts = Vec::new();
                    if let Some(title) = &filter.title {
                        criteria_parts.push(t!("export.criteriaTitle", value = title).to_string());
                    }
                    if let Some(id) = &filter.id {
                        criteria_parts.push(t!("export.criteriaId", value = id).to_string());
                    }
                    if let Some(is_installed) = filter.is_installed {
                        criteria_parts.push(t!("export.criteriaIsInstalled", value = is_installed).to_string());
                    }
                    if let Some(description) = &filter.description {
                        criteria_parts.push(t!("export.criteriaDescription", value = description).to_string());
                    }
                    if let Some(is_uninstallable) = filter.is_uninstallable {
                        criteria_parts.push(t!("export.criteriaIsUninstallable", value = is_uninstallable).to_string());
                    }
                    if let Some(kb_ids) = &filter.kb_article_ids {
                        criteria_parts.push(t!("export.criteriaKbArticleIds", value = format!("{:?}", kb_ids)).to_string());
                    }
                    if let Some(space) = filter.recommended_hard_disk_space {
                        criteria_parts.push(t!("export.criteriaRecommendedHardDiskSpace", value = space).to_string());
                    }
                    if let Some(severity) = &filter.msrc_severity {
                        criteria_parts.push(t!("export.criteriaMsrcSeverity", value = format!("{:?}", severity)).to_string());
                    }
                    if let Some(bulletin_ids) = &filter.security_bulletin_ids {
                        criteria_parts.push(t!("export.criteriaSecurityBulletinIds", value = format!("{:?}", bulletin_ids)).to_string());
                    }
                    if let Some(update_type) = &filter.update_type {
                        criteria_parts.push(t!("export.criteriaUpdateType", value = format!("{:?}", update_type)).to_string());
                    }
                    
                    let criteria_str = criteria_parts.join(", ");
                    let error_msg = t!("export.noMatchingUpdateForFilter", index = filter_index, criteria = criteria_str).to_string();
                    
                    // Emit JSON error to stderr
                    eprintln!("{{\"error\":\"{}\"}}", error_msg);
                    
                    return Err(Error::new(E_FAIL.into(), error_msg));
                }
            }
        }

        Ok(all_found_updates)
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
                metadata: None,
                updates
            };
            serde_json::to_string(&result)
                .map_err(|e| Error::new(E_FAIL.into(), t!("export.failedSerializeOutput", err = e.to_string()).to_string()))
        }
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
