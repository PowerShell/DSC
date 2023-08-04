# DSC Resource _inDesiredState property schema

## Synopsis

Indicates whether an instance is in the desired state.

## Metadata

```yaml
Schema Dialect : https://json-schema.org/draft/2020-12/schema
Schema ID      : https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/resource/properties/inDesiredState.json
Type           : [boolean, 'null']
Read Only      : true
```

## Description

The `_inDesiredState` property indicates whether a resource instance is in the desired state.
Whether a resource's instance schema should include this property depends on whether the resource's
[manifest][01] defines the [test][02] property.

If the resource's manifest doesn't define `test`, the resource relies on DSC's synthetic test. The
resource's instance schema must not include the `_inDesiredState` property.

If the resource's manifest defines `test`, the instance schema must include the `_inDesiredState`
property. When the resource returns the state of an instance for the get and set operations,
`_inDesiredState` must be `null`. When the resource returns the state of an instance for the test
operation, `_inDesiredState` must be `true` if the instance is in the desired state and otherwise
`false`.

This property is read-only. The resource returns instances with this property, but the desired
state can't include it.

To add this property to a resource's instance schema, define the property with the following
snippet:

```json
"_inDesiredState": {
  "$ref": "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/resource/properties/inDesiredState.json"
}
```

[01]: ../manifest/root.md
[02]: ../manifest/test.md
