# Python Adapter Test Layers

This folder contains three test layers for the Python adapter, each with a different scope and dependency profile.

## Test Files In This Folder

- `pythonunit.tests.py`
- `pythoncomponent.tests.ps1`
- `pythonintegration.tests.ps1`
- `pythontest.dsc.manifests.json`
- `pyproject.toml` (test class mapping for `PythonTest/*` resources)
- `src/` test resource implementations

## 1) Unit Layer

**File:** `pythonunit.tests.py`

**Purpose:** validate adapter internals in-process using Python `unittest`.

**Covers:**
- adapter initialization and trace/profiling flags
- manifest discovery and class-map resolution from `pyproject.toml`
- operation routing (`list`, `validate`, `get`, `set`, `test`, `export`)
- stdout emission contract for `set`/`test`
- logging and edge/error handling paths

**Run:**

```powershell
python adapters/python/tests/pythonunit.tests.py
```

## 2) Component Layer

**File:** `pythoncomponent.tests.ps1`

**Purpose:** validate adapter behavior as a subprocess without going through DSC CLI.

**How it runs:**
- resolves fixture resource paths from `pythontest.dsc.manifests.json`
- executes adapter entrypoint directly: `pyDscAdapter/__main__.py`
- passes both `--resource` and `--resource-path`

**Covers:**
- get/set/test/export/list/validate behavior via adapter subprocess
- output shape and exit-code behavior
- invalid JSON and unknown resource handling

**Run:**

```powershell
Invoke-Pester adapters/python/tests/pythoncomponent.tests.ps1 -Output Detailed
```

## 3) Integration Layer

**File:** `pythonintegration.tests.ps1`

**Purpose:** validate end-to-end behavior through DSC CLI.

**How it runs:**
- executes `dsc resource list|get|set|test|export`
- validates DSC-level wrapping and adapter interoperability
- includes tolerated known-environment failures (for example missing `python3` in DSC process context)

**Covers:**
- adapter discovery/listing via DSC
- operation contracts through DSC wrappers
- error propagation from adapter to DSC output

**Run:**

```powershell
Invoke-Pester adapters/python/tests/pythonintegration.tests.ps1 -Output Detailed
```

## Suggested Order

1. Run unit tests first (fastest feedback).
2. Run component tests next (adapter subprocess behavior).
3. Run integration tests last (full DSC environment validation).

## Fixtures And Mapping

- `src/get.py`, `src/set.py`, `src/test.py`, `src/export.py` are fixture resources.
- `pyproject.toml` in this folder provides `[tool.dsc.resources]` mapping for `PythonTest/*`.
- Component and integration tests rely on those mappings through resource-path based resolution.

## Troubleshooting

### Unit tests cannot import adapter modules

- Run from repo root: `C:/.../DSC`.
- Use: `python adapters/python/tests/pythonunit.tests.py`.

### Component tests cannot find manifest or adapter entrypoint

- Ensure `adapters/python/tests/pythontest.dsc.manifests.json` exists.
- Ensure `adapters/python/pyDscAdapter/__main__.py` exists.

### Integration tests fail due to Python executable resolution

- Ensure `python3` is available in PATH for DSC-invoked processes.
- Ensure `dsc` CLI is installed and available.
