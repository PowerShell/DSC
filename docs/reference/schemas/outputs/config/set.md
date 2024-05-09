---
description: JSON schema reference for the data returned by the 'dsc config set' command.
ms.date:     01/17/2024
ms.topic:    reference
title:       dsc config set result schema reference
---

# dsc config set result schema reference

## Synopsis

The result output from the `dsc config set` command.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/outputs/config/set.json
Type:          object
```

## Description

The output from the `dsc config set` command includes the state of every resource instance in the
configuration before and after the set operation, and the list of properties the operation changed
for each instance.

## Required properties

The output always includes these properties:

- [metadata](#metadata-1)
- [results](#results)
- [messages](#messages)
- [hadErrors](#haderrors)

## Properties

### metadata

Defines metadata DSC returns for a configuration operation. The properties under the
`Microsoft.DSC` property describe the context of the operation.

```yaml
Type:     object
Required: true
```

#### Microsoft.DSC

The metadata under this property describes the context of the overall operation:

- [version][01] defines the version of DSC that ran the command. This value is always the semantic
  version of the DSC command, like `3.0.0-preview.7`.
- [operation][02] defines the operation that DSC applied to the configuration document: `Get`,
  `Set`, `Test`, or `Export`.
- [executionType][03] defines whether DSC actually applied an operation to the configuration or was
  run in `WhatIf` mode. This property is always `Actual` for `Get`, `Test`, and `Export`
  operations. For `Set` operations, this value is `WhatIf` when DSC is invoked with the `--whatIf`
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
  metadata property is `Elevated`, DSC was run as `root` (non-Windows) or an elevated session with
  Administrator privileges (on Windows). If the value is `Restricted`, DSC was run as a normal user
  or account in a non-elevated session.

### results

Defines the list of results for the `set` operation invoked against every instance in the
configuration document. Every entry in the list includes the resource's type name, instance name,
and the result data for an instance.

```yaml
Type:      array
Required:  true
ItemsType: object
```

#### type

An item's `type` property identifies the instance's DSC Resource by its fully qualified type name.
For more information about type names, see
[DSC Resource fully qualified type name schema reference][10].

```yaml
Type:     string
Required: true
Pattern:  ^\w+(\.\w+){0,2}\/\w+$
```

#### name

An item's `name` property identifies the instance by its short, unique, human-readable name.

```yaml
Type:     string
Required: true
```

#### result

An item's `result` property includes the enforced state for the resource instance. The value for
this property adheres to the same schema as the output for the `dsc resource set` command. For more
information, see [dsc resource set result schema reference][11].

### messages

Defines the list of structured messages emitted by resources during the set operation. For more
information, see [Structured message schema reference][12].

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
[10]: ../../definitions/resourceType.md
[11]: ../resource/set.md
[12]: ../../definitions/message.md
