---
description: JSON schema reference for the 'export' property in a DSC Resource manifest
ms.date:     09/06/2023
ms.topic:    reference
title:       DSC Resource manifest export property schema reference
---

# DSC Resource manifest export property schema reference

## Synopsis

Defines how to retrieve the current state of every instance for a DSC Resource.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/resource/manifest.export.json
Type:          object
```

## Description

A command-based DSC Resource that can enumerate every instance of itself with a single command
should define the `export` property in its manifest. This property defines how DSC can get the
current state for every resource instance. When this property is defined, users can:

- Specify an instance of the resource in the input configuration for the [dsc config export][01]
  command to generate an usable configuration document.
- Specify the resource with the [dsc resource export][02] command to generate a configuration
  document that defines every instance of the resource.
- Specify the resource with the [dsc resource get][03] command and the [--all][04] option to return
  the current state for every instance of the resource.

When the DSC calls the command defined by this property, the resource must return the current state
of every instance as [JSON lines][05]. Each JSON Line should be an object representing the instance
and validate against the [defined resource instance schema][06].

## Required Properties

The `export` definition must include these properties:

- [executable](#executable)

## Properties

### executable

The `executable` property defines the name of the command to run. The value must be the name of a
command discoverable in the system's `PATH` environment variable or the full path to the command. A
file extension is only required when the command isn't recognizable by the operating system as an
executable.

```yaml
Type:     string
Required: true
```

### args

The `args` property defines an array of strings to pass as arguments to the command. DSC passes the
arguments to the command in the order they're specified.

```yaml
Type:     array
Required: false
Default:  []
```

[01]: ../../../cli/config/export.md
[02]: ../../../cli/resource/export.md
[03]: ../../../cli/resource/get.md
[04]: ../../../cli/resource/get.md#a---all
[05]: https://jsonlines.org/
[06]: schema/property.md
