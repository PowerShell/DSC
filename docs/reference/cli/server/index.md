---
description: Command line reference for the 'dsc mcp' command
ms.date:     06/17/2026
ms.topic:    reference
title:       dsc mcp
---

# dsc mcp

## Synopsis

Starts DSC as an MCP server.

## Syntax

```sh
dsc mcp [Options]
```

## Description

The `mcp` command starts DSC as a long-running process as a Model Context Protocol (MCP) server.

## Examples

### Example 1 - Start the DSC MCP server

<a id="example-1"></a>

```sh
dsc mcp
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
