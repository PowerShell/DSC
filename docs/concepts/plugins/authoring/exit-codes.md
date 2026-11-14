---
description: >-
  Considerations and guidance for defining exit codes for a DSC command plug-in.
ms.date: 11/14/2026
title: Defining exit codes for a DSC plug-in
---

# Defining exit codes for a DSC plug-in

DSC exits with the exit code of the plug-in. A script that runs `dsc graph` and checks the exit
code sees the code your plug-in returned, unchanged.

## How DSC uses exit codes

DSC doesn't interpret the exit code of a plug-in and doesn't write a message for it. This differs
from resources, where DSC looks the code up in the `exitCodes` property of the resource manifest.
A plug-in manifest has no such property. The plug-in is responsible for telling the user what went
wrong, on standard error, before it exits.

The exit codes of the built-in `dsc` commands don't apply to a plug-in command. The one exception
is a command that no plug-in provides: DSC reports an unrecognized subcommand and exits with code
`2` itself.

## Defining useful exit codes

- Return `0` for success and only for success.
- Return `1` for invalid arguments. The built-in commands do the same, so scripts can treat `1`
  the same way for every `dsc` command.
- Define a code for each kind of failure a caller would handle differently. A pipeline wants to
  tell "the document is wrong" from "the tool is broken".
- Keep the list short. Callers rarely distinguish more than a few cases.
- On Linux and macOS, stay within `0` through `255`.
- Don't renumber. Callers depend on the codes once you publish them.
- Document the codes in the plug-in's help or its documentation.

## Example

The `dsc graph` plug-in defines three exit codes:

| Exit code | Meaning                                                   |
| :-------: | :-------------------------------------------------------- |
| `0`       | The graph was written.                                    |
| `1`       | The arguments are invalid.                                |
| `2`       | The document can't be read or isn't a document DSC knows. |

A document with a dependency that can't be resolved still returns `0`. The graph is drawn, with
the missing resource marked, and a warning says what's wrong. Whether a problem in the input is an
error or a warning is a design decision: return a nonzero code when the command couldn't do what
the user asked, not when it did so and found something worth mentioning.

The script names its codes once and uses the names:

```powershell
$exitSuccess = 0
$exitInvalidArgs = 1
$exitInvalidDocument = 2

if (-not (Test-Path -LiteralPath $options.File -PathType Leaf)) {
    Write-Trace -Level error -Message "The file '$($options.File)' doesn't exist."
    exit $exitInvalidDocument
}
```

## Related content

- [Emitting messages from a DSC plug-in][02]
- [Plug-in commands in the dsc command reference][01]

<!-- Link reference definitions -->
[01]: ../../../reference/cli/index.md#plug-in-commands
[02]: ./emitting-messages.md
