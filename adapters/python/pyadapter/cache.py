# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""
Disk cache for the Python DSC adapter.

The adapter (list) uses this cache format: a JSON file keyed by a fingerprint
of all installed distribution name==version pairs.  When the fingerprint matches
and every cached path still exists on disk, the cached entries are used directly;
otherwise the cache is regenerated.

Cache location:
    Windows : %LocalAppData%\\dsc\\PythonListCache.json
    Others  : ~/.dsc/PythonListCache.json

The discovery extension (extensions/python/python.discover.py) no longer uses
a cache; importlib.resources lookup is fast enough at startup.
"""
from __future__ import annotations

import json
import logging
import os
import sys
from importlib.metadata import distributions
from pathlib import Path

_logger = logging.getLogger(__name__)


def _cache_dir() -> Path:
    if sys.platform == "win32":
        base = Path(os.environ.get("LocalAppData") or Path.home())
    else:
        base = Path.home()
    return base / ".dsc"


def dist_fingerprint() -> str:
    """Return a stable string that changes whenever the installed package set changes."""
    return "|".join(sorted(f"{d.name}=={d.version}" for d in distributions()))


class DscCache:
    """Simple JSON cache keyed by installed-distribution fingerprint."""

    def __init__(self, filename: str) -> None:
        self._path = _cache_dir() / filename

    def load(self, fingerprint: str) -> list[dict] | None:
        """
        Return cached entries when the fingerprint matches and all paths exist.
        Returns None on any mismatch or error.
        """
        try:
            data = json.loads(self._path.read_text(encoding="utf-8"))
            if data.get("fingerprint") != fingerprint:
                _logger.debug("Cache miss (fingerprint changed): %s", self._path.name)
                return None
            entries: list[dict] = data.get("manifests", [])
            for entry in entries:
                path = entry.get("manifestPath") or entry.get("path")
                if path and not Path(path).exists():
                    _logger.debug("Cache miss (missing path %s): %s", path, self._path.name)
                    return None
            _logger.debug("Cache hit (%d entries): %s", len(entries), self._path.name)
            return entries
        except Exception as exc:
            _logger.debug("Cache unreadable (%s): %s", exc, self._path.name)
            return None

    def save(self, fingerprint: str, entries: list[dict]) -> None:
        """Persist entries to disk; failures are logged and silently swallowed."""
        try:
            self._path.parent.mkdir(parents=True, exist_ok=True)
            self._path.write_text(
                json.dumps({"fingerprint": fingerprint, "manifests": entries}),
                encoding="utf-8",
            )
            _logger.debug("Cache saved (%d entries): %s", len(entries), self._path.name)
        except Exception as exc:
            _logger.warning("Failed to save cache %s: %s", self._path.name, exc)

    def clear(self) -> None:
        self._path.unlink(missing_ok=True)
        _logger.debug("Cache cleared: %s", self._path.name)


LIST_CACHE = DscCache("PythonListCache.json")
