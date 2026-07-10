# adapters/python/tests/src/test.py
import json
from typing import Dict, Any, List, Tuple

class TestOnlyResource:
    _PACKAGE_REGISTRY = {
        "pkg": True,
        "curl": False,
    }

    def __init__(self, name: str = "pkg", _exist: bool = True, **_):
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
        data = json.loads(json_str or "{}")
        source = data.get("desiredState") if isinstance(data.get("desiredState"), dict) else data
        return cls(
            name=source.get("name"),
            _exist=source.get("_exist", True)
        )

    def test(self) -> Tuple[Dict[str, Any], List[str]]:
        """Simulate testing desired state against current state. Return (actual_state, diffs)."""
        actual_exist = self._simulate_package_exists()
        actual = {"name": self.name, "_exist": actual_exist}
        diffs: List[str] = []
        if actual_exist != self._exist:
            diffs.append("_exist")
        # Contract: (actual_state_dict, diffs_list)
        return actual, diffs
