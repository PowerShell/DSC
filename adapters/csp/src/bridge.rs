// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::model::{ADAPTER_TYPE, AdapterError, Class, Property, RESOURCE_PREFIX, class_name};
use rust_i18n::t;
use serde_json::{Map, Value, json};
use windows::{
    Win32::{
        Foundation::{CloseHandle, ERROR_INSUFFICIENT_BUFFER, ERROR_NO_TOKEN, HANDLE},
        Security::{
            GetTokenInformation, IsWellKnownSid, TOKEN_QUERY, TOKEN_USER, TokenUser,
            WinLocalSystemSid,
        },
        System::{
            Threading::{GetCurrentProcess, GetCurrentThread, OpenProcessToken, OpenThreadToken},
            Variant::{VARIANT, VariantClear},
            Wmi::{CIMTYPE_ENUMERATION, IWbemQualifierSet, WBEM_E_NOT_FOUND},
        },
    },
    core::HSTRING,
};
use wmi::{IWbemClassWrapper, Variant, WMIConnection};

const NAMESPACE: &str = r"root\cimv2\mdm\dmmap";

#[derive(Default)]
struct SafeVariant(VARIANT);

impl Drop for SafeVariant {
    fn drop(&mut self) {
        if let Err(error) = unsafe { VariantClear(&raw mut self.0) } {
            crate::write_error(t!("adapter.variantClear", error = error).as_ref());
        }
    }
}

impl From<wmi::WMIError> for AdapterError {
    fn from(error: wmi::WMIError) -> Self {
        Self::Resource(t!("adapter.wmi", error = error).to_string())
    }
}

impl From<windows::core::Error> for AdapterError {
    fn from(error: windows::core::Error) -> Self {
        Self::Resource(t!("adapter.wmi", error = error).to_string())
    }
}

pub trait InstanceStore {
    fn query(&self, query: &str) -> Result<Vec<IWbemClassWrapper>, AdapterError>;
    fn spawn(&self, class: &str) -> Result<IWbemClassWrapper, AdapterError>;
    fn put(&self, object: &IWbemClassWrapper) -> Result<(), AdapterError>;
    fn delete(&self, object: &IWbemClassWrapper) -> Result<(), AdapterError>;
}

impl InstanceStore for WMIConnection {
    fn query(&self, query: &str) -> Result<Vec<IWbemClassWrapper>, AdapterError> {
        self.exec_query(query)?
            .map(|object| object.map_err(AdapterError::from))
            .collect()
    }

    fn spawn(&self, class: &str) -> Result<IWbemClassWrapper, AdapterError> {
        Ok(self.get_object(class)?.spawn_instance()?)
    }

    fn put(&self, object: &IWbemClassWrapper) -> Result<(), AdapterError> {
        Ok(self.put_instance(object)?)
    }

    fn delete(&self, object: &IWbemClassWrapper) -> Result<(), AdapterError> {
        Ok(self.delete_instance(&object.path()?)?)
    }
}

pub struct Bridge<S = WMIConnection> {
    connection: S,
}

impl Bridge {
    pub fn new() -> Result<Self, AdapterError> {
        Ok(Self {
            connection: WMIConnection::with_namespace_path(NAMESPACE)?,
        })
    }

    pub fn class(&self, resource_type: &str) -> Result<Class, AdapterError> {
        let name = class_name(resource_type)?;
        let object = self.connection.get_object(name)?;
        metadata(&object)?.ok_or_else(|| {
            AdapterError::Input(t!("adapter.notBridgeClass", name = name).to_string())
        })
    }

    pub fn list(&self) -> Result<Vec<Value>, AdapterError> {
        let executable = std::env::current_exe()
            .map_err(|error| AdapterError::Resource(t!("adapter.io", error = error).to_string()))?;
        let directory = executable.parent().ok_or_else(|| {
            AdapterError::Resource(t!("adapter.io", error = executable.display()).to_string())
        })?;
        let mut classes = Vec::new();
        for object in self.connection.exec_query("SELECT * FROM meta_class")? {
            if let Some(class) = metadata(&object?)? {
                classes.push(class);
            }
        }
        classes.sort_by(|a, b| a.name.cmp(&b.name));
        classes.into_iter().map(|class| {
            let mut capabilities = vec!["get", "test", "export"];
            if class.writable() {
                capabilities.extend(["set", "setHandlesExist"]);
            }
            Ok(json!({
                "type": format!("{RESOURCE_PREFIX}{}", class.name),
                "kind": "resource",
                "version": env!("CARGO_PKG_VERSION"),
                "capabilities": capabilities,
                "path": executable,
                "directory": directory,
                "implementedAs": "WmiBridge",
                "author": "Microsoft",
                "properties": class.properties.iter().map(|p| p.name.as_str())
                    .chain(["_exist", "_inDesiredState"]).collect::<Vec<_>>(),
                "requireAdapter": ADAPTER_TYPE,
                "description": t!("adapter.description", name = class.name, partition = class.partitions.join(", ")),
                "schema": class.schema()?
            }))
        }).collect()
    }

    pub fn check_context(class: &Class) -> Result<(), AdapterError> {
        let system = is_local_system()?;
        let expected = if system { "local-system" } else { "local-user" };
        if class.partitions.iter().any(|p| p == expected) {
            Ok(())
        } else {
            let message = if system {
                t!("adapter.userRequired", name = class.name)
            } else {
                t!("adapter.systemRequired", name = class.name)
            };
            Err(AdapterError::Resource(message.to_string()))
        }
    }
}

impl<S: InstanceStore> Bridge<S> {
    fn instances(
        &self,
        class: &Class,
        input: &Map<String, Value>,
    ) -> Result<Vec<IWbemClassWrapper>, AdapterError> {
        self.connection.query(&class.query(input))
    }

    fn instance(
        &self,
        class: &Class,
        input: &Map<String, Value>,
    ) -> Result<Option<IWbemClassWrapper>, AdapterError> {
        let mut objects = self.instances(class, input)?;
        if objects.len() > 1 {
            return Err(AdapterError::Resource(
                t!("adapter.multipleInstances", name = class.name).to_string(),
            ));
        }
        Ok(objects.pop())
    }

    pub fn get(
        &self,
        class: &Class,
        input: &Map<String, Value>,
    ) -> Result<Map<String, Value>, AdapterError> {
        self.instance(class, input)?.map_or_else(
            || Ok(class.absent(input)),
            |object| state(class, &object, false),
        )
    }

    pub fn set(
        &self,
        class: &Class,
        input: &Map<String, Value>,
    ) -> Result<Map<String, Value>, AdapterError> {
        class.validate_set(input)?;
        let object = self.instance(class, input)?;
        if input.get("_exist") == Some(&json!(false)) {
            if let Some(object) = object {
                self.connection.delete(&object)?;
            }
        } else {
            let object = match object {
                Some(object) => object,
                None => self.connection.spawn(&class.name)?,
            };
            // Mutate the existing object so omitted properties retain their provider values.
            for property in &class.properties {
                if let Some(value) = input.get(&property.name) {
                    object.put_property(&property.name, to_variant(property, value)?)?;
                }
            }
            self.connection.put(&object)?;
        }
        self.get(class, input)
    }

    pub fn export(
        &self,
        class: &Class,
        input: &Map<String, Value>,
    ) -> Result<Vec<Map<String, Value>>, AdapterError> {
        self.instances(class, input)?
            .iter()
            .map(|object| state(class, object, true))
            .collect()
    }
}

fn state(
    class: &Class,
    object: &IWbemClassWrapper,
    export: bool,
) -> Result<Map<String, Value>, AdapterError> {
    let mut state = Map::new();
    for property in class
        .properties
        .iter()
        .filter(|p| p.readable && (!export || !class.writable() || p.key || p.writable))
    {
        let mut native_value = SafeVariant::default();
        let mut cim_type = 0;
        unsafe {
            object.inner.Get(
                &HSTRING::from(&property.name),
                0,
                &raw mut native_value.0,
                Some(&raw mut cim_type),
                None,
            )?;
        }
        // wmi's get_property converts null CIM arrays to empty arrays.
        let value = match Variant::from_variant(&native_value.0)? {
            Variant::Empty | Variant::Null => Variant::Null,
            value => value.convert_into_cim_type(CIMTYPE_ENUMERATION(cim_type))?,
        };
        state.insert(
            property.name.clone(),
            serde_json::to_value(value)
                .map_err(|error| AdapterError::Resource(error.to_string()))?,
        );
    }
    state.insert("_exist".to_string(), json!(true));
    Ok(state)
}

fn to_variant(property: &Property, value: &Value) -> Result<Variant, AdapterError> {
    if !property.accepts(value) {
        return Err(AdapterError::Input(
            t!(
                "adapter.invalidValue",
                name = property.name,
                cimType = property.cim_type
            )
            .to_string(),
        ));
    }
    if value.is_null() {
        return Ok(Variant::Null);
    }
    let variant: Variant = serde_json::from_value(value.clone()).map_err(|error| {
        AdapterError::Input(t!("adapter.invalidInput", error = error).to_string())
    })?;
    Ok(variant.convert_into_cim_type(CIMTYPE_ENUMERATION(property.cim_type))?)
}

pub fn normalize_input(
    class: &Class,
    input: &Map<String, Value>,
) -> Result<Map<String, Value>, AdapterError> {
    let mut normalized = input.clone();
    for property in &class.properties {
        if let Some(value) = input.get(&property.name) {
            let value = serde_json::to_value(to_variant(property, value)?)
                .map_err(|error| AdapterError::Resource(error.to_string()))?;
            normalized.insert(property.name.clone(), value);
        }
    }
    Ok(normalized)
}

fn qualifier(set: &IWbemQualifierSet, name: &str) -> Result<Option<Value>, AdapterError> {
    let mut value = SafeVariant::default();
    // SafeVariant releases native allocations even if conversion or the provider call fails.
    match unsafe {
        set.Get(
            &HSTRING::from(name),
            0,
            &raw mut value.0,
            std::ptr::null_mut(),
        )
    } {
        Ok(()) => Ok(Some(
            serde_json::to_value(Variant::from_variant(&value.0)?)
                .map_err(|error| AdapterError::Resource(error.to_string()))?,
        )),
        Err(error) if error.code().0 == WBEM_E_NOT_FOUND.0 => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn metadata(object: &IWbemClassWrapper) -> Result<Option<Class>, AdapterError> {
    let name = object.class()?;
    if !name.starts_with("MDM_") {
        return Ok(None);
    }
    let qualifiers = unsafe { object.inner.GetQualifierSet()? };
    if qualifier(&qualifiers, "provider")?
        .as_ref()
        .and_then(Value::as_str)
        != Some("DMWmiBridgeProv")
    {
        return Ok(None);
    }
    let partitions: Vec<String> = match qualifier(&qualifiers, "InPartition")? {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
        Some(Value::String(value)) => vec![value],
        _ => Vec::new(),
    };
    if !partitions
        .iter()
        .any(|p| p == "local-system" || p == "local-user")
    {
        return Ok(None);
    }
    let mut properties = Vec::new();
    for name in object.list_properties()? {
        if name.starts_with("__") {
            continue;
        }
        let mut value = SafeVariant::default();
        let mut cim_type = 0;
        let qualifiers = unsafe {
            object.inner.Get(
                &HSTRING::from(&name),
                0,
                &raw mut value.0,
                Some(&raw mut cim_type),
                None,
            )?;
            object
                .inner
                .GetPropertyQualifierSet(&HSTRING::from(&name))?
        };
        let key = qualifier(&qualifiers, "key")? == Some(json!(true));
        properties.push(Property {
            name,
            cim_type,
            key,
            readable: key || qualifier(&qualifiers, "read")? != Some(json!(false)),
            writable: qualifier(&qualifiers, "write")? == Some(json!(true)),
        });
    }
    properties.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(Some(Class {
        name,
        partitions,
        properties,
    }))
}

fn is_local_system() -> Result<bool, AdapterError> {
    struct Token(HANDLE);
    impl Drop for Token {
        fn drop(&mut self) {
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }
    let mut handle = HANDLE::default();
    unsafe {
        match OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, true, &raw mut handle) {
            Ok(()) => {}
            Err(error) if error.code() == ERROR_NO_TOKEN.into() => {
                OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &raw mut handle)?;
            }
            Err(error) => return Err(error.into()),
        }
    }
    let token = Token(handle);
    let mut required = 0;
    // The aligned buffer owns TOKEN_USER and its SID; the token remains open while both are read.
    unsafe {
        let sizing = GetTokenInformation(token.0, TokenUser, None, 0, &raw mut required);
        if let Err(error) = sizing
            && error.code() != ERROR_INSUFFICIENT_BUFFER.into()
        {
            return Err(error.into());
        }
        let mut buffer = vec![0usize; (required as usize).div_ceil(size_of::<usize>())];
        GetTokenInformation(
            token.0,
            TokenUser,
            Some(buffer.as_mut_ptr().cast()),
            required,
            &raw mut required,
        )?;
        let user = &*buffer.as_ptr().cast::<TOKEN_USER>();
        Ok(IsWellKnownSid(user.User.Sid, WinLocalSystemSid).as_bool())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};

    struct MemoryStore {
        connection: WMIConnection,
        saved: RefCell<Option<IWbemClassWrapper>>,
        calls: RefCell<Vec<&'static str>>,
        fail_on: Cell<Option<&'static str>>,
    }

    impl MemoryStore {
        fn record(&self, operation: &'static str) -> Result<(), AdapterError> {
            self.calls.borrow_mut().push(operation);
            if self.fail_on.get() == Some(operation) {
                return Err(AdapterError::Resource(format!(
                    "injected {operation} failure"
                )));
            }
            Ok(())
        }

        fn snapshot(object: &IWbemClassWrapper) -> Result<IWbemClassWrapper, AdapterError> {
            Ok(IWbemClassWrapper::new(unsafe { object.inner.Clone()? }))
        }
    }

    impl InstanceStore for MemoryStore {
        fn query(&self, _query: &str) -> Result<Vec<IWbemClassWrapper>, AdapterError> {
            self.record("query")?;
            self.saved.borrow().as_ref().map_or_else(
                || Ok(Vec::new()),
                |object| Ok(vec![Self::snapshot(object)?]),
            )
        }

        fn spawn(&self, class: &str) -> Result<IWbemClassWrapper, AdapterError> {
            self.record("spawn")?;
            Ok(self.connection.get_object(class)?.spawn_instance()?)
        }

        fn put(&self, object: &IWbemClassWrapper) -> Result<(), AdapterError> {
            self.record("put")?;
            *self.saved.borrow_mut() = Some(Self::snapshot(object)?);
            Ok(())
        }

        fn delete(&self, _object: &IWbemClassWrapper) -> Result<(), AdapterError> {
            self.record("delete")?;
            *self.saved.borrow_mut() = None;
            Ok(())
        }
    }

    fn persistence_fixture() -> (Bridge<MemoryStore>, Class) {
        let bridge = Bridge {
            connection: MemoryStore {
                connection: WMIConnection::with_namespace_path(r"root\cimv2").unwrap(),
                saved: RefCell::new(None),
                calls: RefCell::new(Vec::new()),
                fail_on: Cell::new(None),
            },
        };
        let class = Class {
            name: "Win32_Process".to_string(),
            partitions: vec!["local-user".to_string()],
            properties: ["Handle", "Caption", "Description"]
                .into_iter()
                .map(|name| Property {
                    name: name.to_string(),
                    cim_type: 8,
                    key: name == "Handle",
                    readable: true,
                    writable: name != "Handle",
                })
                .collect(),
        };
        (bridge, class)
    }

    fn existing_input() -> Map<String, Value> {
        json!({
            "Handle": "DSC-CSP-Test",
            "Caption": "original",
            "Description": "preserved"
        })
        .as_object()
        .unwrap()
        .clone()
    }

    #[test]
    fn set_creates_and_rereads_an_instance_without_live_writes() {
        let (bridge, class) = persistence_fixture();
        let input = existing_input();
        let actual = bridge.set(&class, &input).unwrap();
        assert_eq!(actual["Handle"], input["Handle"]);
        assert_eq!(actual["Caption"], input["Caption"]);
        assert_eq!(actual["Description"], input["Description"]);
        assert_eq!(actual["_exist"], true);
        assert_eq!(
            *bridge.connection.calls.borrow(),
            ["query", "spawn", "put", "query"]
        );
    }

    #[test]
    fn set_updates_without_replacing_omitted_properties() {
        let (bridge, class) = persistence_fixture();
        bridge.set(&class, &existing_input()).unwrap();
        bridge.connection.calls.borrow_mut().clear();
        let input = json!({"Handle": "DSC-CSP-Test", "Caption": "updated"});
        let actual = bridge.set(&class, input.as_object().unwrap()).unwrap();
        assert_eq!(actual["Caption"], "updated");
        assert_eq!(actual["Description"], "preserved");
        assert_eq!(*bridge.connection.calls.borrow(), ["query", "put", "query"]);
        assert_eq!(
            bridge.get(&class, input.as_object().unwrap()).unwrap(),
            actual
        );
    }

    #[test]
    fn set_deletes_existing_instances_and_skips_absent_instances() {
        let (bridge, class) = persistence_fixture();
        bridge.set(&class, &existing_input()).unwrap();
        let input = json!({"Handle": "DSC-CSP-Test", "_exist": false});
        let input = input.as_object().unwrap();
        bridge.connection.calls.borrow_mut().clear();
        assert_eq!(bridge.set(&class, input).unwrap(), *input);
        assert!(bridge.connection.saved.borrow().is_none());
        assert_eq!(
            *bridge.connection.calls.borrow(),
            ["query", "delete", "query"]
        );

        bridge.connection.calls.borrow_mut().clear();
        assert_eq!(bridge.set(&class, input).unwrap(), *input);
        assert_eq!(*bridge.connection.calls.borrow(), ["query", "query"]);
    }

    #[test]
    fn set_propagates_create_update_and_delete_failures() {
        for operation in ["spawn", "create", "update", "delete"] {
            let (bridge, class) = persistence_fixture();
            if matches!(operation, "update" | "delete") {
                bridge.set(&class, &existing_input()).unwrap();
            }
            let failure = if matches!(operation, "create" | "update") {
                "put"
            } else {
                operation
            };
            bridge.connection.fail_on.set(Some(failure));
            bridge.connection.calls.borrow_mut().clear();
            let input = if operation == "delete" {
                json!({"Handle": "DSC-CSP-Test", "_exist": false})
            } else {
                json!({"Handle": "DSC-CSP-Test", "Caption": "changed"})
            };
            let error = bridge.set(&class, input.as_object().unwrap()).unwrap_err();
            assert!(matches!(error, AdapterError::Resource(_)));
            assert_eq!(error.to_string(), format!("injected {failure} failure"));
            assert_eq!(bridge.connection.calls.borrow().last(), Some(&failure));

            bridge.connection.fail_on.set(None);
            let actual = bridge.get(&class, input.as_object().unwrap()).unwrap();
            if matches!(operation, "update" | "delete") {
                assert_eq!(actual["Caption"], "original");
                assert_eq!(actual["Description"], "preserved");
            } else {
                assert_eq!(actual["_exist"], false);
            }
        }
    }

    #[test]
    fn set_does_not_treat_query_failures_as_missing_instances() {
        let (bridge, class) = persistence_fixture();
        bridge.connection.fail_on.set(Some("query"));
        let error = bridge.set(&class, &existing_input()).unwrap_err();
        assert_eq!(error.to_string(), "injected query failure");
        assert_eq!(*bridge.connection.calls.borrow(), ["query"]);
    }

    #[test]
    fn converts_cim_values_without_coercing_or_truncating_inputs() {
        let mut property = Property {
            name: "Setting".to_string(),
            cim_type: 3,
            key: false,
            readable: true,
            writable: true,
        };
        assert_eq!(to_variant(&property, &json!(42)).unwrap(), Variant::I4(42));
        assert!(to_variant(&property, &json!(2147483648u64)).is_err());
        assert!(to_variant(&property, &json!("42")).is_err());
        property.cim_type = 11;
        assert_eq!(
            to_variant(&property, &json!(true)).unwrap(),
            Variant::Bool(true)
        );
        property.cim_type = 8;
        assert_eq!(
            to_variant(&property, &json!("text")).unwrap(),
            Variant::String("text".to_string())
        );
        property.cim_type = 3 | 0x2000;
        assert_eq!(
            to_variant(&property, &json!([1, 2])).unwrap(),
            Variant::Array(vec![Variant::I4(1), Variant::I4(2)])
        );
        assert_eq!(to_variant(&property, &Value::Null).unwrap(), Variant::Null);
        assert_eq!(
            to_variant(&property, &json!([])).unwrap(),
            Variant::Array(vec![])
        );
    }

    #[test]
    fn normalizes_real32_precision_before_testing_state() {
        let class = Class {
            name: "MDM_Test".to_string(),
            partitions: vec!["local-user".to_string()],
            properties: vec![Property {
                name: "Setting".to_string(),
                cim_type: 4,
                key: false,
                readable: true,
                writable: true,
            }],
        };
        for value in [json!(1), json!(0.1)] {
            let input = json!({"Setting":value}).as_object().unwrap().clone();
            let actual =
                json!({"Setting":serde_json::from_value::<f32>(value).unwrap(), "_exist":true});
            let desired = normalize_input(&class, &input).unwrap();
            assert!(crate::model::diff(&desired, actual.as_object().unwrap()).is_empty());
        }
    }

    #[test]
    fn round_trips_null_and_populated_arrays_without_persisting_changes() {
        let connection = WMIConnection::with_namespace_path(r"root\cimv2").unwrap();
        let object = connection
            .get_object("Win32_NetworkAdapterConfiguration")
            .unwrap()
            .spawn_instance()
            .unwrap();
        let class = Class {
            name: "Win32_NetworkAdapterConfiguration".to_string(),
            partitions: vec!["local-user".to_string()],
            properties: vec![Property {
                name: "IPAddress".to_string(),
                cim_type: 8 | 0x2000,
                key: false,
                readable: true,
                writable: true,
            }],
        };
        for value in [Value::Null, json!(["192.0.2.1"]), Value::Null] {
            let input = json!({"IPAddress": value}).as_object().unwrap().clone();
            class.validate_set(&input).unwrap();
            object
                .put_property(
                    "IPAddress",
                    to_variant(&class.properties[0], &value).unwrap(),
                )
                .unwrap();
            let actual = state(&class, &object, false).unwrap();
            assert_eq!(actual["IPAddress"], value);
            let desired = normalize_input(&class, &input).unwrap();
            assert!(crate::model::diff(&desired, &actual).is_empty());

            let exported = state(&class, &object, true).unwrap();
            assert_eq!(exported["IPAddress"], value);
            let exported = class
                .parse_input(&serde_json::to_string(&exported).unwrap(), false)
                .unwrap();
            class.validate_set(&exported).unwrap();
            assert_eq!(
                to_variant(&class.properties[0], &exported["IPAddress"]).unwrap(),
                to_variant(&class.properties[0], &value).unwrap()
            );
        }
    }
}
