---
description: Command line reference for the 'dsc completer' command
ms.date:     03/25/2025
ms.topic:    reference
title:       dsc completer
---

# dsc completer

## Synopsis

Generates a shell completion script.

## Syntax

```sh
dsc completer [Options] <SHELL>
```

## Description

The `completer` command returns a shell script that, when executed, registers completions for the
given shell. DSC can generate completion scripts for the following shells:

- [Bourne Again SHell (BASH)][01]
- [Elvish][02]
- [Friendly Interactive SHell (fish)][03]
- [PowerShell][04]
- [Z SHell (ZSH)][05]

The output for this command is the script itself. To register completions for DSC, execute the
script.

> [!WARNING]
> Always review scripts before executing them, especially in an elevated execution context.

## Examples

### Example 1 - Register completions for Bash

```sh
completer=~/dsc_completion.bash
# Export the completion script
dsc completer bash > $completer
# Review the completion script
cat $completer
# Add the completion script to your profile
echo "source $completer" >> ~/.bashrc
# Execute the completion script to register completions for this session
source $completer
```

### Example 2 - Register completions for PowerShell

```powershell
$Completer = '~/dsc_completion.ps1'
# Export the completion script
dsc completer powershell | Out-File $Completer
# Review the completion script
Get-Content $completer
# Add the completion script to your profile
Add-Content -Path $PROFILE ". $Completer"
# Execute the completion script to register completions for this session
. $Completer
```

## Arguments

### SHELL

This argument is mandatory for the `completer` command. The value for this option determines which
shell the application returns a completion script for:

- `bash` - [Bourne Again SHell (BASH)][01]
- `elvish` - [Elvish][02]
- `fish` - [Friendly Interactive SHell (fish)][03]
- `powershell` - [PowerShell][04]
- `zsh` - [Z SHell (ZSH)][05]

```yaml
Type        : string
Mandatory   : true
ValidValues : [
                bash,
                elvish,
                fish,
                powershell,
                zsh,
              ]
```

## Options

### -h, --help

<a id="-h"></a>
<a id="--help"></a>

Displays the help for the current command or subcommand. When you specify this option, the
application ignores all other options and arguments.

```yaml
Type        : boolean
Mandatory   : false
LongSyntax  : --help
ShortSyntax : -h
```

[01]: https://www.gnu.org/software/bash/
[02]: https://elv.sh/
[03]: https://fishshell.com/
[04]: /powershell/scripting/overview
[05]: https://zsh.sourceforge.io/
