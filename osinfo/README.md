# osinfo resource

This resource only supports `get` and returns basic information about the OS. It's intended to be
an example of a assertion type resource where `test` is synthetically implemented by DSC.

As this resource is, by design, basic, it doesn't even include JSON schema as it's not intended to
accept any input.

## Direct invocation

This command takes no arguments so only outputs basic info as JSON:

```powershell
osinfo
```

Example output:

```json
{
  "$id": "https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json",
  "type": "Windows",
  "version": "10.0.25309",
  "edition": "Windows 11 Professional",
  "bitness": "X64"
}
```

> [!NOTE]
> In this document it's formatted, but the command outputs as one line of compressed JSON without
> any whitespace.

## Performing a `get`

Since this resource takes no input, you can run:

```powershell
dsc resource get -r osinfo
```

Example output as YAML:

```yaml
actual_state:
  $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
  type: Windows
  version: 10.0.25309
  edition: Windows 11 Professional
  bitness: X64
```

## Performing a `test`

A `test` does require input, but keep in mind this resource doesn't implement schema so the input
isn't validated:

```powershell
'{"type":"Unknown"}' | dsc resource test -r osinfo
```

Example output as YAML:

```yaml
expected_state:
  type: unknown
actual_state:
  $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
  type: Windows
  version: 10.0.25309
  edition: Windows 11 Professional
  bitness: X64
diff_properties:
- type
```
