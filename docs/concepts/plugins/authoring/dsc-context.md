---
description: >-
  How a DSC command plug-in uses the environment variables DSC sets, including calling back into
  the DSC that started it.
ms.date: 11/14/2026
title: Using the context DSC provides to a plug-in
---

# Using the context DSC provides to a plug-in

DSC doesn't pass anything to a plug-in on the command line except the user's arguments. It tells
the plug-in about the invocation through environment variables.

## Environment variables

| Environment variable       | Value                                                                  |
|:---------------------------|:-----------------------------------------------------------------------|
| `DSC_EXE`                  | The absolute path to the `dsc` executable that started the plug-in.    |
| `DSC_VERSION`              | The semantic version of that `dsc` executable.                         |
| `DSC_PLUGIN_COMMAND`       | What the user typed to run the plug-in: its command name or type name. |
| `DSC_PLUGIN_DIR`           | The directory that contains the plug-in manifest.                      |
| `DSC_TRACE_LEVEL`          | The trace level in effect.                                             |
| `DSC_TRACE_FORMAT`         | The trace format in effect.                                            |
| `DSC_PROGRESS_FORMAT`      | The value of `--progress-format`. Only set when the user specified it. |
| `DSC_IGNORE_SETTINGS_FILE` | `1` or `0`. Only set when the user specified `--ignore-settings-file`. |

A user can run your executable directly, without `dsc`. Decide whether that's supported. If it
is, give every variable a fallback, as the examples in this article do.

## Showing the command the user typed

Use `DSC_PLUGIN_COMMAND` in help and error messages, so they match what the user typed. The value
is the command name for `dsc graph` and the type name for `dsc Microsoft.DSC.Samples/Graph`.

## Finding files that ship with the plug-in

Use `DSC_PLUGIN_DIR` to find data files, templates, or helper scripts that are installed next to
the manifest. Don't rely on the current directory: the plug-in runs in the directory the user ran
`dsc` in.

## Honoring the options for dsc

Options for `dsc` itself go before the command name:

```sh
dsc --trace-level info graph --file ./app.dsc.yaml
```

DSC reads `--trace-level info`, and the plug-in receives `graph`'s arguments only. The trace level
arrives in `DSC_TRACE_LEVEL`. For how to use it, see
[Emitting messages from a DSC plug-in][02].

## Calling back into DSC

A plug-in often needs something DSC already does: list resources, evaluate a configuration, read a
document. Call the `dsc` executable in `DSC_EXE`. Don't call `dsc` from `PATH`, which can be a
different version than the one that started the plug-in.

DSC sets `DSC_TRACE_LEVEL` and `DSC_TRACE_FORMAT` for the plug-in process, and a `dsc` process the
plug-in starts inherits them. The tracing the user asked for applies to the whole chain.

The `dsc graph` plug-in is a PowerShell script, and PowerShell can't parse YAML without an extra
module. When neither the `powershell-yaml` nor the `Yayaml` module is installed, it asks DSC to
read the document:

```powershell
$dsc = if ($env:DSC_EXE) { $env:DSC_EXE } else { 'dsc' }
$include = @{ configurationContent = $Text } | ConvertTo-Json -Compress
$resolved = $include | & $dsc config resolve --file - --output-format json
if ($LASTEXITCODE -ne 0) {
    Write-Trace -Level error -Message "DSC couldn't read the document as a configuration document."
    exit $exitInvalidDocument
}
$document = ($resolved | ConvertFrom-Json -AsHashtable -Depth 100).configuration
```

Keep these points in mind when you call back:

- **Check the exit code.** A failed `dsc` command writes its own error to standard error, which
  the user sees. Add one line that says what the plug-in was trying to do, and exit.
- **It costs time.** Each call starts a process, and some commands discover resources. The
  `config resolve` command looks up the resource types in the document, so it's slow, or fails,
  when they aren't installed. That's why `dsc graph` only uses it as a fallback.
- **Check the version when you depend on newer behavior.** Compare `DSC_VERSION` with the version
  that introduced what you need, and tell the user when their DSC is too old.

## Related content

- [Plug-in commands in the dsc command reference][01]
- [Emitting messages from a DSC plug-in][02]
- [Example: the `dsc graph` plug-in][03]

<!-- Link reference definitions -->
[01]: ../../../reference/cli/index.md#plug-in-commands
[02]: ./emitting-messages.md
[03]: ./example-graph.md
