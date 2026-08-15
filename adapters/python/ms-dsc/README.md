# ms-dsc

Python SDK for writing [Microsoft DSC v3](https://github.com/PowerShell/DSC) resources.

## Installation

For **development**, install the package in editable mode with build-time dependencies:

```bash
pip install -e .
```

The build system will automatically install `ms-dsc` from PyPI for schema generation and IDE support.

**Note:** Resources deployed to production should declare `ms-dsc` only as a build-time dependency (`[build-system] requires`), not in `dependencies`. DSC provides `ms-dsc` at runtime via the bundled copy.

## Quick start

```python
# my_package/my_resource.py
from __future__ import annotations

import logging
from dataclasses import dataclass, field
from collections.abc import Iterator

from ms_dsc import DscResource, dsc_resource, SetResult, TestResult
from ms_dsc.metadata import SetReturn, TestReturn
from ms_dsc.schema import DataclassSchemaProvider

logger = logging.getLogger(__name__)


@dataclass
class MySchema:
    name: str = field(metadata={"description": "Unique name of the managed instance."})
    _exist: bool = field(default=True, metadata={"description": "Whether the instance should exist."})


@dsc_resource(
    type="MyPackage/MyResource",
    version="1.0.0",
    description="Manages a widget",
    set_return=SetReturn.STATE_AND_DIFF,
    test_return=TestReturn.STATE_AND_DIFF,
)
class MyResource(DscResource[MySchema]):
    schema_provider = DataclassSchemaProvider(MySchema)

    def get(self, instance: MySchema) -> MySchema:
        logger.info("Getting %s", instance.name)
        # Query actual system state here.
        return MySchema(name=instance.name, _exist=True)

    def set(self, instance: MySchema) -> SetResult[MySchema]:
        # Apply desired state here.
        return SetResult(
            actual_state=MySchema(name=instance.name, _exist=instance._exist),
            changed_properties=["_exist"],
        )

    def test(self, instance: MySchema) -> TestResult[MySchema]:
        actual = self.get(instance)
        diffs = ["_exist"] if actual._exist != instance._exist else []
        return TestResult(actual_state=actual, differing_properties=diffs)

    def delete(self, instance: MySchema) -> None:
        pass  # Remove the managed instance here.

    def export(self, instance: MySchema | None) -> Iterator[MySchema]:
        yield MySchema(name="example", _exist=True)
```

Register the resource as an entry point in `pyproject.toml` using the **plugin pattern**:

```toml
[build-system]
requires = ["hatchling", "ms-dsc"]  # Build-time: for schema generation
build-backend = "hatchling.build"

[project]
name = "my-package"
dependencies = []                    # Runtime: EMPTY (DSC provides ms-dsc)

[project.entry-points."microsoft.dsc.resources"]
"MyPackage/MyResource" = "my_package.my_resource:MyResource"
```

**Why?** DSC resources are plugins. They're intended for use within DSC (which bundles ms-dsc), not as standalone packages.
This design:
- Prevents duplicate dependencies when shipping with existing packages
- Allows teams to add DSC resources to their infrastructure without forcing DSC on all users
- Follows the same pattern as pytest plugins, Flask extensions, and Jupyter adapters

Generate an adapted resource manifest so DSC can discover the resource without
running the adapter's `list` operation:

```bash
dsc-gen manifest --out my_package/dsc/
```

Include the generated manifests in your wheel:

```toml
[tool.setuptools.package-data]
my_package = ["dsc/*.json"]
```

## API reference

### `@dsc_resource` decorator

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `type` | `str` | required | Fully-qualified resource type, e.g. `"MyPackage/MyResource"` |
| `version` | `str` | required | Semantic version, e.g. `"1.0.0"` |
| `description` | `str` | `""` | Human-readable description |
| `tags` | `tuple[str, ...]` | `()` | Discovery tags |
| `set_return` | `SetReturn` | `SetReturn.STATE` | Whether `set()` returns state only or state + diff |
| `test_return` | `TestReturn` | `TestReturn.STATE` | Whether `test()` returns state only or state + diff |

### Capability Protocols

| Protocol | Method signature | Description |
|----------|-----------------|-------------|
| `Gettable[T]` | `get(instance: T) -> T` | Return current state |
| `Settable[T]` | `set(instance: T) -> SetResult[T]` | Enforce desired state |
| `Testable[T]` | `test(instance: T) -> TestResult[T]` | Compare actual vs desired |
| `Deletable[T]` | `delete(instance: T) -> None` | Remove the managed entity |
| `Exportable[T]` | `export(instance: T \| None) -> Iterator[T]` | Enumerate all instances |

### Result types

```python
@dataclass
class SetResult(Generic[T]):
    actual_state: T
    changed_properties: list[str] | None = None  # None = STATE mode

@dataclass
class TestResult(Generic[T]):
    actual_state: T
    differing_properties: list[str] | None = None  # None = STATE mode
```

### Schema providers

| Provider | Requires | Usage |
|----------|----------|-------|
| `DataclassSchemaProvider(T)` | stdlib only | `schema_provider = DataclassSchemaProvider(MySchema)` |
| `PydanticSchemaProvider(T)` | `pip install pydantic` | `schema_provider = PydanticSchemaProvider(MyModel)` |

### `dsc-gen manifest` CLI

```
dsc-gen manifest [--out DIR] [--pyproject FILE]
```

Reads `[project.entry-points."microsoft.dsc.resources"]` from `pyproject.toml`,
imports each resource class, generates a `*.dsc.adaptedResource.json` file per
resource, and writes them to `--out` (default: `<package_name>/dsc/`).

### Logging

Resource authors use Python's standard `logging` module. The DSC adapter
automatically routes log records to DSC's structured stderr JSON format:

```python
import logging
logger = logging.getLogger(__name__)

def get(self, instance):
    logger.info("Getting %s", instance.name)  # → {"info": "my_module: Getting foo"}
    ...
```

Log level is controlled by the `DSC_TRACE_LEVEL` environment variable
(`trace`/`debug`/`info`/`warn`/`error`/`critical`).

## Hatchling build hook

For [Hatchling](https://hatch.pypa.io/latest/)-based projects, the build hook
runs `dsc-gen manifest` automatically at wheel-build time:

```toml
[build-system]
requires = ["hatchling", "ms-dsc"]
build-backend = "hatchling.build"

[tool.hatch.build.hooks.dsc]
# output_dir = "my_package/dsc"  # defaults to <package_name>/dsc/
```

## Requirements

- Python ≥ 3.11
- `ms-dsc` has **zero mandatory runtime dependencies**
- Pydantic is optional (only required for `PydanticSchemaProvider`)
