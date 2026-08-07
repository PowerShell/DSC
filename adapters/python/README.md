# Microsoft.Adapter/Python — Python DSC Adapter

This directory contains the `Microsoft.Adapter/Python` DSC v3 adapter, the
`Microsoft.Python/Discover` discovery extension, and the `ms-dsc` SDK package
for writing Python-based DSC resources.

## Directory structure

```
adapters/python/
├── python.dsc.resource.json   # Adapter manifest (Windows — uses 'python')
├── python3.dsc.resource.json  # Adapter manifest (Linux/macOS — uses 'python3')
├── pyadapter/                 # Adapter implementation (stdlib-only)
│   ├── __main__.py   # Entry point: python -m pyadapter [<verb>] or python -m pyadapter.cli <verb>
│   ├── cli.py        # Argument parser + __main__ guard for -m pyadapter.cli invocation
│   ├── router.py     # Operation dispatcher (get/set/test/delete/export)
│   ├── discovery.py  # Resource discovery (entry points + editable install scanning)
│   ├── cache.py      # Disk cache keyed by installed-distribution fingerprint
│   ├── schema.py     # Dataclass → JSON Schema generation (standalone, no ms-dsc dep)
│   └── logging.py    # DscLogHandler → DSC structured JSON stderr
├── ms-dsc/                                       # SDK for resource authors (PyPI source)
│   ├── pyproject.toml
│   └── ms_dsc/       # See ms-dsc/README.md
├── examples/
│   ├── file_presence/         # Single-resource example: ensure a file exists
│   └── dsc-example-resource/  # Multi-resource demo package (Greeting, Counter, EnvVar)
├── rfc/
│   └── 0001-python-adapter.md # Design RFC
└── tests/
    ├── conftest.py     # pytest sys.path setup + manifest regeneration fixture
    ├── pytest.ini
    ├── unit/           # pytest unit tests (180+ tests)
    ├── fixture/        # installable test resource package (DscTest/*)
    └── integration/    # Pester component + DSC CLI tests
```

## Requirements

| Requirement | Purpose |
|-------------|---------|
| Python ≥ 3.11 | Adapter runtime (`python` on Windows, `python3` on Linux/macOS) |
| `dsc` CLI | DSC CLI integration tests only |

**Note:** `ms-dsc` is bundled with DSC and automatically available to resources at runtime.
For resource development, the build-time requirement is declared in `pyproject.toml` `[build-system] requires`.

## Bundled ms_dsc SDK

When DSC ships the Python adapter, the build process copies the `ms_dsc/` SDK
alongside `pyadapter/` in the same output directory.  DSC invokes the adapter
with `python -m pyadapter.cli`, which adds `''` (CWD = manifest directory) to
`sys.path[0]`.  Both `pyadapter/` and `ms_dsc/` reside in that directory, so
user resources can `import ms_dsc` without a separate `pip install` and without
any sys.path or PYTHONPATH configuration.

The build step (`Copy-PythonAdapterSdk` in `helpers.build.psm1`) copies the
runtime-only subset of `ms_dsc/` — excluding the `build/` Hatchling hook and
all `__pycache__/` directories.

### Plugin architecture

DSC resources follow a **plugin pattern** similar to pytest, Flask, and Jupyter extensions:

- **Build-time only**: `ms-dsc` is declared in `[build-system] requires` for schema generation, decorators, and IDE support
- **Not a runtime dependency**: Resources do NOT declare `ms-dsc` in `dependencies`
  - DSC provides ms-dsc at runtime via the bundled copy
  - Resources are intended for use within DSC; if shipped standalone, they fail gracefully without ms-dsc
  - This allows resources to be included in existing packages without forcing DSC/ms-dsc on users

See [Writing a Python DSC resource](#writing-a-python-dsc-resource) for the recommended `pyproject.toml` structure.

## Adapter invocation

DSC invokes the adapter as a Python module:

```
python -m pyadapter.cli <verb> [--resource TYPE]
```

Python's `-m` invocation adds `''` (CWD) to `sys.path[0]`, making both
`pyadapter` and the bundled `ms_dsc` importable without any path manipulation.

For development and manual testing, use either form from the adapter root:

```bash
# Preferred (matches DSC invocation)
python -m pyadapter.cli list
echo '{"name":"World"}' | python -m pyadapter.cli get --resource Example/Greeting

# Also works (via __main__.py)
python -m pyadapter list
echo '{"name":"World"}' | python -m pyadapter get --resource Example/Greeting
```

## Running tests

### Unit tests (no DSC required)

```bash
# Install build-time dependencies and test fixture
pip install -e tests/fixture/   # installs DscTest/* resources

# Run
cd tests/
python -m pytest unit/ -v
```

### Integration tests (Pester)

```powershell
# From the ms-dsc repo root:
Invoke-Pester adapters/python/tests/integration/python.adapter.tests.ps1 -Output Detailed
```

Requirements for integration tests:
- Python on PATH as `python` (Windows) or `python3` (Linux/macOS)
- `pip install -e adapters/python/tests/fixture/` (fetches ms-dsc as a build-time dep)
- DSC CLI on PATH (for the DSC CLI layer only)

## How discovery works

DSC discovers Python resources through two mechanisms:

1. **Discovery extension** (`Microsoft.Python/Discover`)  
   Uses `importlib.resources` to locate `<package_name>/dsc/*.dsc.adaptedResource.json`
   files inside installed distributions. Works correctly across regular wheels, editable
   installs, and bundled (non-pip-installed) resources whose manifests are found directly
   by DSC's own bin-directory scan. No caching is performed; `importlib.resources` lookup
   is fast enough at startup.

## Writing a Python DSC resource

See [ms-dsc/README.md](ms-dsc/README.md) for the full SDK guide.

### Quick summary

**pyproject.toml structure** (plugin pattern):

```toml
[build-system]
requires = ["hatchling", "ms-dsc"]  # Build-time: schema generation
build-backend = "hatchling.build"

[project]
name = "my-package"
dependencies = []                    # Runtime: EMPTY (ms-dsc provided by DSC)

[project.entry-points."microsoft.dsc.resources"]
"MyOrg/MyResource" = "my_package.dsc_resource:MyResource"
```

**Development workflow**:

1. Clone this repo or use `ms-dsc` from PyPI
2. Create your package with the structure above
3. Install for development: `pip install -e .` (automatically fetches ms-dsc build-time dep)
4. Write a class decorated with `@dsc_resource`
5. Run `dsc-gen manifest` to generate `*.dsc.adaptedResource.json`
6. Include the manifest in your wheel as package data

**Key points**:
- ms-dsc is **only** declared in `[build-system] requires`, not `dependencies`
- Your package can be installed without forcing users to install DSC or ms-dsc
- Resources work seamlessly via DSC (which provides the bundled ms-dsc)
- If someone installs your package outside of DSC and tries to import the resource code directly, they'll get a clear `ImportError` (which is correct — resources are DSC plugins)

## Examples

| Package | Resources | Purpose |
|---------|-----------|---------|
| `examples/file_presence/` | `Example/FilePresence` | Ensures a file exists or does not exist |
| `examples/file-resource/` | `Example/File` | Manages file existence and text content |
| `examples/dsc-example-resource/` | `Example/Greeting`, `Example/Counter`, `Example/EnvVar` | Full demo covering read-only, stateful, and `_exist` patterns |

### Try the demo package

```bash
pip install -e examples/dsc-example-resource/
dsc-gen manifest --pyproject examples/dsc-example-resource/pyproject.toml \
                 --out examples/dsc-example-resource/dsc_example_resource/dsc

# Get a greeting (from adapter root)
echo '{"name":"World"}' | python -m pyadapter.cli get --resource Example/Greeting

# Manage a counter
echo '{"id":"hits","value":42}' | python -m pyadapter.cli set --resource Example/Counter

# Check environment variable presence
echo '{"name":"PATH"}' | python -m pyadapter.cli get --resource Example/EnvVar
```

## Platform manifests

Two adapter manifests are provided for cross-platform support:

| Manifest | Platform | Executable | Condition |
|----------|----------|-----------|-----------|
| `python.dsc.resource.json` | Windows | `python` | `python` on PATH |
| `python3.dsc.resource.json` | Linux/macOS | `python3` | `python3` on PATH |

Both declare the same resource type (`Microsoft.Adapter/Python`) and use identical
argument structures — only the executable name differs.

## Design RFC

See [rfc/0001-python-adapter.md](rfc/0001-python-adapter.md) for the full design
rationale, component overview, discovery pipeline, and open questions.

## Discovery cache locations

| Platform | Path |
|----------|------|
| Windows | `%LOCALAPPDATA%\.dsc\PythonListCache.json` |
| Linux/macOS | `~/.dsc/PythonListCache.json` |

The discovery cache (`PythonDiscoverCache.json`) has been removed. The
`importlib.resources`-based lookup is fast enough that caching is unnecessary.

To clear the list cache:
```bash
python -m pyadapter.cli clear-cache
```
