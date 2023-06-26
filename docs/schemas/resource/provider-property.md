# DSC Resource manifest provider property schema reference

DSC Resource Providers must define the `provider` property in their DSC Resource manifest. DSC uses
this property to determine which command-based DSC Resources are DSC Resource providers. The
`provider` property defines how DSC should call the DSC Resource Provider.

## Metadata

| Metadata Key | Metadata Value                                                           |
|:------------:|:-------------------------------------------------------------------------|
|  `$schema`   | `https://json-schema.org/draft/2020-12/schema`                           |
|    `$id`     | `https://aka.ms/dsc/schemas/resource/manifest.yaml#/properties/provider` |
|    `type`    | `object`                                                                 |

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

The manifest sets `config` to `full`, indicating that it expects a JSON blob representing the full
and unprocessed configuration from `stdin`.

It defines `list.executable` as `pwsh`. The arguments defined in `list.args` ensure that DSC runs
PowerShell:

- Without the logo banner
- In non-interactive mode
- Without loading any profile scripts
- To invoke the `powershellgroup.resource.ps1` executable in the same folder as the `dsc`
  executable and pass the `List` argument.

With this definition, DSC calls the `list` method for this DSC Resource provider by running:

```sh
pwsh -NoLogo -NonInteractive -NoProfile -Command "./powershellgroup.resource.ps1 List"
```

## Required Properties

The `provider` definition must include these properties:

- [config](#config)
- [list](#list)

## Properties

### config

The `config` property defines how the DSC Resource Provider expects to receive resource
configurations. The value must be one of the following options:

- `full` - Indicates that the DSC Resource Provider expects a JSON blob containing the full and
  unprocessed configuration as a single JSON blob over `stdin`.
- `sequence` - Indicates that the DSC Resource Provider expects each resource's configuration as
  a JSON Line over `stdin`.

```yaml
Type: string
Valid Values:
  - full
  - sequence
```

### list

The `list` property defines how DSC must call the DSC Resource Provider to list the DSC Resources
it supports. The value of this property must be an object and define the `executable` property.

```yaml
Type:     object
Required: true
Required Properties:
  - executable
```

#### executable

The `executable` property defines the name of the command to run. The value must be the name of a
command discoverable in the system's `PATH` environment variable or the full path to the
application. A file extension is only required when the executable isn't recognizable by the
operating system as an executable.

```yaml
Type:     string
Required: true
```

#### args

The `args` property defines an array of strings to pass as arguments to the command. DSC passes the
arguments to the command in the order they're specified.

```yaml
Type:     array
Required: false
Default:  []
```
