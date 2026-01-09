// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::{Deserialize, Serialize};
use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::System::Com::*,
    Win32::System::Variant::*,
};

// DISPID_VALUE constant for IDispatch default property
const DISPID_VALUE: i32 = 0;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub title: String,
    pub is_installed: bool,
    pub description: String,
    pub id: String,
    pub is_uninstallable: bool,
    #[serde(rename = "KBArticleIDs")]
    pub kb_article_ids: Vec<String>,
    pub max_download_size: i64,
    pub msrc_severity: Option<MsrcSeverity>,
    pub security_bulletin_ids: Vec<String>,
    pub update_type: UpdateType,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MsrcSeverity {
    Critical,
    Important,
    Moderate,
    Low,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UpdateType {
    Software,
    Driver,
}

impl std::fmt::Display for MsrcSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MsrcSeverity::Critical => write!(f, "Critical"),
            MsrcSeverity::Important => write!(f, "Important"),
            MsrcSeverity::Moderate => write!(f, "Moderate"),
            MsrcSeverity::Low => write!(f, "Low"),
        }
    }
}

impl std::fmt::Display for UpdateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateType::Software => write!(f, "Software"),
            UpdateType::Driver => write!(f, "Driver"),
        }
    }
}

pub fn handle_get(input: &str) -> Result<String> {
    // Parse input
    let update_input: UpdateInput = serde_json::from_str(input)
        .map_err(|e| Error::new(E_INVALIDARG, format!("Failed to parse input: {}", e)))?;
    
    // Initialize COM
    unsafe {
        CoInitializeEx(Some(std::ptr::null()), COINIT_MULTITHREADED).ok()?;
    }

    let result = unsafe {
        // Create update session
        let update_session: IDispatch = CoCreateInstance(
            &CLSID_UPDATE_SESSION,
            None,
            CLSCTX_INPROC_SERVER,
        )?;

        // Create update searcher
        let searcher = invoke_method(&update_session, "CreateUpdateSearcher", &[])
            .map_err(|e| Error::new(E_FAIL, format!("Failed to create update searcher: {}", e)))?;

        // Search for updates
        let search_result = invoke_method(&searcher, "Search", &["IsInstalled=0 or IsInstalled=1"])
            .map_err(|e| Error::new(E_FAIL, format!("Failed to search for updates: {}", e)))?;

        // Get updates collection
        let updates_collection = get_property(&search_result, "Updates")
            .map_err(|e| Error::new(E_FAIL, format!("Failed to get Updates collection: {}", e)))?;
        let count = get_property_int(&updates_collection, "Count")
            .map_err(|e| Error::new(E_FAIL, format!("Failed to get update count: {}", e)))?;

        // Find the update by title or id
        let mut found_update: Option<UpdateInfo> = None;
        for i in 0..count {
            let update = invoke_method(&updates_collection, "Item", &[&i.to_string()])?;
            let title = get_property_string(&update, "Title")?;
            let identity = get_property(&update, "Identity")?;
            let update_id = get_property_string(&identity, "UpdateID")?;

            let matches = if let Some(search_title) = &update_input.title {
                title.to_lowercase().contains(&search_title.to_lowercase())
            } else if let Some(search_id) = &update_input.id {
                update_id.eq_ignore_ascii_case(search_id)
            } else {
                false
            };

            if matches {
                // Extract update information
                let is_installed = get_property_bool(&update, "IsInstalled").unwrap_or(false);
                let description = get_property_string(&update, "Description")?;
                let id = update_id;
                let is_uninstallable = get_property_bool(&update, "IsUninstallable").unwrap_or(false);

                // Get KB Article IDs
                let kb_articles = get_property(&update, "KBArticleIDs")?;
                let kb_count = get_property_int(&kb_articles, "Count").unwrap_or(0);
                let mut kb_article_ids = Vec::new();
                for j in 0..kb_count {
                    if let Ok(kb_item) = invoke_method(&kb_articles, "Item", &[&j.to_string()]) {
                        if let Ok(kb_str) = dispatch_to_string(&kb_item) {
                            kb_article_ids.push(kb_str);
                        }
                    }
                }

                // Get max download size
                let max_download_size = get_property_i64(&update, "MaxDownloadSize").unwrap_or(0);

                // Get MSRC Severity
                let msrc_severity_str = get_property_string(&update, "MsrcSeverity").ok();
                let msrc_severity = msrc_severity_str.and_then(|s| match s.as_str() {
                    "Critical" => Some(MsrcSeverity::Critical),
                    "Important" => Some(MsrcSeverity::Important),
                    "Moderate" => Some(MsrcSeverity::Moderate),
                    "Low" => Some(MsrcSeverity::Low),
                    _ => None,
                });

                // Get Security Bulletin IDs
                let security_bulletins = get_property(&update, "SecurityBulletinIDs")?;
                let bulletin_count = get_property_int(&security_bulletins, "Count").unwrap_or(0);
                let mut security_bulletin_ids = Vec::new();
                for j in 0..bulletin_count {
                    if let Ok(bulletin_item) = invoke_method(&security_bulletins, "Item", &[&j.to_string()]) {
                        if let Ok(bulletin_str) = dispatch_to_string(&bulletin_item) {
                            security_bulletin_ids.push(bulletin_str);
                        }
                    }
                }

                // Determine update type
                let type_value = get_property_int(&update, "Type").unwrap_or(1);
                let update_type = match type_value {
                    2 => UpdateType::Driver,
                    _ => UpdateType::Software,
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
        Some(update_info) => serde_json::to_string_pretty(&update_info)
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

// Helper functions for COM automation
unsafe fn get_property(object: &IDispatch, name: &str) -> Result<IDispatch> {
    let name_wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    let mut dispid: i32 = 0;
    
    let name_bstr = BSTR::from_wide(&name_wide);
    let names = [PCWSTR::from_raw(name_bstr.as_ptr())];
    
    object.GetIDsOfNames(
        &GUID::zeroed(),
        &names as *const _,
        1,
        0,
        &mut dispid,
    )?;

    let mut result = VARIANT::default();
    let params = DISPPARAMS::default();
    
    object.Invoke(
        dispid,
        &GUID::zeroed(),
        0,
        DISPATCH_METHOD | DISPATCH_PROPERTYGET,
        &params,
        Some(&mut result),
        None,
        None,
    )?;

    let dispatch: IDispatch = result.Anonymous.Anonymous.Anonymous.pdispVal.as_ref()
        .ok_or_else(|| Error::new(E_FAIL, "Failed to get IDispatch from property"))?
        .clone();
    
    VariantClear(&mut result)?;
    Ok(dispatch)
}

unsafe fn get_property_string(object: &IDispatch, name: &str) -> Result<String> {
    let name_wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    let mut dispid: i32 = 0;
    
    let name_bstr = BSTR::from_wide(&name_wide);
    let names = [PCWSTR::from_raw(name_bstr.as_ptr())];
    
    object.GetIDsOfNames(
        &GUID::zeroed(),
        &names as *const _,
        1,
        0,
        &mut dispid,
    )?;

    let mut result = VARIANT::default();
    let params = DISPPARAMS::default();
    
    object.Invoke(
        dispid,
        &GUID::zeroed(),
        0,
        DISPATCH_METHOD | DISPATCH_PROPERTYGET,
        &params,
        Some(&mut result),
        None,
        None,
    )?;

    let value = variant_to_string(&result)?;
    VariantClear(&mut result)?;
    Ok(value)
}

unsafe fn get_property_int(object: &IDispatch, name: &str) -> Result<i32> {
    let name_wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    let mut dispid: i32 = 0;
    
    let name_bstr = BSTR::from_wide(&name_wide);
    let names = [PCWSTR::from_raw(name_bstr.as_ptr())];
    
    object.GetIDsOfNames(
        &GUID::zeroed(),
        &names as *const _,
        1,
        0,
        &mut dispid,
    )?;

    let mut result = VARIANT::default();
    let params = DISPPARAMS::default();
    
    object.Invoke(
        dispid,
        &GUID::zeroed(),
        0,
        DISPATCH_METHOD | DISPATCH_PROPERTYGET,
        &params,
        Some(&mut result),
        None,
        None,
    )?;

    let value = match result.vt() {
        VT_I4 => {
            let i_val = result.Anonymous.Anonymous.Anonymous.lVal;
            VariantClear(&mut result)?;
            Ok(i_val)
        }
        _ => {
            VariantClear(&mut result)?;
            Err(Error::new(E_FAIL, format!("Property '{}' is not an integer", name)))
        }
    };

    value
}

unsafe fn get_property_i64(object: &IDispatch, name: &str) -> Result<i64> {
    let name_wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    let mut dispid: i32 = 0;
    
    let name_bstr = BSTR::from_wide(&name_wide);
    let names = [PCWSTR::from_raw(name_bstr.as_ptr())];
    
    object.GetIDsOfNames(
        &GUID::zeroed(),
        &names as *const _,
        1,
        0,
        &mut dispid,
    )?;

    let mut result = VARIANT::default();
    let params = DISPPARAMS::default();
    
    object.Invoke(
        dispid,
        &GUID::zeroed(),
        0,
        DISPATCH_METHOD | DISPATCH_PROPERTYGET,
        &params,
        Some(&mut result),
        None,
        None,
    )?;

    let value = match result.vt() {
        VT_I8 => {
            let ll_val = result.Anonymous.Anonymous.Anonymous.llVal;
            VariantClear(&mut result)?;
            Ok(ll_val)
        }
        VT_I4 => {
            let l_val = result.Anonymous.Anonymous.Anonymous.lVal as i64;
            VariantClear(&mut result)?;
            Ok(l_val)
        }
        _ => {
            VariantClear(&mut result)?;
            Err(Error::new(E_FAIL, format!("Property '{}' is not a 64-bit integer", name)))
        }
    };

    value
}

unsafe fn get_property_bool(object: &IDispatch, name: &str) -> Result<bool> {
    let name_wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    let mut dispid: i32 = 0;
    
    let name_bstr = BSTR::from_wide(&name_wide);
    let names = [PCWSTR::from_raw(name_bstr.as_ptr())];
    
    object.GetIDsOfNames(
        &GUID::zeroed(),
        &names as *const _,
        1,
        0,
        &mut dispid,
    )?;

    let mut result = VARIANT::default();
    let params = DISPPARAMS::default();
    
    object.Invoke(
        dispid,
        &GUID::zeroed(),
        0,
        DISPATCH_METHOD | DISPATCH_PROPERTYGET,
        &params,
        Some(&mut result),
        None,
        None,
    )?;

    let value = match result.vt() {
        VT_BOOL => {
            let bool_val = result.Anonymous.Anonymous.Anonymous.boolVal.0 != 0;
            VariantClear(&mut result)?;
            Ok(bool_val)
        }
        _ => {
            VariantClear(&mut result)?;
            Err(Error::new(E_FAIL, format!("Property '{}' is not a boolean", name)))
        }
    };

    value
}

unsafe fn invoke_method(object: &IDispatch, method: &str, args: &[&str]) -> Result<IDispatch> {
    let method_wide: Vec<u16> = method.encode_utf16().chain(std::iter::once(0)).collect();
    let mut dispid: i32 = 0;
    
    let method_bstr = BSTR::from_wide(&method_wide);
    let names = [PCWSTR::from_raw(method_bstr.as_ptr())];
    
    object.GetIDsOfNames(
        &GUID::zeroed(),
        &names as *const _,
        1,
        0,
        &mut dispid,
    )?;

    let mut variants: Vec<VARIANT> = Vec::new();
    for arg in args.iter().rev() {
        if let Ok(int_val) = arg.parse::<i32>() {
            variants.push(VARIANT::from(int_val));
        } else {
            let arg_wide: Vec<u16> = arg.encode_utf16().chain(std::iter::once(0)).collect();
            let bstr = BSTR::from_wide(&arg_wide);
            variants.push(VARIANT::from(bstr));
        }
    }

    let params = DISPPARAMS {
        rgvarg: if variants.is_empty() { std::ptr::null_mut() } else { variants.as_mut_ptr() },
        rgdispidNamedArgs: std::ptr::null_mut(),
        cArgs: variants.len() as u32,
        cNamedArgs: 0,
    };

    let mut result = VARIANT::default();
    
    object.Invoke(
        dispid,
        &GUID::zeroed(),
        0,
        DISPATCH_METHOD | DISPATCH_PROPERTYGET,
        &params,
        Some(&mut result),
        None,
        None,
    )?;

    let dispatch = if result.vt() == VT_DISPATCH {
        result.Anonymous.Anonymous.Anonymous.pdispVal.as_ref()
            .ok_or_else(|| Error::new(E_FAIL, "Failed to get IDispatch from method result"))?
            .clone()
    } else {
        return Err(Error::new(E_FAIL, format!("Method '{}' did not return IDispatch", method)));
    };

    for variant in variants.iter_mut() {
        VariantClear(variant)?;
    }
    VariantClear(&mut result)?;
    
    Ok(dispatch)
}

unsafe fn variant_to_string(variant: &VARIANT) -> Result<String> {
    match variant.vt() {
        VT_BSTR => {
            let bstr_ref = &variant.Anonymous.Anonymous.Anonymous.bstrVal;
            Ok(bstr_ref.to_string())
        }
        VT_DISPATCH => {
            // For IDispatch, try to convert to string
            Ok(String::from("(IDispatch object)"))
        }
        _ => {
            Err(Error::new(E_FAIL, format!("Unsupported variant type for string conversion: {}", variant.vt().0)))
        }
    }
}

unsafe fn dispatch_to_string(dispatch: &IDispatch) -> Result<String> {
    // Try to get the string value from an IDispatch object
    // For simple string wrappers, we need to invoke the default property
    let dispid: i32 = DISPID_VALUE;
    
    let mut result = VARIANT::default();
    let params = DISPPARAMS::default();
    
    dispatch.Invoke(
        dispid,
        &GUID::zeroed(),
        0,
        DISPATCH_PROPERTYGET,
        &params,
        Some(&mut result),
        None,
        None,
    )?;

    let value = variant_to_string(&result)?;
    VariantClear(&mut result)?;
    Ok(value)
}

// CLSID for Windows Update Session
const CLSID_UPDATE_SESSION: GUID = GUID::from_u128(0x4cb43d7f_7eee_4906_8698_60da1c38f2fe);
