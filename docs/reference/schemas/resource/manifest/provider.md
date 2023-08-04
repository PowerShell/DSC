# DSC Resource manifest provider property schema reference

## Synopsis

Defines a DSC Resource as a DSC Resource Provider.

## Metadata

```yaml
Schema Dialect : https://json-schema.org/draft/2020-12/schema
Schema ID      : https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/resource/manifest.provider.json
Type           : object
```

## Description

DSC Resource Providers must define the `provider` property in their manifest. This property
identifies the resource as a provider and defines how DSC can call the provider to get the
resources the provider supports and how to pass resource instances to the provider.

## Examples

### Example 1 - DSC/PowerShellGroup

This example is from the `DSC/PowerShellGroup` DSC Resource Provider.

```json
"provider": {
  "config": "full",
  "list": {
    "executable": "pwsh",
    "args": [
      "-NoLogo"
      "-NonInteractive"
      "-NoProfile"
      "-Command"
      "./powershellgroup.resource.ps1 List"
    ]
  }
}
```

The manifest sets `config` to `full`, indicating that the provider expects a JSON blob representing
the full and unprocessed configuration from `stdin`.

It defines `list.executable` as `pwsh`. The arguments defined in `list.args` ensure that DSC runs
PowerShell:

- Without the logo banner
- In non-interactive mode
- Without loading any profile scripts
- To invoke the `powershellgroup.resource.ps1` script in the same folder as the `dsc` command and
  pass the `List` argument.

With this definition, DSC calls the `list` method for this provider by running:

```sh
pwsh -NoLogo -NonInteractive -NoProfile -Command "./powershellgroup.resource.ps1 List"
```

## Required Properties

The `provider` definition must include these properties:

- [config](#config)
- [list](#list)

## Properties

### config

The `config` property defines how the provider expects to receive resource configurations. The
value must be one of the following options:

- `full` - Indicates that the provider expects a JSON blob containing the full and
  unprocessed configuration as a single JSON blob over `stdin`.
- `sequence` - Indicates that the provider expects each resource's configuration as
  a [JSON Line][01] over `stdin`.

```yaml
Type: string
Valid Values:
  - full
  - sequence
```

### list

The `list` property defines how to call the provider to list the resources it supports. The value
of this property must be an object and define the `executable` subproperty.

```yaml
Type:     object
Required: true
Required Properties:
  - executable
```

#### executable

The `executable` subproperty defines the name of the command to run. The value must be the name of
a command discoverable in the system's `PATH` environment variable or the full path to the command.
A file extension is only required when the command isn't recognizable by the operating system as an
executable.

```yaml
Type:     string
Required: true
```

#### args

The `args` subproperty defines an array of strings to pass as arguments to the command. DSC passes
the arguments to the command in the order they're specified.

```yaml
Type:     array
Required: false
Default:  []
```

[01]: https://jsonlines.org/
