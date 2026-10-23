# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""
Example DSC resource: Example/File
===================================

Manages a file at a given path — its existence and text content.

Properties
----------
path    : Absolute path of the file to manage.
content : Desired text content of the file (UTF-8).  Ignored when _exist=False.
_exist  : Whether the file should exist.  Defaults to True.

Usage (via DSC CLI after installing the package):

    dsc resource get  --resource Example/File --input '{"path":"C:\\temp\\demo.txt"}'
    dsc resource set  --resource Example/File --input '{"path":"C:\\temp\\demo.txt","content":"Hello!","_exist":true}'
    dsc resource test --resource Example/File --input '{"path":"C:\\temp\\demo.txt","content":"Hello!","_exist":true}'
    dsc resource delete --resource Example/File --input '{"path":"C:\\temp\\demo.txt"}'
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
    """Schema for the Example/File resource."""

    path: str = field(
        metadata={"description": "Absolute path of the file to manage.", "title": "Path"}
    )
    content: str = field(
        default="",
        metadata={
            "description": "Text content of the file (UTF-8).  Ignored when _exist is false.",
            "title": "Content",
        },
    )
    _exist: bool = field(
        default=True,
        metadata={"description": "Whether the file should exist.", "title": "Exist"},
    )


@dsc_resource(
    type="Example/File",
    version="1.0.0",
    description="Manages the existence and text content of a file.",
    tags=["file", "cross-platform"],
    set_return=SetReturn.STATE_AND_DIFF,
    test_return=TestReturn.STATE_AND_DIFF,
)
class FileResource(DscResource[FileSchema], Gettable, Settable, Testable, Deletable, Exportable):
    """
    Manages a single file on disk.

    get    → returns the actual existence and content of the file
    set    → creates/overwrites or deletes the file to match desired state
    test   → reports which properties differ from desired
    delete → unconditionally removes the file (equivalent to set _exist=False)
    export → yields a FileSchema for every regular file under the directory
             part of the path, or the single file if the path is a file
    """

    schema_provider = DataclassSchemaProvider(FileSchema)

    # ------------------------------------------------------------------
    # Helpers
    # ------------------------------------------------------------------

    @staticmethod
    def _read(path: Path) -> FileSchema:
        if path.is_file():
            try:
                content = path.read_text(encoding="utf-8")
            except OSError as exc:
                _logger.warning("Could not read %s: %s", path, exc)
                content = ""
            return FileSchema(path=str(path), content=content, _exist=True)
        return FileSchema(path=str(path), content="", _exist=False)

    # ------------------------------------------------------------------
    # Capability methods
    # ------------------------------------------------------------------

    def get(self, instance: FileSchema) -> FileSchema:
        actual = self._read(Path(instance.path))
        _logger.info("get(%s) → _exist=%s", instance.path, actual._exist)
        return actual

    def set(self, instance: FileSchema) -> SetResult[FileSchema]:
        path = Path(instance.path)
        before = self._read(path)
        changed: list[str] = []

        if instance._exist:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(instance.content, encoding="utf-8")
            if not before._exist:
                changed.append("_exist")
            if before.content != instance.content:
                changed.append("content")
        else:
            if before._exist:
                path.unlink()
                changed.append("_exist")

        actual = self._read(path)
        _logger.info("set(%s) changed=%s", instance.path, changed)
        return SetResult(actual_state=actual, changed_properties=changed)

    def test(self, instance: FileSchema) -> TestResult[FileSchema]:
        actual = self._read(Path(instance.path))
        diffs: list[str] = []

        if actual._exist != instance._exist:
            diffs.append("_exist")
        elif actual._exist and instance._exist and actual.content != instance.content:
            diffs.append("content")

        _logger.info("test(%s) diffs=%s", instance.path, diffs)
        return TestResult(actual_state=actual, differing_properties=diffs)

    def delete(self, instance: FileSchema) -> None:
        path = Path(instance.path)
        if path.is_file():
            path.unlink()
            _logger.info("delete(%s)", instance.path)

    def export(self, instance: FileSchema | None) -> Iterator[FileSchema]:
        if instance is None:
            return
        target = Path(instance.path)
        if target.is_file():
            yield self._read(target)
        elif target.is_dir():
            for child in sorted(target.rglob("*")):
                if child.is_file():
                    yield self._read(child)


# ---------------------------------------------------------------------------
# Example/ManagedFile — identical logic, but backed by a pre-built
# .dsc.adaptedResource.json manifest in file_resource/dsc/.
# Discovered by the Python extension (python.discover.py) rather than by
# the adapter's runtime list operation.
# ---------------------------------------------------------------------------

@dsc_resource(
    type="Example/ManagedFile",
    version="1.0.0",
    description=(
        "Manages the existence and text content of a file. "
        "Discovered via a pre-built adapted resource manifest in the pip package."
    ),
    tags=["file", "cross-platform", "manifest"],
    set_return=SetReturn.STATE_AND_DIFF,
    test_return=TestReturn.STATE_AND_DIFF,
)
class ManagedFileResource(DscResource[FileSchema]):
    """
    Same file management capability as Example/File, but this class is
    registered with a pre-built .dsc.adaptedResource.json in file_resource/dsc/.
    DSC discovers it via the python.discover.py extension rather than via the
    adapter's list operation.
    """

    schema_provider = DataclassSchemaProvider(FileSchema)

    @staticmethod
    def _read(path: Path) -> FileSchema:
        if path.is_file():
            try:
                content = path.read_text(encoding="utf-8")
            except OSError as exc:
                _logger.warning("Could not read %s: %s", path, exc)
                content = ""
            return FileSchema(path=str(path), content=content, _exist=True)
        return FileSchema(path=str(path), content="", _exist=False)

    def get(self, instance: FileSchema) -> FileSchema:
        actual = self._read(Path(instance.path))
        _logger.info("ManagedFile get(%s) → _exist=%s", instance.path, actual._exist)
        return actual

    def set(self, instance: FileSchema) -> SetResult[FileSchema]:
        path = Path(instance.path)
        before = self._read(path)
        changed: list[str] = []
        if instance._exist:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(instance.content, encoding="utf-8")
            if not before._exist:
                changed.append("_exist")
            if before.content != instance.content:
                changed.append("content")
        else:
            if before._exist:
                path.unlink()
                changed.append("_exist")
        actual = self._read(path)
        _logger.info("ManagedFile set(%s) changed=%s", instance.path, changed)
        return SetResult(actual_state=actual, changed_properties=changed)

    def test(self, instance: FileSchema) -> TestResult[FileSchema]:
        actual = self._read(Path(instance.path))
        diffs: list[str] = []
        if actual._exist != instance._exist:
            diffs.append("_exist")
        elif actual._exist and instance._exist and actual.content != instance.content:
            diffs.append("content")
        _logger.info("ManagedFile test(%s) diffs=%s", instance.path, diffs)
        return TestResult(actual_state=actual, differing_properties=diffs)

    def delete(self, instance: FileSchema) -> None:
        path = Path(instance.path)
        if path.is_file():
            path.unlink()
            _logger.info("ManagedFile delete(%s)", instance.path)

    def export(self, instance: FileSchema | None) -> Iterator[FileSchema]:
        if instance is None:
            return
        target = Path(instance.path)
        if target.is_file():
            yield self._read(target)
        elif target.is_dir():
            for child in sorted(target.rglob("*")):
                if child.is_file():
                    yield self._read(child)
