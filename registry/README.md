# Registry command and resource

## Example JSON

```json
{
  "keyPath": "HKLM\\Software\\Microsoft\\Windows NT\\CurrentVersion",
  "valueName": "ProductName"
}
```

## Examples for config

### Get the ProductName of current version of Windows

```powershell
@'
{
  "keyPath": "HKLM\\Software\\Microsoft\\Windows NT\\CurrentVersion",
  "valueName": "ProductName"
}
'@ | registry config get
```

### Test that the key exists

```powershell
@'
{
  "keyPath": "HKLM\\Software\\Microsoft\\Windows NT\\CurrentVersion",
  "_ensure": "Present"
}
'@ | registry config test
$LASTEXITCODE -eq 0
```

### Test that the key does not exist

```powershell
@'
{
  "keyPath": "HKLM\\Software\\Microsoft\\Windows NT\\CurrentVersion",
  "_ensure": "Absent"
}
'@ | registry config test
$LASTEXITCODE -ne 0
```

### Test the the value matches

```powershell
@'
{
  "keyPath": "HKLM\\Software\\Microsoft\\Windows NT\\CurrentVersion",
  "valueName": "CompositionEditionId",
  "valueData": {
    "String": "Enterprise"
  }
}
'@ | registry config test
$LASTEXITCODE -eq 0
```
