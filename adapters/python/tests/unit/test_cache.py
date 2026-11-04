# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""Unit tests for pyadapter.cache — DscCache and dist_fingerprint."""
from __future__ import annotations

import json

import pytest

from pyadapter.cache import DscCache, dist_fingerprint


class TestDscCache:
    def _make(self, tmp_path) -> DscCache:
        c = DscCache("test.json")
        c._path = tmp_path / "test.json"
        return c

    def test_load_returns_none_when_file_missing(self, tmp_path):
        c = self._make(tmp_path)
        assert c.load("any-fingerprint") is None

    def test_save_and_load_roundtrip(self, tmp_path):
        c = self._make(tmp_path)
        fake_path = tmp_path / "resource.json"
        fake_path.write_text("{}")  # must exist for path-validity check
        entries = [{"manifestPath": str(fake_path)}]
        c.save("fp1", entries)
        result = c.load("fp1")
        assert result == entries

    def test_load_returns_none_on_fingerprint_mismatch(self, tmp_path):
        c = self._make(tmp_path)
        c.save("fp1", [])
        assert c.load("fp2") is None

    def test_load_returns_none_when_manifest_path_missing(self, tmp_path):
        c = self._make(tmp_path)
        entries = [{"manifestPath": str(tmp_path / "does_not_exist.json")}]
        c.save("fp1", entries)
        # Path does not exist → cache invalid
        assert c.load("fp1") is None

    def test_load_returns_entries_without_manifest_path(self, tmp_path):
        """Entries without a manifestPath key are allowed (e.g., list cache entries)."""
        c = self._make(tmp_path)
        entries = [{"type": "Test/Resource", "version": "1.0.0"}]
        c.save("fp1", entries)
        result = c.load("fp1")
        assert result == entries

    def test_clear_removes_file(self, tmp_path):
        c = self._make(tmp_path)
        c.save("fp1", [])
        assert c._path.exists()
        c.clear()
        assert not c._path.exists()

    def test_clear_is_idempotent_when_missing(self, tmp_path):
        c = self._make(tmp_path)
        c.clear()  # should not raise

    def test_load_handles_corrupt_json(self, tmp_path):
        c = self._make(tmp_path)
        c._path.write_text("THIS IS NOT JSON", encoding="utf-8")
        assert c.load("any") is None

    def test_save_creates_parent_directory(self, tmp_path):
        nested = tmp_path / "a" / "b" / "c"
        c = DscCache("test.json")
        c._path = nested / "test.json"
        c.save("fp1", [])
        assert c._path.exists()

    def test_multiple_entries(self, tmp_path):
        c = self._make(tmp_path)
        p1, p2 = tmp_path / "m1.json", tmp_path / "m2.json"
        p1.write_text("{}"); p2.write_text("{}")
        entries = [{"manifestPath": str(p1)}, {"manifestPath": str(p2)}]
        c.save("fp1", entries)
        result = c.load("fp1")
        assert len(result) == 2


class TestDistFingerprint:
    def test_returns_string(self):
        fp = dist_fingerprint()
        assert isinstance(fp, str)

    def test_non_empty(self):
        # At least Python itself is installed in this environment.
        fp = dist_fingerprint()
        assert len(fp) > 0

    def test_consistent_across_calls(self):
        assert dist_fingerprint() == dist_fingerprint()

    def test_contains_name_version_pairs(self):
        fp = dist_fingerprint()
        # Should contain at least one "name==version" pair.
        assert "==" in fp
