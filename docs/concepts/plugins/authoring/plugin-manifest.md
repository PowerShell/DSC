---
description: >-
  Considerations and guidance for defining the manifest of a DSC command plug-in.
ms.date: 11/14/2026
title: Defining a DSC plug-in manifest
---

# Defining a DSC plug-in manifest

A plug-in manifest is a JSON or YAML file that tells DSC which command a plug-in provides and how
to start it. DSC finds the manifest by its file name, which must end with `.dsc.plugin.json`,
`.dsc.plugin.yaml`, or `.dsc.plugin.yml`.

This article gives guidance for each part of the manifest. For the full definition of every
property, see the [DSC plug-in manifest schema reference][01].

## Minimal manifest

The smallest manifest defines the schema, the type, the version, and the command:

```json
{
  "$schema": "https://aka.ms/dsc/schemas/v3/bundled/plugin/manifest.json",
  "type": "Contoso.Tools/Report",
  "version": "1.0.0",
  "command": {
    "name": "report",
    "executable": "dsc-report"
  }
}
```

With this manifest, `dsc report --summary` starts `dsc-report --summary`.

## Identifying the plug-in

- **`type`** is the fully qualified type name, with the same syntax as a resource type name:
  `<owner>[.<group>][.<area>]/<name>`. Use an owner you control, so your plug-in doesn't collide
  with one from another publisher. Users can run a plug-in by this name, like
  `dsc Contoso.Tools/Report`, so keep it short enough to type.
- **`version`** is the semantic version of the plug-in. When DSC finds two manifests with the same
  type, it uses the one with the highest version.
- **`description`** is one line. DSC shows it next to the command in `dsc --help` and
  `dsc plugin list`. Start with a verb and describe what the command does, like the descriptions
  of the built-in commands.
- **`tags`** are search terms for the plug-in.

## Defining the command

- **`command.name`** is what the user types after `dsc`. It must be lowercase letters, digits,
  and single hyphens, and start with a letter. For guidance on choosing a name, see
  [Designing a DSC plug-in command][04].
- **`command.executable`** is the program DSC starts. Use a name that's found on `PATH`, like
  `dsc-report` or `pwsh`. Don't put a full path in the manifest, because it ties the manifest to
  one machine.
- **`command.args`** are arguments DSC puts before the arguments the user typed. Use them for a
  plug-in that's a script, to pass the script to its interpreter.

A manifest defines one command. A plug-in that needs nested commands, like `dsc report build`,
parses them from its arguments.

### Plug-ins that are scripts

When the plug-in is a script, the executable is the interpreter and the script is an argument. The
script usually sits next to the manifest, and the manifest can't know where that is on the user's
machine. Use a `manifestRelativePath` argument, which DSC replaces with the full path:

```json
{
  "command": {
    "name": "graph",
    "executable": "pwsh",
    "args": [
      "-NoLogo",
      "-NoProfile",
      "-NonInteractive",
      "-File",
      { "manifestRelativePath": "graph.ps1" }
    ]
  }
}
```

## Hiding a plug-in that can't run

Use **`condition`** when the plug-in depends on something that might not be installed. The
condition is a DSC expression. When it evaluates to `false`, DSC treats the plug-in as not
installed: it isn't listed and can't be run.

The `dsc graph` plug-in needs PowerShell, so its manifest checks for it:

```json
{
  "condition": "[not(equals(tryWhich('pwsh'), null()))]"
}
```

A user who doesn't have PowerShell then gets the normal "unrecognized subcommand" error instead of
a failure to start `pwsh`.

## Retiring a plug-in

Define **`deprecationMessage`** when you replace or retire a plug-in. DSC writes the message as a
warning each time the command runs. Say what to use instead:

```json
{
  "deprecationMessage": "Use 'dsc diagram' instead. This command is removed in version 2.0.0."
}
```

## Installing the manifest

DSC searches for plug-in manifests in the same directories as resource manifests: the directory of
the `dsc` executable, the directories in the `DSC_RESOURCE_PATH` environment variable or the
`resourcePath` setting, and the directories in `PATH`. Put the manifest and the files it refers to
in one of those directories.

To check that DSC finds the plug-in, run:

```sh
dsc plugin list
```

## Related content

- [DSC plug-in manifest schema reference][01]
- [DSC plug-in manifest command property schema reference][02]
- [dsc plugin list][03]

<!-- Link reference definitions -->
[01]: ../../../reference/schemas/plugin/manifest/root.md
[02]: ../../../reference/schemas/plugin/manifest/command.md
[03]: ../../../reference/cli/plugin/list.md
[04]: ./command-design.md#choosing-a-command-name
