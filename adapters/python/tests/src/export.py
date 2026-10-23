# tests/resources/export_only_resource.py
import json
import logging
from typing import Dict, Any, Optional

logger = logging.getLogger("dsc_adapter.resource.PythonTest.Export")


class ExportOnlyResource:
    """A resource class that only implements the export operation for testing purposes."""

    def __init__(self, **kwargs):
        self.name = kwargs.get("name")
        self.version = kwargs.get("version")
        self._exist = kwargs.get("_exist")

    @classmethod
    def from_json(cls, json_str: str, operation: Optional[str] = None):
        logger.debug(f"Deserializing input for operation='{operation}'")
        try:
            data = json.loads(json_str or "{}")
        except json.JSONDecodeError as err:
            logger.error(f"Failed to parse JSON input: {err}")
            raise
        return cls(**(data if isinstance(data, dict) else {}))

    @staticmethod
    def export(instance: Optional[object] = None) -> Dict[str, Any]:
        """Simulate exporting state with optional filter support."""
        packages = [
            {"name": "pkg", "version": "1.0.0", "_exist": True},
            {"name": "curl", "version": "2.0.0", "_exist": True}
        ]

        if instance is None:
            logger.info("ExportOnlyResource.export called with no filter; returning all packages")
            logger.debug(f"Exporting {len(packages)} packages")
            return {"packages": packages}

        logger.info(f"ExportOnlyResource.export called with filter: name={instance.name}, version={instance.version}, _exist={instance._exist}")
        filtered = [
            package
            for package in packages
            if (instance.name is None or package["name"] == instance.name)
            and (instance.version is None or package["version"] == instance.version)
            and (instance._exist is None or package["_exist"] == instance._exist)
        ]

        if not filtered:
            logger.warning(f"No packages matched the provided filter criteria")
        else:
            logger.debug(f"Filtered export returned {len(filtered)} package(s): {[p['name'] for p in filtered]}")

        return {"packages": filtered}
