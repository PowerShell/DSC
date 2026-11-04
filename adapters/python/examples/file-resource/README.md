# dsc-example-file-resource

Example DSC resource demonstrating how to manage file existence and content using the `ms-dsc` Python SDK.

## Resource: `Example/File`

Manages a single file on disk.

### Properties

| Property  | Type    | Default | Description                                      |
|-----------|---------|---------|--------------------------------------------------|
| `path`    | string  | —       | Absolute path of the file to manage.             |
| `content` | string  | `""`    | Text content of the file (UTF-8).                |
| `_exist`  | boolean | `true`  | Whether the file should exist.                   |

### Examples

```bash
# Get current state
dsc resource get --resource Example/File --input '{"path":"C:\\temp\\demo.txt"}'

# Ensure file exists with content
dsc resource set --resource Example/File --input '{"path":"C:\\temp\\demo.txt","content":"Hello, DSC!","_exist":true}'

# Test whether the file matches desired state
dsc resource test --resource Example/File --input '{"path":"C:\\temp\\demo.txt","content":"Hello, DSC!","_exist":true}'

# Ensure file is absent
dsc resource set --resource Example/File --input '{"path":"C:\\temp\\demo.txt","_exist":false}'

# Remove the file
dsc resource delete --resource Example/File --input '{"path":"C:\\temp\\demo.txt"}'
```

## Installation

```bash
pip install -e .
```
