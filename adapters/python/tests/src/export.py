# tests/resources/export_only_resource.py
from typing import Dict, Any, Optional

class ExportOnlyResource:
    """A resource class that only implements the export operation for testing purposes."""
    @staticmethod
    def export(instance: Optional[object] = None) -> Dict[str, Any]:
        """Simulate exporting state. In a real resource, gather and return actual state here."""
        #Future change: _exist will be named differently in the resource and mapped with DSC's _exist.
        return {
            "packages": [
                {"name": "alpha", "version": "1.0.0", "_exist": True},
                {"name": "beta", "version": "2.0.0", "_exist": True}
            ]
        }
