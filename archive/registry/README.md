# Registry command and resource

`_ensure` is optional, but if not specified,
it defaults to `Present`.

`keyPath` is a required key.
`valueName` is optional, but required if `valueData` is specified.

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

### Set a deep key with a value

```powershell
@'
{
  "keyPath": "HKCU\\1\\2\\3",
  "valueName": "Hello",
  "valueData": {
    "String": "World"
  },
  "_ensure": "Present"
}
'@ | registry config set
```

### Remove a key and everything under it

```powershell
@'
{
  "keyPath": "HKCU\\1",
  "_ensure": "Absent"
}
'@ | registry config set
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

### Set a key

```powershell
@'
{
  "keyPath": "HKCU\\Test",
  "_ensure": "Present"
}
'@ | registry config set
$LASTEXITCODE -ne 0
```

> [Note] Should this automatically create all necessary parent keys?
