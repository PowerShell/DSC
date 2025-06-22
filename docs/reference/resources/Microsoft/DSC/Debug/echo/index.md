---
description: Microsoft.DSC.Debug/Echo resource reference documentation
ms.date:     06/22/2025
ms.topic:    reference
title:       Microsoft.DSC.Debug/Echo
---

# Microsoft.DSC.Debug/Echo

## Synopsis

A debug resource for testing and troubleshooting DSC (Desired State Configuration) behavior.

## Metadata

```yaml
Version    : 1.0.0
Kind       : resource
Tags       : [Windows, MacOS, Linux]
Author     : Microsoft
```

## Instance definition syntax

```yaml
resources:
  - name: <instance name>
    type: Microsoft.DSC.Debug/Echo
    properties:
      # Required properties
      output: anyOf # array, boolean, integer, string
```

## Description

The `Microsoft.DSC.Debug/Echo` resource is a debugging utility that echoes back the configuration
data passed to it. This resource is particularly useful for:

- Testing DSC configuration syntax and structure
- Debugging parameter passing between resources
- Verifying that DSC is processing configurations as expected
- Understanding how DSC transforms and handles configuration data

> [!NOTE]
> This resource is installed with DSC itself on any systems.
>
> You can update this resource by updating DSC. When you update DSC, the updated version of this
> resource is automatically available.

## Capabilities

The resource has the following capabilities:

- `get` - You can use the resource to retrieve the actual state of an instance.
- `set` - You can use the resource to enforce the desired state for an instance.
- `test` - You can use the resource to check if the actual state matches the desired state
  for an instance.

For more information about resource capabilities, see
[DSC resource capabilities][01].

> [!NOTE]
> Calling any capability on this resource does not affect the system;
> it only echoes the value in the output.

## Examples

1. [Basic echo example](./examples/basic-echo-example.md) - Shows how to use the Echo resource
   for basic string and complex data output.

## Properties

The following list describes the properties for the resource.

- **Required properties:** <a id="required-properties"></a> The following property is always
  required when defining an instance of the resource. An instance that doesn't define this
  property is invalid. For more information, see the "Required resource properties" section in
  [DSC resource properties][02]

  - [output](#output) - The value to be echoed back by the resource.

- **Key properties:** <a id="key-properties"></a> The following property uniquely identifies an
  instance. If two instances of a resource have the same value for this property, the instances are
  conflicting. For more information about key properties, see the "Key resource properties" section in [DSC resource properties][03].

  - [output](#output) (required) - The value to be echoed back by the resource.

### output

<details><summary>Expand for <code>output</code> property metadata</summary>

```yaml
Type             : anyOf (string, array, boolean, integer)
IsRequired       : true
IsKey            : true
IsReadOnly       : false
IsWriteOnly      : false
```

</details>

Defines the value to be echoed back by the resource. The `output` property can be any of the following types:

| Type    | Description                                 |
|---------|---------------------------------------------|
| string  | A string value                              |
| array   | An array of values                          |
| boolean | A boolean value (`true` or `false`)         |
| integer | An integer value                            |

## Instance validating schema

The following snippet contains the JSON Schema that validates an instance of the resource. The
validating schema only includes schema keywords that affect how the instance is validated. All
non validating keywords are omitted.

```json
{
  "type": "object",
  "required": [
    "output"
  ],
  "properties": {
    "output": {
      "$ref": "#/definitions/Output"
    }
  },
  "additionalProperties": false,
  "definitions": {
    "Output": {
      "anyOf": [
        {
          "type": "array",
          "items": true
        },
        {
          "type": "boolean"
        },
        {
          "type": "integer",
          "format": "int64"
        },
        true,
        true,
        {
          "type": "string"
        },
        {
          "type": "string"
        }
      ]
    }
  }
}
```

## See also

- [Microsoft/OSInfo resource][04]
- [DSC resource capabilities][01]

<!-- Link definitions -->
[01]: ../../../../../concepts/resources/capabilities.md
[02]: ../../../../../concepts/resources/properties.md#required-resource-properties
[03]: ../../../../../concepts/resources/properties.md#key-resource-properties
[04]: ../../osinfo/index.md
