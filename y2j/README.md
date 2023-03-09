# Y2J

Simple tool to convert YAML -> JSON -> YAML making it easier to read by a person.

## Input

This command has no parameters and takes input only via STDIN (as UTF-8).

If the input is JSON, then it outputs YAML.
If the input is YAML, then it outputs pretty JSON.

JSON input is expected to be a single JSON document.

## Example

Get the JSON schema for registry resource as YAML:

```powershell
registry schema | y2j
```

Convert back to JSON to get pretty print JSON:

```powershell
registry schema | y2j | y2j
```
