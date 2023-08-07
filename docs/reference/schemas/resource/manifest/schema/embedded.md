---
description: JSON schema reference for the embedded instance schema in a DSC Resource manifest
ms.date:     08/04/2023
ms.topic:    reference
title:       DSC Resource manifest embedded schema reference
---

# DSC Resource manifest embedded schema reference

## Synopsis

Defines a JSON Schema that validates a DSC Resource instance.

## Metadata

```yaml
Schema Dialect : https://json-schema.org/draft/2020-12/schema
Schema ID      : https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/resource/manifest.schema.json#/properties/embedded
Type           : object
```

## Description

The `embedded` subproperty defines a full JSON schema for a DSC Resource's instances. DSC uses the
JSON schema to validate every instance of the resource before calling the resource's commands and
after receiving an instance as output from the resource.

Embedded JSON schemas are also used by integrating and authoring tools like VS Code to validate
resource instances and provide IntelliSense.

## Required keywordds

The `embedded` definition must include these keywords:

- [$schema](#schema)
- [type](#type)
- [properties](#properties)

## Keywords

### $schema

The `$schema` keyword defines the dialect of JSON Schema the resource's instance schema uses. DSC
uses this keyword when processing the schema. The dialect defines which keywords are available and
how to interpret them.

DSC only supports JSON Schema Draft 07 and later.

```yaml
Type:     string
Required: true
Format:   uri-reference
Valid Values:
  - https://json-schema.org/draft/2020-12/schema
  - https://json-schema.org/draft/2019-09/schema
  - http://json-schema.org/draft-07/schema#
```

### $id

The `$id` keyword defines the unique ID for the instance schema. If the instance schema is published
to its own public URI, set this keyword to that URI.

```yaml
Type:     string
Required: false
Format:   uri-reference
```

### type

The `type` keyword defines what kind of value the instance is. Instances must be objects. Set this
keyword to `object`.

```yaml
Type:        string
Required:    true
Valid Value: object
```

### properties

The `properties` keyword defines the properties that DSC can retrieve and manage for the resource's
instances. This keyword must define at least one property as a key-value pair. The key is the
property's name. The value is a subschema that validates the property.

Resources can define any properties they need for managing instances. DSC defines shared schemas
for well-known properties. Some well-known properties enable a DSC Resource to use built-in
processing. The well-known properties always start with an underscore (`_`) and DSC Resources that
use these properties may not override or extend them. If a resource specifies a well-known property
in the embedded schema, the schema _must_ define the property as a reference.

- [_ensure](#_ensure)
- [_inDesiredState](#_indesiredstate)
- [_purge](#_purge)
- [_rebootRequested](#_rebootrequested)

#### _ensure

The `_ensure` property indicates that the resource can enforce whether instances exist using the
shared present and absent semantics. If a resource must distinguish between states beyond whether
an instance is present or absent, the resource should define its own `ensure` property without the
leading underscore. This property provides shared semantics for DSC Resources and integrating
tools, but doesn't enable any additional built-in processing with DSC.

If defined, this property must be a reference to the schema for the well-known property:

```json
"_ensure": {
  "$ref": "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/resource/properties/ensure.json"
}
```

For more information, see [DSC Resource _ensure property schema][01].

#### _inDesiredState

The read-only `_inDesiredState` property indicates whether a resource instance is in the desired
state. This property is mandatory for command-based DSC Resources that define the [test][02]
property.

If defined, this property must be a reference to the schema for the well-known property:

```json
"_inDesiredState": {
  "$ref": "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/resource/properties/inDesiredState.json"
}
```

For more information, see [DSC Resource _inDesiredState property schema][03].

#### _purge

Resources that need to distinguish between whether unmanaged entries in a list are valid or must be
removed can define the write-only `_purge` property. This property provides shared semantics for
DSC Resources and integrating tools, but doesn't enable any built-in processing with DSC.

If defined, this property must be a reference to the schema for the well-known property:

```json
"_inDesiredState": {
  "$ref": "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/resource/properties/purge.json"
}
```

For more information, see [DSC Resource _purge property schema][04].

#### _rebootRequested

The read-only `_rebootRequested` property indicates whether a resource instance requires a reboot
after a set operation. To use DSC's built-in reboot notification processing, resources must define
this property in their manifest.

If defined, this property must be a reference to the schema for the well-known property:

```json
"_rebootRequested": {
  "$ref": "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/resource/properties/rebootRequested.json"
}
```

For more information, see [DSC Resource _rebootRequested property schema][05]

[01]: ../../properties/ensure.md
[02]: ../test.md
[03]: ../../properties/inDesiredState.md
[04]: ../../properties/purge.md
[05]: ../../properties/rebootRequested.md
