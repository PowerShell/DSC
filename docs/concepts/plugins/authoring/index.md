---
description: >-
  Overview of the process and considerations for designing a DSC command plug-in.
ms.date: 11/14/2026
title: Designing a DSC command plug-in
---

# Designing a DSC command plug-in

A command plug-in is an executable or a script, plus a manifest in a directory that DSC searches.
This guide covers the decisions that are the same for every plug-in, whatever language you write
it in. It uses one plug-in as a running example: `dsc graph`, which shows the dependency graph of
a configuration document.

If you're new to plug-ins, read [DSC command plug-ins][01] first.

## Deciding whether to write a plug-in

Write a plug-in when you want a command that users run as `dsc <command>`:

- A tool that reads or produces configuration documents, like a linter, a formatter, or a
  converter.
- A view of DSC data, like a graph or a report.
- A workflow that combines several `dsc` commands.

Write a resource instead when DSC should manage the state of something as part of a configuration.
Write an extension instead when DSC should call your code during discovery, import, or secret
retrieval.

## Defining the plug-in manifest

DSC identifies a plug-in by its manifest. At a minimum, the manifest defines the fully qualified
type name, the version, the command name, and the executable. The description is optional, but DSC
shows it in `dsc --help`, so define it.

For more information, see [Defining a DSC plug-in manifest][02].

## Designing the command

DSC passes the arguments after the command name to the plug-in without reading them. The plug-in
owns its options, its help, and what it reads and writes. Users expect a plug-in command to behave
like the built-in commands, so follow the same conventions for option names, input and output.

For more information, see [Designing a DSC plug-in command][03].

## Using the context DSC provides

DSC tells the plug-in how it was started through environment variables: the path of the `dsc`
executable, its version, the command the user typed, and the tracing options. A plug-in can use
the path to call back into the same DSC.

For more information, see [Using the context DSC provides to a plug-in][04].

## Defining exit codes

DSC exits with the exit code of the plug-in. Scripts and pipelines that call `dsc <command>` see
that code, so define what each code means and keep it stable.

For more information, see [Defining exit codes for a DSC plug-in][05].

## Emitting messages

A plug-in writes its result to standard output and everything else to standard error. DSC passes
the trace level and trace format the user chose, so the plug-in can write messages the same way
DSC does.

For more information, see [Emitting messages from a DSC plug-in][06].

## Shell completion

<!--
    Placeholder. How a plug-in describes its options for shell completion is the second phase of
    the command plug-ins RFC and isn't decided yet. Today, `dsc completer` completes the command
    name of a plug-in and nothing after it.
-->

## Related content

- [Defining a DSC plug-in manifest][02]
- [Designing a DSC plug-in command][03]
- [Using the context DSC provides to a plug-in][04]
- [Defining exit codes for a DSC plug-in][05]
- [Emitting messages from a DSC plug-in][06]
- [Example: the `dsc graph` plug-in][07]

<!-- Link reference definitions -->
[01]: ../overview.md
[02]: ./plugin-manifest.md
[03]: ./command-design.md
[04]: ./dsc-context.md
[05]: ./exit-codes.md
[06]: ./emitting-messages.md
[07]: ./example-graph.md
