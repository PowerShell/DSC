// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

extern crate ntreg;
use ntreg::registry_key::RegistryKey;
use ntreg::registry_value::*;

#[cfg(test)]

#[test]
fn test_registry_key_iter_values() {
    let key = RegistryKey::new("HKLM\\Software\\Microsoft\\Windows NT\\CurrentVersion").unwrap();
    let mut found : bool = false;
    for value in key.values() {
        if value.name == "ProductName" {
            found = true;
            match value.data {
                RegistryValueData::String(data) => {
                    assert!(data.to_string().starts_with("Windows "));
                },
                _ => {
                    assert!(false);
                }
            }
        }
        assert_ne!(value.key_path, "".to_string());
        assert_ne!(value.name, "".to_string());
    }

    assert_eq!(found, true);
}

#[test]
fn test_set_value_none() {
    let key = RegistryKey::new("HKCU");
    assert!(key.is_ok());
    let new_key = key.unwrap().create_or_get_key("TestKey_none");
    assert!(new_key.is_ok());
    let new_key = new_key.unwrap();
    let reg_value = RegistryValueData::None;
    let result = new_key.set_value("TestNone", &reg_value);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.name, "TestNone".to_string());
    assert_eq!(result.data, RegistryValueData::None);
    assert!(new_key.delete(false).is_ok());
}

#[test]
fn test_set_value_string() {
    let key = RegistryKey::new("HKCU");
    assert!(key.is_ok());
    let new_key = key.unwrap().create_or_get_key("TestKey_string");
    assert!(new_key.is_ok());
    let new_key = new_key.unwrap();
    let reg_value = RegistryValueData::String("TestStringValue".to_string());
    let result = new_key.set_value("TestString", &reg_value);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.name, "TestString".to_string());
    assert_eq!(result.data, RegistryValueData::String("TestStringValue".to_string()));
    assert!(new_key.delete(false).is_ok());
}

#[test]
fn test_set_value_expandstring() {
    let key = RegistryKey::new("HKCU");
    assert!(key.is_ok());
    let new_key = key.unwrap().create_or_get_key("TestKey_expandstring");
    assert!(new_key.is_ok());
    let new_key = new_key.unwrap();
    let reg_value = RegistryValueData::ExpandString("%PATH%;TestExpandStringValue".to_string());
    let result = new_key.set_value("TestExpandString", &reg_value);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.name, "TestExpandString".to_string());
    assert_eq!(result.data, RegistryValueData::ExpandString("%PATH%;TestExpandStringValue".to_string()));
    assert!(new_key.delete(false).is_ok());
}

#[test]
fn test_set_value_dword() {
    let key = RegistryKey::new("HKCU");
    assert!(key.is_ok());
    let new_key = key.unwrap().create_or_get_key("TestKey_dword");
    assert!(new_key.is_ok());
    let new_key = new_key.unwrap();
    let reg_value = RegistryValueData::DWord(1234);
    let result = new_key.set_value("TestDword", &reg_value);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.name, "TestDword".to_string());
    assert_eq!(result.data, RegistryValueData::DWord(1234));
    assert!(new_key.delete(false).is_ok());
}

#[test]
fn test_set_value_qword() {
    let key = RegistryKey::new("HKCU");
    assert!(key.is_ok());
    let new_key = key.unwrap().create_or_get_key("TestKey_qword");
    assert!(new_key.is_ok());
    let new_key = new_key.unwrap();
    let reg_value = RegistryValueData::QWord(12_345_678_901);
    let result = new_key.set_value("TestQword", &reg_value);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.name, "TestQword".to_string());
    assert_eq!(result.data, RegistryValueData::QWord(12_345_678_901));
    assert!(new_key.delete(false).is_ok());
}

#[test]
fn test_set_value_multistring() {
    let key = RegistryKey::new("HKCU");
    assert!(key.is_ok());
    let new_key = key.unwrap().create_or_get_key("TestKey_multistring");
    assert!(new_key.is_ok());
    let new_key = new_key.unwrap();
    let reg_value = RegistryValueData::MultiString(vec!["TestMultiStringValue1".to_string(), "TestMultiStringValue2".to_string()]);
    let result = new_key.set_value("TestMultiString", &reg_value);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.name, "TestMultiString".to_string());
    assert_eq!(result.data, RegistryValueData::MultiString(vec!["TestMultiStringValue1".to_string(), "TestMultiStringValue2".to_string()]));
    assert!(new_key.delete(false).is_ok());
}

#[test]
fn test_set_value_binary() {
    let key = RegistryKey::new("HKCU");
    assert!(key.is_ok());
    let new_key = key.unwrap().create_or_get_key("TestKey_binary");
    assert!(new_key.is_ok());
    let new_key = new_key.unwrap();
    let reg_value = RegistryValueData::Binary(vec![0x01, 0x02, 0x03, 0x04]);
    let result = new_key.set_value("TestBinary", &reg_value);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.name, "TestBinary".to_string());
    assert_eq!(result.data, RegistryValueData::Binary(vec![0x01, 0x02, 0x03, 0x04]));
    assert!(new_key.delete(false).is_ok());
}

#[test]
fn test_set_value_resoucelist() {
    let key = RegistryKey::new("HKCU");
    assert!(key.is_ok());
    let new_key = key.unwrap().create_or_get_key("TestKey_resourcelist");
    assert!(new_key.is_ok());
    let new_key = new_key.unwrap();
    let reg_value = RegistryValueData::ResourceList(vec![0x01, 0x02, 0x03, 0x04]);
    let result = new_key.set_value("TestResourceList", &reg_value);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.name, "TestResourceList".to_string());
    assert_eq!(result.data, RegistryValueData::ResourceList(vec![0x01, 0x02, 0x03, 0x04]));
    assert!(new_key.delete(false).is_ok());
}

#[test]
fn test_set_value_fullresourcedescriptor() {
    let key = RegistryKey::new("HKCU");
    assert!(key.is_ok());
    let new_key = key.unwrap().create_or_get_key("TestKey_FullResourceDescriptor");
    assert!(new_key.is_ok());
    let new_key = new_key.unwrap();
    let reg_value = RegistryValueData::FullResourceDescriptor(vec![0x01, 0x02, 0x03, 0x04]);
    let result = new_key.set_value("TestFullResourceDescriptor", &reg_value);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.name, "TestFullResourceDescriptor".to_string());
    assert_eq!(result.data, RegistryValueData::FullResourceDescriptor(vec![0x01, 0x02, 0x03, 0x04]));
    assert!(new_key.delete(false).is_ok());
}

#[test]
fn test_set_value_resourcerequirementslist() {
    let key = RegistryKey::new("HKCU");
    assert!(key.is_ok());
    let new_key = key.unwrap().create_or_get_key("TestKey_ResourceRequirementsList");
    assert!(new_key.is_ok());
    let new_key = new_key.unwrap();
    let reg_value = RegistryValueData::ResourceRequirementsList(vec![0x01, 0x02, 0x03, 0x04]);
    let result = new_key.set_value("TestResourceRequirementsList", &reg_value);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.name, "TestResourceRequirementsList".to_string());
    assert_eq!(result.data, RegistryValueData::ResourceRequirementsList(vec![0x01, 0x02, 0x03, 0x04]));
    assert!(new_key.delete(false).is_ok());
}

#[test]
fn test_value_refresh() {
    let key = RegistryKey::new("HKCU").unwrap();
    let new_key = key.create_or_get_key("TestKey_refresh");
    assert!(new_key.is_ok());
    let new_key = new_key.unwrap();
    let mut reg_value = new_key.set_value("TestValue", &RegistryValueData::String("TestStringValue".to_string())).unwrap();
    assert_eq!(reg_value.name, "TestValue".to_string());
    assert_eq!(reg_value.data, RegistryValueData::String("TestStringValue".to_string()));
    let _ = new_key.set_value("TestValue", &RegistryValueData::String("NewValue".to_string())).unwrap();
    assert!(reg_value.query().is_ok());
    assert_eq!(reg_value.name, "TestValue".to_string());
    assert_eq!(reg_value.data, RegistryValueData::String("NewValue".to_string()));
    assert!(new_key.delete(false).is_ok());
}
