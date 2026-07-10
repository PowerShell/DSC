# tests/resources/export_only_resource.py
import json
from typing import Dict, Any, Optional


class ExportOnlyResource:
    """A resource class that only implements the export operation for testing purposes."""

    def __init__(self, **kwargs):
        self.name = kwargs.get("name")
        self.version = kwargs.get("version")
        self._exist = kwargs.get("_exist")

    @classmethod
    def from_json(cls, json_str: str, operation: Optional[str] = None):
        data = json.loads(json_str or "{}")
        return cls(**(data if isinstance(data, dict) else {}))

    @staticmethod
    def export(instance: Optional[object] = None) -> Dict[str, Any]:
        """Simulate exporting state with optional filter support."""
        packages = [
            {"name": "alpha", "version": "1.0.0", "_exist": True},
            {"name": "beta", "version": "2.0.0", "_exist": True}
        ]

        if instance is None:
            return {"packages": packages}

        filtered = [
            package
            for package in packages
            if (instance.name is None or package["name"] == instance.name)
            and (instance.version is None or package["version"] == instance.version)
            and (instance._exist is None or package["_exist"] == instance._exist)
        ]

        return {"packages": filtered}
