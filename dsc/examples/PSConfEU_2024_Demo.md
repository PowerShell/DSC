# PSConfEU 2024 Demos

## DSC Command-line

### List DSC resources

- List DSC resources

```powershell
dsc resource list
```

- List adapted PowerShell DSC resources

```powershell
dsc resource list --adapter Microsoft.DSC/PowerShell
dsc resource list --adapter Microsoft.Windows/PowerShell
dsc resource list --adapter Microsoft.Windows/WMI *computer*
```

- Find DSC resources

```powershell
dsc resource list '*DSC*'
```

- Find specific adapted PSDSC resources

```powershell
dsc resource list --adapter Microsoft.DSC/PowerShell PSDscResources/*
### DSC Resource Invocation

- Invoke OSInfo DSC resource

```powershell
dsc resource get -r Microsoft/osinfo
```

### Output formats

- YAML
- JSON
- Pretty-JSON

### Tracing

- Error, Warn, Info, Debug, Trace

### WhatIf

## DSC Resources

### Resource Manifest

- JSON vs YAML
- Args, JSON Input, Env Var
- JSON Schema

```powershell
dsc resource schema -r Test/Echo
```

### Resource Kinds

- Resource
- Group
- Adapter
- Import

### Resource Capabilities

- Get, Set, Test, Export, SetSupportsExist, WhatIf, Resolve

## DSC Configuration

### ARM Template-like

- JSON vs YAML

### Expressions

- Functions
  - reference()
  - envvar()
- Dot-notation

### Group Resources

- Scope, examples:
  - DependsOn
  - Reference
- AssertionGroup

### PowerShell Adapter

- WindowsPowerShell
- PowerShell 7

Winget example

### WMI Adapter

```powershell
dsc resource get -r root.cimv2/Win32_ComputerSystem
```

### Include Resource

### Parameters

- Winget

```powershell
dsc config -p '{"parameters":{"ensureCalc":"Absent"}}' set -p .\dsc\examples\winget.dsc.yaml
```

- Secure paramters

### Metadata

- securityContext

### Editor experience

- JSON schema
- VSCode extension prototype
- Bicep coming later
