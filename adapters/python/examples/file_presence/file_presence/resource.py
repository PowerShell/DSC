# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""
Example DSC resource: Example/FilePresence
==========================================

Ensures that a file at a given path either exists (is created as empty) or
does not exist (is removed).

This example illustrates the recommended pattern for writing a Python DSC
resource using the ms-dsc SDK:

  1. Define the schema as a @dataclass.
  2. Decorate the resource class with @dsc_resource.
  3. Implement the capability methods (get, set, test, delete).
  4. Register the class as an entry point in pyproject.toml.

Usage (via DSC CLI after installing the package):

    dsc resource get --resource Example/FilePresence --input '{"path":"/tmp/hello.txt"}'
    dsc resource set --resource Example/FilePresence --input '{"path":"/tmp/hello.txt","_exist":true}'
    dsc resource test --resource Example/FilePresence --input '{"path":"/tmp/hello.txt","_exist":true}'
    dsc resource delete --resource Example/FilePresence --input '{"path":"/tmp/hello.txt"}'
"""
from __future__ import annotations

import logging
from collections.abc import Iterator
from dataclasses import dataclass, field
from pathlib import Path

from ms_dsc import DscResource, SetResult, TestResult, dsc_resource
from ms_dsc.metadata import SetReturn, TestReturn
from ms_dsc.protocols import Deletable, Exportable, Gettable, Settable, Testable
from ms_dsc.schema import DataclassSchemaProvider

_logger = logging.getLogger(__name__)


@dataclass
class FileSchema:
    """Schema for the Example/FilePresence resource."""

    path: str = field(
        metadata={"description": "Absolute path of the file to manage.", "title": "Path"}
    )
    _exist: bool = field(
        default=True,
        metadata={"description": "Whether the file should exist.", "title": "Exist"},
    )


@dsc_resource(
    type="Example/FilePresence",
    version="1.0.0",
    description="Ensures a file exists or does not exist at the given path.",
    tags=["file", "cross-platform"],
    set_return=SetReturn.STATE_AND_DIFF,
    test_return=TestReturn.STATE_AND_DIFF,
)
class FilePresenceResource(DscResource[FileSchema], Gettable, Settable, Testable, Deletable, Exportable):
    """
    Manages the presence of a single file.

    get    → returns actual file existence state
    set    → creates an empty file or removes it to match _exist
    test   → compares actual existence against desired; reports differing properties
    delete → unconditionally removes the file (equivalent to set _exist=False)
    export → enumerates all files matching the filter
    """

    schema_provider = DataclassSchemaProvider(FileSchema)

    def get(self, instance: FileSchema) -> FileSchema:
        exists = Path(instance.path).exists()
        _logger.info("get(%s) → _exist=%s", instance.path, exists)
        return FileSchema(path=instance.path, _exist=exists)

    def set(self, instance: FileSchema) -> SetResult[FileSchema]:
        path = Path(instance.path)
        before = path.exists()
        changed: list[str] = []

        if instance._exist and not before:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.touch()
            changed.append("_exist")
            _logger.info("set(%s): created", instance.path)
        elif not instance._exist and before:
            path.unlink()
            changed.append("_exist")
            _logger.info("set(%s): removed", instance.path)
        else:
            _logger.info("set(%s): no change needed", instance.path)

        return SetResult(
            actual_state=FileSchema(path=instance.path, _exist=path.exists()),
            changed_properties=changed,
        )

    def test(self, instance: FileSchema) -> TestResult[FileSchema]:
        actual = self.get(instance)
        diffs = ["_exist"] if actual._exist != instance._exist else []
        return TestResult(actual_state=actual, differing_properties=diffs)

    def delete(self, instance: FileSchema) -> None:
        path = Path(instance.path)
        if path.exists():
            path.unlink()
            _logger.info("delete(%s): removed", instance.path)

    def export(self, instance: FileSchema | None) -> Iterator[FileSchema]:
        """
        Enumerate files in the parent directory of the requested path.

        When ``instance`` is provided, its ``path`` is treated as a directory
        to scan; if omitted, the current working directory is scanned.
        Yields one ``FileSchema`` per regular file found.
        """
        scan_dir = Path(instance.path) if instance is not None else Path.cwd()
        _logger.info("export(%s)", scan_dir)
        try:
            for entry in scan_dir.iterdir():
                if entry.is_file():
                    yield FileSchema(path=str(entry), _exist=True)
        except PermissionError as exc:
            _logger.warning("export: cannot read %s: %s", scan_dir, exc)
