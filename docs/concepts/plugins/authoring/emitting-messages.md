---
description: >-
  Considerations and guidance for writing messages from a DSC command plug-in, and for honoring
  the trace level and trace format the user chose.
ms.date: 11/14/2026
title: Emitting messages from a DSC plug-in
---

# Emitting messages from a DSC plug-in

A plug-in has two output streams. Standard output is for the result. Standard error is for
everything else: errors, warnings, and information about what the plug-in is doing.

DSC connects both streams of the plug-in straight to the user's terminal or pipeline. It doesn't
read, filter, or reformat them. This differs from resources, whose messages DSC consumes and
emits again as its own trace messages. A plug-in that wants its messages to look and behave like
DSC's has to do so itself.

## Honoring the trace level

The user sets the trace level with the `--trace-level` option before the command name, or in the
settings. DSC passes the level in effect to the plug-in in the `DSC_TRACE_LEVEL` environment
variable. The value is one of `error`, `warn`, `info`, `debug`, or `trace`.

Only write a message when its level is at or above the level in effect:

| Level   | Use it for                                                                   |
| :------ | :--------------------------------------------------------------------------- |
| `error` | The command can't continue. Follow it with a nonzero exit code.              |
| `warn`  | The command continues, but the result may not be what the user expects.      |
| `info`  | Decisions the user may want to know about.                                   |
| `debug` | What the plug-in is doing, for someone investigating a problem.              |
| `trace` | Detailed data, like the content it read. Never write secrets at any level.   |

When the variable isn't set or has another value, use `warn`, which is the default for `dsc`.

## Honoring the trace format

DSC passes the trace format in the `DSC_TRACE_FORMAT` environment variable. When the value is
`json`, the user is collecting the messages with a tool. Write each message as one line of JSON
with the same shape DSC uses for its own messages:

```json
{"timestamp":"2026-11-14T12:11:58.4990644Z","level":"WARN","fields":{"message":"..."}}
```

For any other value, write plain text. Start the line with the level and the name of the command,
so the user can tell the plug-in's messages from DSC's:

```console
WARN graph: 'Echo' depends on 'os' of type 'Microsoft/OSInfo', which isn't in the document.
```

## Example

The `dsc graph` plug-in writes every message through one function:

```powershell
$traceLevels = @('error', 'warn', 'info', 'debug', 'trace')

function Write-Trace {
    param(
        [Parameter(Mandatory)][ValidateSet('error', 'warn', 'info', 'debug', 'trace')][string]$Level,
        [Parameter(Mandatory)][string]$Message
    )

    $configured = "$env:DSC_TRACE_LEVEL".ToLowerInvariant()
    if ($configured -notin $traceLevels) {
        $configured = 'warn'
    }
    if ($traceLevels.IndexOf($Level) -gt $traceLevels.IndexOf($configured)) {
        return
    }

    if ($env:DSC_TRACE_FORMAT -eq 'json') {
        $line = [ordered]@{
            timestamp = [DateTime]::UtcNow.ToString('o')
            level     = $Level.ToUpperInvariant()
            fields    = @{ message = $Message }
        } | ConvertTo-Json -Compress
    } else {
        $line = '{0} graph: {1}' -f $Level.ToUpperInvariant(), $Message
    }
    [Console]::Error.WriteLine($line)
}
```

With the default trace level, the warning is written. With `--trace-level error`, it isn't:

```sh
dsc graph --file ./app.dsc.yaml
dsc --trace-level error graph --file ./app.dsc.yaml
```

## Writing good messages

- Say what happened and what the user can do about it.
- Name the thing the message is about: the file, the resource, the option.
- Write an error message before every nonzero exit. An exit code alone doesn't tell the user
  what to fix.
- Don't write to standard output for anything except the result, not even a blank line.

## Related content

- [Defining exit codes for a DSC plug-in][01]
- [Using the context DSC provides to a plug-in][02]

<!-- Link reference definitions -->
[01]: ./exit-codes.md
[02]: ./dsc-context.md
