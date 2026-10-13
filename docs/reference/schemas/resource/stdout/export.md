---
description: JSON schema reference for the expected stdout from the export resource operation
ms.date:     07/29/2025
ms.topic:    reference
title:       DSC resource export operation stdout schema reference
---

# DSC resource export operation stdout schema reference

## Synopsis

Represents the actual state of a resource instance in DSC. DSC expects every JSON Line emitted to
stdout for the **Export** operation to adhere to this schema.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/resource/stdout/export.json
Type:          object
```

## Description

DSC expects a resource implementing the **Export** operation to return a series of JSON Lines.

The data that DSC expects depends on whether the resource kind is defined as `exporter`:

- When the resource kind is `exporter`, DSC expects the resource to return JSON Lines representing
  DSC resource instance definitions to recursively export.
- When the resource kind isn't `exporter`, DSC expects the resource to return JSON Lines
  representing the actual state of every instance of the resource on the system.

## Typical resource expected output

DSC expects a typical resource implementing the **Export** operation to return a series of JSON
Lines.

Each JSON Line represents the actual state of a resource instance in DSC. DSC expects every JSON
Line emitted to stdout for the **Export** operation to adhere to this schema.

The output must be a JSON object. The object must be a valid representation of an instance of the
resource.

Command resources define their instance schema with the [schema.command][01] or
[schema.embedded][02] fields in their resource manifest. If a command resource returns JSON that is
invalid against the resource instance schema, DSC raises an error.

Adapted resource instances are validated by their adapter when the adapter invokes them.

## Exporter resource expected output

DSC expects an exporter resource (one with the [kind][03] field in its manifest set to `exporter`)
to return a series of JSON Lines.

Each JSON Line represents a DSC resource instance definition to recursively invoke the **Export**
operation for. DSC expects every JSON Line emitted to stdout for the **Export** operation to adhere
to this schema.

The output must be a JSON object adhering to [DSC resource instance][04] schema, rather than the
instance schema for a specific resource. DSC expects the object to define at least the [name][05]
and [type][06] fields. If the object defines the [properties][07] field, DSC passes those
properties to the resource when recursively exporting it so that the resource may filter the
exported instance results.

<!-- Reference link definitions -->
[01]: ../manifest/schema/property.md
[02]: ../manifest/schema/embedded.md
[03]: ../manifest/root.md#kind
[04]: ../../config/resource.md
[05]: ../../config/resource.md#name
[06]: ../../config/resource.md#type
[07]: ../../config/resource.md#properties-1
