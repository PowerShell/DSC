---
description: JSON schema reference for the 'import' property in a DSC extension manifest
ms.date:     09/01/2026
ms.topic:    reference
title:       DSC extension manifest import property schema reference
---

# DSC extension manifest import property schema reference

## Synopsis

Defines how to call the extension to convert a file that DSC can't parse directly into a
configuration document.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.0/extension/manifest.import.json
Type:          object
```

## Description

The `import` property of an extension manifest defines how to call the extension to convert a file
that DSC can't parse directly into a configuration document. When this property is defined, the
extension has the `import` capability.

When you pass a file to a `dsc config` command with the `--file` option, DSC checks whether any
extension with the `import` capability handles the file's extension. DSC uses the content returned
by the first extension that successfully imports the file as the configuration document. If no
extension imports the file, DSC reads the file directly.

The extension must return the imported file as a [JSON Line][01]. The JSON Line must be an object
that validates against the [DSC extension import operation stdout schema reference][02].

## Required properties

The `import` definition must include these properties:

- [fileExtensions](#fileextensions)
- [executable](#executable)

## Properties

### fileExtensions

The `fileExtensions` property defines the list of file extensions that the extension can import as
a configuration document, like `["bicep"]` to import `.bicep` files.

The value for this property must be an array. Every item in the array must be a file extension
_without_ a leading period. If this array is empty, DSC writes a warning during discovery and the
extension can't import any files.

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

The `args` property defines the list of arguments to pass to the command. Each item in the array
can be a string representing a static argument or an [file argument](#file-argument) object that
receives the absolute path to the file the extension should import.

```yaml
Type:      array
Required:  false
ItemsType: [string, object(Extensions argument)]
```

#### String arguments

Any item in the argument array can be a string representing a static argument to pass to the
command, like `import` or `--format`.

```yaml
Type: string
```

#### File argument

Defines an argument that receives the path to the file to import.

DSC passes the value of `fileArg` followed by the absolute path to the file to import. If `fileArg`
is an empty string, DSC passes only the path.

A file argument is defined as a JSON object with the following properties:

- `fileArg` (required) - The argument to pass before the path to the file to import, like `--path`.

```yaml
Type:               object
RequiredProperties: [fileArg]
```

### output

The `output` property defines a DSC configuration expression that DSC evaluates after the command
completes to transform the command's output into the configuration document. Use the `stdout()`
function in the expression to access the text the command wrote to stdout. When this property isn't
defined, DSC uses the command's output as the configuration document without modification.

```yaml
type: string
```

<!-- Link reference definitions -->
[01]: https://jsonlines.org/
[02]: ../stdout/import.md
