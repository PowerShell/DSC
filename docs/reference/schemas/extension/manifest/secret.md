---
description: JSON schema reference for the 'secret' property in a DSC extension manifest
ms.date:     09/01/2026
ms.topic:    reference
title:       DSC extension manifest secret property schema reference
---

# DSC extension manifest secret property schema reference

## Synopsis

Defines how to retrieve a secret value from a secure store.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.0/extension/manifest.secret.json
Type:          object
```

## Description

The `secret` property defines how to call the extension to retrieve a secret from a vault at
runtime. When this property is defined, the extension has the `secret` capability and DSC can
invoke the extension for the [`secret()`][aa] configuration function.

When DSC invokes the `secret` operation for an extension, it expects the extension to write the
secret value to stdout as a single line. If the extension writes more than one line to stdout, DSC
raises an error. If the extension writes nothing to stdout, DSC treats the secret as not found for
that extension.

## Required properties

The `secret` definition must include these properties:

- [executable](#executable)

## Properties

### executable

The `executable` property defines the name of the command to run. The value must be the name of a
command secretable in the system's `PATH` environment variable or the full path to the command. A
file extension is only required when the command isn't recognizable by the operating system as an
executable.

```yaml
Type:     string
Required: true
```

### args

The `args` property defines the list of arguments to pass to the command. Each item in the array
can be a string representing a static argument, a [name argument](#name-argument) object, or a
[vault argument](#vault-argument) object.

The array should contain exactly one name argument. It may contain a single vault argument and any
number of static string arguments.

If the array doesn't define a name argument, DSC can't pass the secret name to the extension. If
the array doesn't define a vault argument, DSC can't pass the vault name to the extension.

```yaml
Type:      array
Required:  false
ItemsType: [string, object(Name or Vault argument)]
```

#### String arguments

Any item in the argument array can be a string representing a static argument to pass to the
command, like `secret` or `--format`.

```yaml
Type: string
```

#### Name argument

Defines an argument that receives the path to the file to import.

DSC passes the value of `nameArg` followed by the name of the secret to retrieve.

A file argument is defined as a JSON object with the following properties:

- `nameArg` (required) - The argument to pass before the secret name, like `--secret-name`.

```yaml
Type:               object
RequiredProperties: [nameArg]
```

#### Vault argument

Defines an argument that receives the name of a specific vault to retrieve a secret from.

DSC passes the value of `vaultArg` followed by the name of the vault when the `secret()` function
specifies a vault. When the function doesn't specify a vault, DSC ignores the vault argument.

A vault argument is defined as a JSON object with the following properties:

- `vaultArg` (required) - The argument to pass before the name of the vault to retrieve a secret
from, like `--vault-name`.

```yaml
Type:               object
RequiredProperties: [vaultArg]
```

<!-- Link reference definitions -->
[aa]: ../../config/functions/secret.md
