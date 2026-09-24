# adapters/python/tests/src/test.py
import json
import logging
from typing import Dict, Any, List, Tuple

logger = logging.getLogger("dsc_adapter.resource.PythonTest.Test")

class TestOnlyResource:
    _PACKAGE_REGISTRY = {
        "pkg": True,
        "curl": False,
    }

    def __init__(self, name: str, _exist: bool = True, **_):
        self.name = name
        # Future change: _exist will be named differently in the resource and mapped with DSC's _exist.
        self._exist = bool(_exist)  # desired state from input

    def _simulate_package_exists(self) -> bool:
        """Deterministically simulate package presence using the fixture registry."""
        normalized_name = (self.name or "").strip().lower()
        return bool(self._PACKAGE_REGISTRY.get(normalized_name, False))

    @classmethod
    def from_json(cls, json_str: str, operation: str = None) -> "TestOnlyResource":
        """Instantiate a TestOnlyResource from a JSON string.

        Accepts both flat input {"name": ..., "_exist": ...} and DSC-wrapped
        input {"desiredState": {"name": ..., "_exist": ...}} so behavior is
        consistent across component and integration test paths.
        """
        logger.debug(f"Deserializing input for operation='{operation}'")
        try:
            data = json.loads(json_str or "{}")
        except json.JSONDecodeError as err:
            logger.error(f"Failed to parse JSON input: {err}")
            raise

        source = data.get("desiredState") if isinstance(data.get("desiredState"), dict) else data
        name = source.get("name")
        if not isinstance(name, str) or not name.strip():
            logger.error("Input must include a non-empty string 'name'")
            raise ValueError("Input must include a non-empty string 'name'")

        return cls(
            name=name,
            _exist=source.get("_exist", True)
        )

    def test(self) -> Tuple[Dict[str, Any], List[str]]:
        """Simulate testing desired state against current state. Return (actual_state, diffs)."""
        logger.info(f"TestOnlyResource.test called for name='{self.name}', desired _exist={self._exist}")
        actual_exist = self._simulate_package_exists()
        logger.debug(f"Simulated actual _exist={actual_exist} for name='{self.name}'")
        actual = {"name": self.name, "_exist": actual_exist}
        diffs: List[str] = []
        if actual_exist != self._exist:
            diffs.append("_exist")
            logger.debug(f"Drift detected: actual _exist={actual_exist}, desired _exist={self._exist}")
        else:
            logger.debug("No drift: resource is in desired state")

        if not actual_exist:
            logger.warning(f"Package '{self.name}' does not exist in the registry")

        return actual, diffs
