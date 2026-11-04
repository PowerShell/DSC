# dsc-example-resource

Demo package for the [Microsoft.Adapter/Python](../../../README.md) DSC v3 adapter.

Install this package to see three example DSC resources in action, covering
read-only, stateful read/write, and the `_exist` presence pattern.

## Resources

| Resource | Capabilities | Description |
|----------|-------------|-------------|
| `Example/Greeting` | `get` | Returns a greeting for the given `name`. Read-only. |
| `Example/Counter` | `get`, `set`, `test`, `delete`, `export` | Named integer counter persisted in `~/.dsc/example_counters.json`. |
| `Example/EnvVar` | `get`, `set`, `test`, `delete`, `export` | Process-scope environment variable. Demonstrates `_exist`. |

## Quick start

### 1. Install for development

```bash
pip install -e .          # Fetches ms-dsc as a build-time dep; installs dsc-example-resource in editable mode
dsc-gen manifest          # Generates dsc_example_resource/dsc/*.json
```

Note: `ms-dsc` is declared in `[build-system] requires` for schema generation and IDE support during development.
It is **not** a runtime dependency; DSC provides it at runtime via the bundled copy.

### 2. Verify resources are visible

```bash
# Via adapter list (no pre-built manifest needed — entry points are enumerated)
python -m pyadapter list

# Or via DSC CLI (requires DSC on PATH and discovery extension installed)
dsc resource list --adapter Microsoft.Adapter/Python
```

### 3. Try Example/Greeting (read-only)

```bash
# Get state
python -m pyadapter get --resource Example/Greeting <<'EOF'
{"name": "World"}
EOF
# → {"name": "World", "message": "Hello, World!"}
```

### 4. Try Example/Counter (stateful read/write)

```bash
# Get (counter starts at 0)
echo '{"id":"hits"}' | python -m pyadapter get --resource Example/Counter
# → {"id": "hits", "value": 0}

# Set to 42
echo '{"id":"hits","value":42}' | python -m pyadapter set --resource Example/Counter
# → {"id": "hits", "value": 42}
# → ["value"]   ← changed properties (stateAndDiff mode)

# Test (should be in desired state)
echo '{"id":"hits","value":42}' | python -m pyadapter test --resource Example/Counter
# → {"id": "hits", "value": 42}
# → []   ← no diffs

# Export all counters
echo '{}' | python -m pyadapter export --resource Example/Counter
# → {"id": "hits", "value": 42}

# Delete
echo '{"id":"hits"}' | python -m pyadapter delete --resource Example/Counter
```

### 5. Try Example/EnvVar (_exist presence pattern)

```bash
# Check if MY_VAR exists
echo '{"name":"MY_VAR"}' | python -m pyadapter get --resource Example/EnvVar
# → {"name": "MY_VAR", "value": "", "_exist": false}

# Ensure MY_VAR=hello (creates it)
echo '{"name":"MY_VAR","value":"hello","_exist":true}' | python -m pyadapter set --resource Example/EnvVar
# → {"name": "MY_VAR", "value": "hello", "_exist": true}
# → ["_exist", "value"]   ← changed properties

# Test desired state (drift detected in the same process)
echo '{"name":"MY_VAR","value":"hello","_exist":true}' | python -m pyadapter test --resource Example/EnvVar
# → {"name": "MY_VAR", "value": "hello", "_exist": true}
# → []   ← no drift
```

## DSC configuration examples

With the DSC CLI on PATH and the package installed, you can use these resources
in a DSC configuration document. See [`examples/`](examples/) for complete YAML examples.

```yaml
# ensure-greeting.dsc.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: greeting
    type: Example/Greeting
    properties:
      name: World
```

```bash
dsc config get --document examples/ensure-greeting.dsc.yaml
```

## Schema reference

### Example/Greeting

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `name` | `string` | yes | Name to greet |
| `message` | `string` | no | Generated greeting (read-only output) |

### Example/Counter

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `id` | `string` | yes | Unique counter identifier |
| `value` | `integer` | no (default: `0`) | Counter value |

### Example/EnvVar

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `name` | `string` | yes | Environment variable name |
| `value` | `string` | no (default: `""`) | Variable value |
| `_exist` | `boolean` | no (default: `true`) | Whether the variable should exist |

## Key patterns illustrated

- **Minimal resource** (`Example/Greeting`): `get()` only, no side effects, no `_exist`.
- **Stateful resource** (`Example/Counter`): JSON file persistence, `STATE_AND_DIFF` mode.
- **`_exist` presence pattern** (`Example/EnvVar`): creation/deletion semantics, all five capabilities.
- **`@dsc_resource` decorator**: `type`, `version`, `description`, `set_return`, `test_return`.
- **`DataclassSchemaProvider`**: `field(metadata={"description": ...})` for JSON Schema keywords.
- **Standard `logging`**: `logging.getLogger(__name__)` — no DSC-specific logging API.
