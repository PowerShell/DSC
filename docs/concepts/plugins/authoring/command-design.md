---
description: >-
  Considerations and guidance for designing the command a DSC plug-in provides: its name,
  arguments, input, output, and help.
ms.date: 11/14/2026
title: Designing a DSC plug-in command
---

# Designing a DSC plug-in command

DSC starts a plug-in and then stays out of the way. It doesn't read the plug-in's arguments, input
or output. That gives you freedom, and it also means that everything users expect from a `dsc`
command is yours to provide. This article gives guidance for making a plug-in command feel like
part of DSC.

## Choosing a command name

The command name is what users type after `dsc`.

- Use a short, lowercase word that says what the command does: `graph`, `lint`, `report`.
- Prefer a verb or a noun that reads well after `dsc`. `dsc graph` reads better than
  `dsc show-dependency-graph`.
- Don't use the name or alias of a built-in command, like `config`, `resource`, or `help`. DSC
  skips a plug-in that does, with a warning.
- Check for other plug-ins with the same name. When two plug-ins provide the same command, the one
  DSC finds first wins and the other is shadowed. Users can still run a shadowed plug-in by its
  type name, like `dsc Contoso.Tools/Report`, but they have to know to do so.

## Defining arguments

DSC passes every argument after the command name to the plug-in as written.

- Use long options with two hyphens and kebab case, like `--output-format`. Add a one-letter short
  form only for the options users type most.
- Reuse the option names of the built-in commands where the meaning is the same: `--file` and
  `--input` for a document, `--output-format` for the format of the result.
- Don't define `--trace-level`, `--trace-format`, or `--progress-format` unless you need them to
  differ from DSC. Users specify those before the command name, and DSC passes them to the plug-in
  as environment variables. For more information, see
  [Using the context DSC provides to a plug-in][02].

The `dsc graph` plug-in defines these options:

```text
dsc graph [--file <FILE> | --input <INPUT>] [--format <FORMAT>] [--direction <DIR>]
```

## Reading input

The plug-in has the standard input of `dsc`. To work in a pipeline, read the document from
standard input when the user doesn't name a file:

```sh
dsc graph --file ./app.dsc.yaml
cat ./app.dsc.yaml | dsc graph
```

Accept both JSON and YAML when the input is a configuration document, because the built-in
commands do.

## Writing output

- Write the result, and only the result, to standard output. A user must be able to redirect it to
  a file or pipe it to another command.
- Write messages, warnings, errors, and progress to standard error. For more information, see
  [Emitting messages from a DSC plug-in][03].
- When the result is data, offer JSON. Other tools can then use your command without parsing text.

The `dsc graph` plug-in writes a Mermaid flowchart by default, because that's what a person wants
to look at, and the same graph as JSON with `--format json`.

## Providing help

DSC forwards help requests to the plug-in:

- `dsc graph --help` passes `--help` like any other argument.
- `dsc help graph` runs the plug-in with `--help`.

Handle `--help` and `-h`, write the help to standard output, and exit with code `0`. In the usage
line, show the command as the user typed it. DSC provides it in the `DSC_PLUGIN_COMMAND`
environment variable:

```powershell
$command = if ($env:DSC_PLUGIN_COMMAND) { "dsc $env:DSC_PLUGIN_COMMAND" } else { 'graph.ps1' }
"Usage: $command [OPTIONS]"
```

```console
Usage: dsc graph [OPTIONS]
```

The one-line `description` in the manifest is what `dsc --help` shows for the command. Make the
first line of the plug-in's own help say the same thing.

## Shell completion

<!--
    Placeholder. `dsc completer` completes the command name of a plug-in. Completion of the
    plug-in's own options is the second phase of the command plug-ins RFC and isn't decided yet.
-->

## Related content

- [Plug-in commands in the dsc command reference][01]
- [Using the context DSC provides to a plug-in][02]
- [Emitting messages from a DSC plug-in][03]

<!-- Link reference definitions -->
[01]: ../../../reference/cli/index.md#plug-in-commands
[02]: ./dsc-context.md
[03]: ./emitting-messages.md
