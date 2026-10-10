---
description: JSON schema reference for the data returned by the 'dsc config get' command.
ms.date:     09/01/2026
ms.topic:    reference
title:       dsc config get result schema reference
---

# dsc config get result schema reference

## Synopsis

The result output from the `dsc config get` command.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/outputs/config/get.json
Type:          object
```

## Description

The output from the `dsc config get` command includes the actual state for every resource instance
in the configuration document.

## Required properties

The output always includes these properties:

- [results](#results)
- [messages](#messages)
- [hadErrors](#haderrors)

## Properties

### executionInformation

Describes the context of the overall operation. DSC adds this property to the output of every
configuration operation. The value is an object with the following properties:

- [version][01] defines the version of DSC that ran the command. This value is always the semantic
  version of the DSC command, like `3.0.0-preview.7`.
- [operation][02] defines the operation that DSC applied to the configuration document: `get`,
  `set`, `test`, or `export`.
- [executionType][03] defines whether DSC actually applied an operation to the configuration or was
  run in what-if mode. This property is always `actual` for `get`, `test`, and `export`
  operations. For `set` operations, this value is `whatIf` when DSC is invoked with the `--what-if`
  argument.
- [startDatetime][04] defines the start date and time for the DSC operation as a timestamp
  following the format defined in [RFC3339, section 5.6 (see `date-time`)][05], like
  `2024-04-14T08:49:51.395686600-07:00`.
- [endDatetime][06] defines the end date and time for the DSC operation as a timestamp
  following the format defined in [RFC3339, section 5.6 (see `date-time`)][05], like
  `2024-04-14T08:49:51.395686600-07:00`.
- [duration][07] defines the duration of a DSC operation against a configuration document or
  resource instance as a string following the format defined in [ISO8601 ABNF for `duration`][08].
  For example, `PT0.611216S` represents a duration of about `0.61` seconds.
- [securityContext][09] defines the security context that DSC was run under. If the value for this
  metadata property is `elevated`, DSC was run as `root` (non-Windows) or an elevated session with
  Administrator privileges (on Windows). If the value is `restricted`, DSC was run as a normal user
  or account in a non-elevated session.
- [restartRequired][10] defines the list of restarts that resource instances reported as required
  during the operation. DSC only includes this property when at least one instance reported a
  required restart.

```yaml
Type:     object
Required: false
```

### metadata

Defines metadata DSC returns for a configuration operation. The properties under the
`Microsoft.DSC` property describe the context of the operation. DSC includes this property for
backwards compatibility with tools and scripts that process DSC output. In DSC version 4.0.0, the
output will no longer include this property. Prefer [executionInformation](#executioninformation)
instead.

```yaml
Type:     object
Required: false
```

#### Microsoft.DSC

The metadata under this property describes the context of the overall operation. It includes the
same properties as [executionInformation](#executioninformation). For more information, see
[Microsoft.DSC metadata property schema reference][11].

### results

Defines the list of results for the `get` operation invoked against every instance in the
configuration document. Every entry in the list includes the resource's type name, instance name,
and the result data for an instance. DSC doesn't include an entry for an instance it skipped
because the instance's `condition` didn't evaluate to `true`.

```yaml
Type:      array
Required:  true
ItemsType: object
```

#### executionInformation

An item's `executionInformation` property describes the context of the operation for the instance.
The value is an object with the following properties:

- [duration][07] defines the duration of the DSC operation against the resource instance as a
  string following the format defined in [ISO8601 ABNF for `duration`][08].
- [restartRequired][10] defines the list of restarts the resource reported as required. DSC only
  includes this property when the resource reported a required restart.

```yaml
Type:     object
Required: false
```

#### metadata

An item's `metadata` property defines the metadata DSC returns for the resource instance operation.
The `Microsoft.DSC` property under this property includes the [duration][07] of the operation. DSC
includes this property for backwards compatibility. In DSC version 4.0.0, the output will no longer
include this property.

```yaml
Type:     object
Required: false
```

#### type

An item's `type` property identifies the instance's DSC Resource by its fully qualified type name.
For more information about type names, see
[DSC Resource fully qualified type name schema reference][12].

```yaml
Type:     string
Required: true
Pattern:  ^\w+(\.\w+)*\/\w+$
```

#### name

An item's `name` property identifies the instance by its short, unique, human-readable name.

```yaml
Type:     string
Required: true
```

#### result

An item's `result` property includes the actual state for the resource instance. The value for this
property adheres to the same schema as the output for the `dsc resource get` command. For more
information, see [dsc resource get result schema reference][13].

### messages

Defines the list of structured messages emitted by resources during the get operation. For more
information, see [Structured message schema reference][14].

```yaml
Type:     array
Required: true
```

### hadErrors

Indicates whether the operation encountered any errors. This value is `true` if the configuration
document failed validation or any resource exited with an exit code other than `0`.

```yaml
Type:     boolean
Required: true
```

### outputs

Defines the values for the outputs that the configuration document defines. Each key is the name of
an output and the value is the evaluated value for that output. DSC only includes this property
when the document defines at least one output that DSC evaluated. For more information about
defining outputs, see the [outputs][15] property in the configuration document schema.

```yaml
Type:     object
Required: false
```

<!-- Link reference definitions -->
[01]: ../../metadata/Microsoft.DSC/properties.md#version
[02]: ../../metadata/Microsoft.DSC/properties.md#operation
[03]: ../../metadata/Microsoft.DSC/properties.md#executiontype
[04]: ../../metadata/Microsoft.DSC/properties.md#startdatetime
[05]: https://datatracker.ietf.org/doc/html/rfc3339#section-5.6
[06]: ../../metadata/Microsoft.DSC/properties.md#enddatetime
[07]: ../../metadata/Microsoft.DSC/properties.md#duration
[08]: https://datatracker.ietf.org/doc/html/rfc3339#appendix-A
[09]: ../../metadata/Microsoft.DSC/properties.md#securitycontext
[10]: ../../metadata/Microsoft.DSC/properties.md#restartrequired
[11]: ../../metadata/Microsoft.DSC/properties.md
[12]: ../../definitions/resourceType.md
[13]: ../resource/get.md
[14]: ../../definitions/message.md
[15]: ../../config/document.md#outputs
