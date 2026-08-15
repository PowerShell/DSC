# Example: Example/FilePresence

This example resource demonstrates how to write a complete Python DSC v3
resource using the `ms-dsc` SDK.

## What it does

`Example/FilePresence` ensures that a file at a given path either exists
(creates an empty file) or does not exist (removes the file).

## Usage

After installing this package for development:

```bash
pip install -e .
```

This automatically fetches `ms-dsc` as a build-time dependency (for IDE support and schema generation).
Note that `ms-dsc` is **not** a runtime dependency; it's provided by DSC when the resource is used via the adapter.

Usage examples:
# Get current state
dsc resource get --resource Example/FilePresence --input '{"path":"/tmp/hello.txt"}'

# Ensure file exists
dsc resource set --resource Example/FilePresence --input '{"path":"/tmp/hello.txt","_exist":true}'

# Test desired state
dsc resource test --resource Example/FilePresence --input '{"path":"/tmp/hello.txt","_exist":true}'

# Ensure file does not exist
dsc resource set --resource Example/FilePresence --input '{"path":"/tmp/hello.txt","_exist":false}'

# Delete
dsc resource delete --resource Example/FilePresence --input '{"path":"/tmp/hello.txt"}'
```

## Schema

| Property | Type | Required | Default | Description |
|----------|------|----------|---------|-------------|
| `path` | `string` | yes | — | Absolute path of the file to manage |
| `_exist` | `boolean` | no | `true` | Whether the file should exist |

## Key patterns illustrated

- `@dataclass` schema with `field(metadata={"description": "..."})` for JSON Schema keywords
- `@dsc_resource` decorator with `set_return=SetReturn.STATE_AND_DIFF`
- All five capability methods: `get`, `set`, `test`, `delete`, `export`
- Standard Python `logging` — no DSC-specific logging knowledge needed
- Entry point registration in `pyproject.toml`
