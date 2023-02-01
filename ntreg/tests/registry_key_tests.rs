extern crate ntreg;
use ntreg::{registry_key::RegistryKey};

#[cfg(test)]

#[test]
fn test_registry_key_constructor() {
    let key = RegistryKey::new("HKLM\\Software\\Microsoft\\Windows\\CurrentVersion\\Run").unwrap();
    assert_eq!(key.path, "HKLM\\Software\\Microsoft\\Windows\\CurrentVersion\\Run".to_string());
}

#[test]
fn test_registry_key_iter_subkey() {
    let key = RegistryKey::new("HKLM\\Software\\Microsoft\\PowerShellCore").unwrap();
    let mut found : bool = false;
    for subkey in key.subkeys() {
        if subkey.name == "InstalledVersions" {
            found = true;
            assert_eq!(subkey.path, "HKLM\\Software\\Microsoft\\PowerShellCore\\InstalledVersions".to_string());
        }
        assert_ne!(subkey.path, "".to_string());
    }

    assert_eq!(found, true);
}

#[test]
fn test_open_hklm() {
    let hklm = RegistryKey::new("HKLM");
    assert!(hklm.is_ok());
    let hklm = hklm.unwrap();
    assert_eq!(hklm.path, "HKLM".to_string());
    assert!(hklm.sub_key_count > 0);
}

#[test]
fn test_open_hkcu() {
    let hkcu = RegistryKey::new("HKCU");
    assert!(hkcu.is_ok());
    let hkcu = hkcu.unwrap();
    assert_eq!(hkcu.path, "HKCU".to_string());
    assert!(hkcu.sub_key_count > 0);
}

#[test]
fn test_create_and_delete() {
    // delete test key if it exists
    let key = RegistryKey::new("HKCU\\TestKey");
    if key.is_ok() {
        key.unwrap().delete(true).unwrap();
    }

    let key = RegistryKey::new("HKCU").unwrap();
    let new_key = key.create_key("TestKey").unwrap();
    assert!(new_key.path.ends_with("\\TestKey"));
    let key = RegistryKey::new("HKCU\\TestKey");
    assert!(key.is_ok());
    assert!(key.unwrap().delete(false).is_ok());
    let key = RegistryKey::new("HKCU\\TestKey");
    assert!(key.is_err());
}

#[test]
fn test_create_and_delete_with_subkeys() {
    // delete test key if it exists
    let key = RegistryKey::new("HKCU\\TestKeyWithChildren");
    if key.is_ok() {
        key.unwrap().delete(true).unwrap();
    }

    let key = RegistryKey::new("HKCU").unwrap();
    let new_key = key.create_key("TestKeyWithChildren").unwrap();
    assert!(new_key.path.ends_with("\\TestKeyWithChildren"));
    let sub_key = new_key.create_key("SubKey").unwrap();
    assert!(sub_key.path.ends_with("\\TestKeyWithChildren\\SubKey"));
    assert!(new_key.delete(true).is_ok());
}
