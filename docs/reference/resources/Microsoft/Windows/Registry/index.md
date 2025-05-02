---
description: Microsoft.Windows/Registry resource reference documentation
ms.date:     03/25/2025
ms.topic:    reference
title:       Microsoft.Windows/Registry
---

# Microsoft.Windows/Registry

## Synopsis

Manage Windows Registry keys and values.

> [!IMPORTANT]
> The `registry` command and `Microsoft.Windows/Registry` resource are a proof-of-concept example
> for use with DSC. Don't use it in production.

## Metadata

```yaml
Version    : 0.1.0
Kind       : resource
Tags       : [Windows]
Author     : Microsoft
```

## Instance definition syntax

```yaml
resources:
  - name: <instance name>
    type: Microsoft.Windows/Registry
    properties:
      # Required properties
      keyPath: string
      # Instance properties
      _exist:
      valueData:
      valueName:
```

## Description

The `Microsoft.Windows/Registry` resource enables you to idempotently manage registry keys and
values. The resource can:

- Add and remove registry keys.
- Add, update, and remove registry values.

> [!NOTE]
> This resource is installed with DSC itself on Windows systems.
>
> You can update this resource by updating DSC. When you update DSC, the updated version of this
> resource is automatically available.

## Requirements

- The resource is only usable on a Windows system.
- The resource must run in a process context that has permissions to manage the keys in the hive
  specified by the value of the **keyPath** property. For more information, see
  [Registry key Security and Access Rights][01].

## Capabilities

The resource has the following capabilities:

- `get` - You can use the resource to retrieve the actual state of an instance.
- `set` - You can use the resource to enforce the desired state for an instance.
- `whatIf` - The resource is able to report how it would change system state during a **Set**
  operation in what-if mode.
- `delete` - You can use the resource to directly remove an instance from the system.

This resource uses the synthetic test functionality of DSC to determine whether an instance is in
the desired state. For more information about resource capabilities, see
[DSC resource capabilities][02].

## Examples

1. [Manage a registry key][03] - Shows how to create and delete registry keys with the
   `dsc resource` commands.
1. [Manage a registry value][04] - Shows how to create, modify, and delete registry values with the
   `dsc resource` commands.
1. [Configure registry keys and values][05] - Shows how to define registry keys and values in a
   configuration document.

## Properties

The following list describes the properties for the resource.

- **Required properties:** <a id="required-properties"></a> The following properties are always
  required when defining an instance of the resource. An instance that doesn't define each of these
  properties is invalid. For more information, see the "Required resource properties" section in
  [DSC resource properties][06]

  - [keyPath](#keypath) - The path to the registry key.

- **Key properties:** <a id="key-properties"> The following properties uniquely identify an
  instance. If two instances of a resource have the same values for their key properties, the
  instances are conflicting. For more information about key properties, see the "Key resource
  properties" section in [DSC resource properties][07].

  - [keyPath](#keypath) (required) - The path to the registry key.
  - [valueName](#valuename) (optional) - The name of the registry value.

- **Instance properties:** <a id="instance-properties"></a> The following properties are optional.
  They define the desired state for an instance of the resource.

  - [_exist](#_exist) - Defines whether the registry key or value should exist.
  - [valueData](#valuedata) - The data for a registry value.
  - [valueName](#valuename) - The name of the registry value.

- **Read-only properties:** <a id="read-only-properties"></a> The resource returns the following
  properties, but they aren't configurable. For more information about read-only properties, see
  the "Read-only resource properties" section in [DSC resource properties][08].

  - [_metadata](#_metadata) - Defines metadata returned by the resource.

### keyPath

<details><summary>Expand for <code>keyPath</code> property metadata</summary>

```yaml
Type             : string
IsRequired       : true
IsKey            : true
IsReadOnly       : false
IsWriteOnly      : false
```

</details>

Defines the path to the registry key for the instance. The path must start with a valid hive
identifier. Separate each segment of the path with a backslash (`\`).

The following table describes the valid hive identifiers for the key path.

| Short Name |       Long Name       | NT Path                                                                 |
|:----------:|:---------------------:|:------------------------------------------------------------------------|
|   `HKCR`   |  `HKEY_CLASSES_ROOT`  | `\Registry\Machine\Software\Classes\`                                   |
|   `HKCU`   |  `HKEY_CURRENT_USER`  | `\Registry\User\<User SID>\`                                            |
|   `HKLM`   | `HKEY_LOCAL_MACHINE`  | `\Registry\Machine\`                                                    |
|   `HKU`    |     `HKEY_USERS`      | `\Registry\User\`                                                       |
|   `HKCC`   | `HKEY_CURRENT_CONFIG` | `\Registry\Machine\System\CurrentControlSet\Hardware Profiles\Current\` |

### _exist

<details><summary>Expand for <code>_exist</code> property metadata</summary>

```yaml
Type             : boolean
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
DefaultValue     : true
```

</details>

The `_exist` canonical resource property determines whether an instance should exist. When the
value for `_exist` is `true`, the resource adds or creates the instance if it doesn't exist. When
the value for `_exist` is `false`, the resource removes or deletes the instance if it does exist.
The default value for this property when not specified for an instance is `true`.

### valueData

<details><summary>Expand for <code>valueData</code> property metadata</summary>

```yaml
Type                 :  object
IsRequired           : false
IsKey                : false
IsReadOnly           : false
IsWriteOnly          : false
RequiresProperties   : [valueName]
MinimumPropertyCount : 1
MaximumPropertyCount : 1
```

</details>

Defines the data for the registry value. If specified, this property must be an object with a
single property. The property name defines the data type. The property value defines the data
value. When the instance defines this property, the `valueName` property must also be defined. An
instance that defines `valueData` without `valueName` is invalid.

`valueData` has the following properties:

- [String](#string-valuedata) - Defines the value as a string (`REG_SZ`).
- [ExpandString](#expandstring-valuedata) - Defines the value as a string with expandable
  references (`REG_EXPAND_SZ`).
- [MultiString](#multistring-valuedata) - Defines the value as a sequence of strings
  (`REG_MULTI_SZ`).
- [Binary](#binary-valuedata) - Defines the value as a sequence of bytes (`REG_BINARY`).
- [DWord](#dword-valuedata) - Defines the value as a 32-bit unsigned integer (`REG_DWORD`).
- [QWord](#qword-valuedata) - Defines the value as a 64-bit unsigned integer (`REG_QWORD`).

For more information on registry value data types, see
[Registry value types][09].

#### String valueData

<details><summary>Expand for <code>valueData.String</code> subproperty metadata</summary>

```yaml
Type : string
```

</details>

Defines the registry value data as a null-terminated UTF-16 string. The resource handles
terminating the string.

#### ExpandString valueData

<details><summary>Expand for <code>valueData.ExpandString</code> subproperty metadata</summary>

```yaml
Type : string
```

</details>

Defines the registry value data as a null-terminated UTF-16 that contains unexpanded references to
environment variables, like `%PATH%`. The resource handles terminating the string.

#### MultiString valueData

<details><summary>Expand for <code>valueData.MultiString</code> subproperty metadata</summary>

```yaml
Type              : array
ItemsType         : string
ItemsMustBeUnique : false
ItemsMinimumCount : 0
```

</details>

Defines the registry value data as a sequence of null-terminated UTF-16 strings. The resource
handles terminating the strings.

#### Binary valueData

<details><summary>Expand for <code>valueData.Binary</code> subproperty metadata</summary>

```yaml
Type                       : array
ItemsType                  : integer
ItemsInclusiveMinimumValue : 0
ItemsInclusiveMaximumValue : 255
ItemsMustBeUnique          : false
```

</details>

Defines the registry value data as binary data in any form. The value must be an array of 8-bit
unsigned integers.

#### DWord valueData

<details><summary>Expand for <code>valueData.DWord</code> subproperty metadata</summary>

```yaml
Type                  : integer
InclusiveMinimumValue : 0
InclusiveMaximumValue : 4294967295
```

</details>

Defines the registry value data as a 32-bit unsigned integer.

#### QWord valueData

<details><summary>Expand for <code>valueData.QWord</code> subproperty metadata</summary>

```yaml
Type                  : integer
InclusiveMinimumValue : 0
InclusiveMaximumValue : 18446744073709551615
```

</details>

Defines the registry value data as a 64-bit unsigned integer.

### valueName

<details><summary>Expand for <code>valueName</code> property metadata</summary>

```yaml
Type:  string
IsKey: false
```

</details>

Defines the name of the value to manage for the registry key. This property is required when
specifying the `valueData` property.

### _metadata

<details><summary>Expand for <code>_metadata</code> property metadata</summary>

```yaml
Type         : object
IsRequired   : false
IsKey        : false
IsReadOnly   : true
IsWriteOnly  : false
IsDeprecated : false
```

</details>

This property is returned by the resource for **Set** operations invoked in what-if mode. For other
operations, the return data from the resource doesn't include this property.

`_metadata` has the following properties:

- [whatIf](#whatif) - Contains messages about how the resource would change the system in a **Set**
  operation.

#### whatIf

<details><summary>Expand for <code>_metadata.whatIf</code> subproperty metadata</summary>

```yaml
Type              : array
ItemsType         : string
ItemsMustBeUnique : false
ItemsMinimumCount : 0
```

</details>

This metadata property is only returned when invoking the resource set operation in what-if mode.
It contains any number of messages from the resource about how it would change the system in a set
operation without the `--what-if` flag.

## Instance validating schema

The following snippet contains the JSON Schema that validates an instance of the resource. The
validating schema only includes schema keywords that affect how the instance is validated. All
nonvalidating keywords are omitted.

```json
{
  "type": "object",
  "required": [
    "keyPath"
  ],
  "dependentRequired": {
    "valueData": [
      "valueName"
    ]
  },
  "additionalProperties": false,
  "properties": {
    "_exist": {
      "type": "boolean",
      "default": true
    },
    "_metadata": {
      "type": "object",
      "readOnly": true,
      "properties": {
        "whatIf": {
          "type": "array",
          "readOnly": true,
          "items": {
            "type": "string"
          }
        }
      }
    },
    "keyPath": {
      "type": "string",
      "pattern": "^(HKCR|HKEY_CLASSES_ROOT|HKCU|HKEY_CURRENT_USER|HKLM|HKEY_LOCAL_MACHINE|HKU|HKEY_USERS|HKCC|HKEY_CURRENT_CONFIG)\\\\"
    },
    "valueData": {
      "type": "object",
      "minProperties": 1,
      "maxProperties": 1,
      "properties": {
        "String": {
          "type": "string"
        },
        "ExpandString": {
          "type": "string"
        },
        "MultiString": {
          "type": "string"
        },
        "Binary": {
          "type": "array",
          "items": {
            "type": "integer",
            "minimum": 0,
            "maximum": 255
          }
        },
        "DWord": {
          "type": "integer",
          "minimum": 0,
          "maximum": 4294967295
        },
        "QWord": {
          "type": "integer",
          "minimum": 0,
          "maximum": 18446744073709551615
        }
      }
    },
    "valueName": {
      "type": "string"
    }
  }
}
```

## Exit codes

The resource returns the following exit codes from operations:

- [0](#exit-code-0) - Success
- [1](#exit-code-1) - Invalid parameter
- [2](#exit-code-2) - Invalid input
- [3](#exit-code-3) - Registry error
- [4](#exit-code-4) - Json serialization failed

### Exit code 0

Indicates the resource operation completed without errors.

### Exit code 1

Indicates the resource operation failed due to an invalid parameter. When the resource returns this
exit code, it also emits an error message with details about the invalid parameter.

### Exit code 2

Indicates the resource operation failed because the input instance was invalid. When the resource
returns this exit code, it also emits one or more error messages with details describing how the
input instance was invalid.

### Exit code 3

Indicates the resource operation failed due to an error raised by the Windows Registry API. When
the resource returns this exit code, it also emits the error message raised by the registry.

### Exit code 4

Indicates the resource operation failed because the result couldn't be serialized to JSON.

## See also

- [Microsoft/OSInfo resource][10]
- For more information about the Windows Registry, see [About the Registry][11]

<!-- Link definitions -->
[01]: /windows/win32/sysinfo/registry-key-security-and-access-rights
[02]: ../../../../../concepts/resources/capabilities.md
[03]: ./examples/manage-a-registry-key.md
[04]: ./examples/manage-a-registry-value.md
[05]: ./examples/configure-registry-keys-and-values.md
[06]: ../../../../../concepts/resources/properties.md#required-resource-properties
[07]: ../../../../../concepts/resources/properties.md#key-resource-properties
[08]: ../../../../../concepts/resources/properties.md#read-only-resource-properties
[09]: /en-us/windows/win32/sysinfo/registry-value-types
[10]: ../../osinfo/index.md
[11]: /windows/win32/sysinfo/about-the-registry
