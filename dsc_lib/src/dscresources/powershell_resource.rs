use serde_json::Value;
use super::{invoke_result::{JsonResult,GetResult,SetResult,TestResult}};
use std::process::Command;
use base64::{Engine as _, engine::{general_purpose}};
use std::collections::HashMap;

use crate::dscerror::DscError;
type JsonMap = HashMap<String, serde_json::Value>;

pub fn json_to_pshashtable(json: &str) -> Result<String, DscError> {
    let mut result = String::new();
    
    result.push_str("@{");
    let map: JsonMap = serde_json::from_str(&json)?;
    for (key, value) in map.iter() {
        if value.is_string()
        {
            result.push_str(key);

            result.push_str("=");
            let z = value.to_string();
            result.push_str(&z);
        }
        else if value.is_i64()
        {
            result.push_str(key);

            result.push_str("=");
            let z = value.as_i64().unwrap_or_default();
            result.push_str(&z.to_string());
        }
        else {
            //TODO add processing for other types
            println!("ERROR: NotImplemented - Processing of Value type of Key {}", key);
            return Err(DscError::NotImplemented);
        }

        result.push_str(",");
    }

    result.pop();
    result.push_str("}");
    //println!("{}", result);
    Ok(result)
}

pub fn invoke_dsc_resource(resource_name: &str, filter: &str, method: &str) -> Result<JsonResult, DscError> {
    // dsc.exe resource get --resource PSRepository --input "{`"Name`": PSGaller, `"Test2`":5}"
    // Invoke-DscResource -Method Get -Name PSRepository -Property @{Name="PSGallery";PackageManagementProvider="NuGet"}
    println!("Debug: 1.1.1");
    println!("Debug: {filter}");
    let properties_ht = json_to_pshashtable(filter)?;
    println!("Debug: 1.1.2");
    let script_text = format!("Invoke-DscResource -Method {method} -Name {resource_name} -Property {properties_ht}|ConvertTo-Json -Depth 3");
    println!("Debug: {script_text}");

    // PowerShell's -EncodedCommand uses UTF16 wrapped into BASE64
    let v: Vec<u16> = script_text.encode_utf16().collect();
    let mut bytes: Vec<u8> = vec![];
    for character in v
    {
        let char_bytes = character.to_le_bytes();
        bytes.push(char_bytes[0]);
        bytes.push(char_bytes[1]);
    }
    let b64 = general_purpose::STANDARD.encode(&bytes);

    let exe = "pwsh.exe"; //TODO: add support for Windows PowerShell
    let exe_args = vec![
    "-NoLogo",
    "-NonInteractive",
    "-EncodedCommand",
    &b64
    ];

    //println!("    Running {} {:?}", exe, exe_args);
    let output = Command::new(exe)
        .args(exe_args)
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    println!("    {}", output.status);
    println!("    stderr: {}", String::from_utf8_lossy(&output.stderr));
    println!("    stdout: {}", stdout);

    let result: Value = serde_json::from_str(&stdout)?;
    Ok(JsonResult {
        json: result
    })
}

pub fn invoke_get(resource_name: &str, filter: &str) -> Result<GetResult, DscError> {
    println!("Debug: 1.1");
    println!("Debug - resource_name: {resource_name}");
    println!("Debug - desired: {filter}");
    let json_result: JsonResult = invoke_dsc_resource(resource_name, filter, "Get")?;
    println!("Debug: 1.2");
    Ok(GetResult {
        actual_state: json_result.json
    })
}

pub fn invoke_set(resource_name: &str, desired: &str) -> Result<SetResult, DscError> {
    println!("Debug: 1");
    println!("Debug - resource_name: {resource_name}");
    println!("Debug - desired: {desired}");
    let pre_state = invoke_get(resource_name, desired)?;
    println!("Debug: 2");
    let json_result: JsonResult = invoke_dsc_resource(resource_name, desired, "Set")?;
    println!("Debug: 3");
    return Ok(SetResult {
        before_state: pre_state.actual_state,
        after_state: json_result.json,
        changed_properties: None, // TODO - fill the diff based on ReturnKind
    });
}

pub fn invoke_test(resource_name: &str, expected: &str) -> Result<TestResult, DscError> {

    let expected_value: Value = serde_json::from_str(expected)?;
    let json_result: JsonResult = invoke_dsc_resource(resource_name, expected, "Test")?;

    return Ok(TestResult {
        expected_state: expected_value,
        actual_state: json_result.json,
        diff_properties: None, // TODO - fill the diff based on ReturnKind
    });
}