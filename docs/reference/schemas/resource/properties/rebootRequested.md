# DSC Resource _rebootRequested property schema

## Synopsis

Indicates whether an instance is in the desired state.

## Metadata

```yaml
Schema Dialect : https://json-schema.org/draft/2020-12/schema
Schema ID      : https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/resource/properties/rebootRequested.json
Type           : [boolean, 'null']
Read Only      : true
```

## Description

The `_rebootRequested` property indicates whether a resource instance requires a reboot after a set
operation.

If the resource determines during a set operation that the node needs to reboot before the state
change takes full effect, it should return the instance's data with the `_rebootRequested` property
set to `true`. If a resource returns an instance from the set operation with the `_rebootRequested`
property set to `true`, DSC generates a reboot notification.

If the resource returns an instance from the set operation without the `_rebootRequested` property
defined, or if the property value is `false`, DSC doesn't generate a reboot notification.

Resources must define this property to use DSC's built-in reboot request notifications.

This property is read-only. The resource returns instances with this property, but the desired
state can't include it.

To add this property to a resource's instance schema, define the property with the following
snippet:

```json
"_rebootRequested": {
  "$ref": "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/resource/properties/rebootRequested.json"
}
```
