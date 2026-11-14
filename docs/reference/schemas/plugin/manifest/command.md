---
description: JSON schema reference for the 'command' property in a DSC plug-in manifest
ms.date:     11/13/2026
ms.topic:    reference
title:       DSC plug-in manifest command property schema reference
---

# DSC plug-in manifest command property schema reference

## Synopsis

Defines the command a plug-in adds to DSC and how DSC runs it.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.4.0/plugin/manifest.command.json
Type:          object
```

## Description

The `command` property of a plug-in manifest defines the name of the command the plug-in adds to
DSC and the executable that implements it.

When the user specifies the command name as the first argument to `dsc`, DSC:

1. Resolves the [executable](#executable).
1. Starts the executable in the current working directory with the standard input, output, and
   error streams of `dsc`. DSC passes the arguments defined in [args](#args), followed by every
   argument the user specified after the command name, exactly as the user specified them.
1. Waits for the executable to exit and exits with the same exit code.

DSC sets environment variables that tell the plug-in how it was invoked, like `DSC_EXE`,
`DSC_VERSION`, `DSC_PLUGIN_COMMAND`, and `DSC_PLUGIN_DIR`. For the full list, see
[Plug-in commands][01].

A plug-in command must display its help and exit with code `0` when it receives the `--help`
option. DSC passes that option when the user requests help with `dsc help <command>`.

## Required properties

The `command` definition must include these properties:

- [name](#name)
- [executable](#executable)

## Properties

### name

The `name` property defines the name of the command, which the user specifies after `dsc`. For
example, a plug-in with the command name `lint` is invoked as `dsc lint`.

A command name must start with a lowercase ASCII letter and can only contain lowercase ASCII
letters, ASCII digits, and hyphens. It can't end with a hyphen or contain consecutive hyphens. DSC
compares command names case-sensitively.

The name can't be the name or the alias of a built-in command, like `config`, `resource`, or
`help`. DSC always matches built-in commands first. When it discovers a plug-in that uses the name
of a built-in command, it writes a warning and skips the plug-in.

When plug-ins with different [types][02] define the same command name, the plug-in DSC discovers
first provides the command. The `dsc plugin list` command reports the others as shadowed. A
shadowed plug-in can still be run by its type name, like `dsc OpenDSC/Lint`.

A manifest defines exactly one command. A plug-in can implement nested commands, like
`dsc package build`, by parsing the arguments DSC passes to it.

```yaml
Type:     string
Required: true
Pattern:  ^[a-z][a-z0-9]*(-[a-z0-9]+)*$
```

### executable

The `executable` property defines the name of the executable to run for the command. The value must
be one of the following:

- The name of an executable discoverable in the system's `PATH` environment variable.
- The name of an executable in the same directory as the manifest.
- The full path to the executable.

A file extension is only required when the operating system doesn't recognize the file as an
executable without it.

To implement a plug-in as a script, define the interpreter as the executable and pass the script
in [args](#args) as a [manifest-relative path argument](#manifest-relative-path-argument).

When the `resourcePath.restricted` setting is `true`, the executable must be in one of the
directories defined by the `resourcePath.directories` setting.

```yaml
Type:     string
Required: true
```

### args

The `args` property defines the list of arguments to pass to the executable before the arguments
the user specified after the command name. Each item in the array can be a string representing a
static argument or a [manifest-relative path argument](#manifest-relative-path-argument) object.

```yaml
Type:      array
Required:  false
ItemsType: [string, object(Manifest-relative path argument)]
```

#### String arguments

Any item in the argument array can be a string representing a static argument to pass to the
executable, like `-NoProfile` or `-File`. DSC passes the string as written.

```yaml
Type: string
```

#### Manifest-relative path argument

Defines an argument that is the path to a file that ships with the plug-in. DSC replaces the item
with the absolute path to the file, resolved against the directory that contains the manifest.

DSC runs a plug-in in the current working directory of the user, not in the directory that contains
the manifest, so a relative path defined as a string argument wouldn't resolve to a file next to
the manifest. Use this argument to pass such a file, like the script that implements the plug-in.

A manifest-relative path argument is defined as a JSON object with the following properties:

- `manifestRelativePath` (required) - The path to the file, relative to the directory that contains
  the manifest, like `graph.ps1` or `scripts/graph.py`. The value can't be empty, an absolute path,
  or a path that starts with a drive or a root. DSC doesn't load a manifest that defines such a
  path.

The object must not define any other properties.

```yaml
Type:                 object
RequiredProperties:   [manifestRelativePath]
AdditionalProperties: false
```

<!-- Link reference definitions -->
[01]: ../../../cli/index.md#plug-in-commands
[02]: ./root.md#type
