# Registry command and resource

## Example JSON

```json
{
  "keyPath": "HKLM\\Software\\Microsoft\\Windows NT\\CurrentVersion",
  "valueName": "ProductName"
}
```

## Example use for config

```powershell
@'
{
  "keyPath": "HKLM\\Software\\Microsoft\\Windows NT\\CurrentVersion",
  "valueName": "ProductName"
}
'@ | registry config get
```
