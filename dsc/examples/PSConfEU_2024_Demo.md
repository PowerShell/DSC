# PSConfEU 2024 Demos

## DSC Command-line

### List DSC resources

- List DSC resources

```powershell
dsc resource list
```

- List PowerShell Adapted DSC resources

```powershell
dsc resource list --adapter Microsoft.DSC/PowerShell
```

- Find DSC resources

```powershell
dsc resource list '*DSC*'
```

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
- Logging to ETW/Syslog and files

### WhatIf

## DSC Resources

### Resource Manifest

- JSON vs YAML
- Args, JSON Input, Env Var
- JSON Schema

### Resource Types

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
  - env()
- Dot-notation

### Group Resources

- DependsOn scope

### PowerShell Adapter

- WindowsPowerShell
- PowerShell 7

### WMI Adapter

### Include Resource

### Parameters

- Secure paramters

### Metadata

- securityContext

### Editor experience

- JSON schema
- VSCode extension prototype
- Bicep coming later
