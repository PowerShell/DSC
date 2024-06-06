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

### Tracing

### WhatIf

## DSC Resources

### Resource Manifest

### Resource Types

## DSC Configuration

### ARM Template-like

- JSON vs YAML

### Expressions

- Reference

### Parameters

- Secure paramters

### Metadata

- securityContext
