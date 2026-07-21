# tests/resources/get.py
import json
import logging
from typing import Dict, Any

logger = logging.getLogger("dsc_adapter.resource.PythonTest.Get")

class GetOnlyResource:
    """A resource class that only implements the get operation for testing purposes."""
    _PACKAGE_REGISTRY = {
        "pkg": True,
        "curl": False,
    }
    
    def __init__(self, name: str, version: str = None, **_):
        self.name = name
        self.version = version

    def _simulate_package_exists(self) -> bool:
        """Deterministically simulate package presence using the fixture registry."""
        normalized_name = (self.name or "").strip().lower()
        return bool(self._PACKAGE_REGISTRY.get(normalized_name, False))

    @classmethod
    def from_json(cls, json_str: str, operation: str = None) -> "GetOnlyResource":
        """Instantiate a GetOnlyResource from a JSON string."""
        logger.debug(f"Deserializing input for operation='{operation}'")
        try:
            data = json.loads(json_str or "{}")
        except json.JSONDecodeError as err:
            logger.error(f"Failed to parse JSON input: {err}")
            raise
        name = data.get("name")
        if not isinstance(name, str) or not name.strip():
            logger.error("Input must include a non-empty string 'name'")
            raise ValueError("Input must include a non-empty string 'name'")

        return cls(
            name=name,
            version=data.get("version")
        )

    def get(self) -> Dict[str, Any]:
        """Simulate current state query. For a real resource, query actual state here."""
        logger.info(f"GetOnlyResource.get called for name='{self.name}'")
        actual_exist = self._simulate_package_exists()
        logger.debug(f"Building state with simulated _exist={actual_exist}, version={self.version}")
        # Minimal, deterministic state (what your adapter expects to embed into actualState)
        state = {
            "name": self.name,
            "_exist": actual_exist
        }
        if self.version:
            state["version"] = self.version

        if not state["_exist"]:
            logger.warning(f"Resource '{self.name}' is reported as not existing")

        logger.debug(f"Computed get() state: {state}")
        return state

