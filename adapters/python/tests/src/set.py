# tests/resources/set.py
import json
from typing import Dict, Any, List, Tuple

class SetOnlyResource:
    """A resource class that only implements the set operation for testing purposes."""
    
    def __init__(self, name: str = "pkg", _exist: bool = True, **_):
        self.name = name
        # Future change: _exist will be named differently in the resource and mapped with DSC's _exist.
        self._exist = _exist  # Desired state from input

    @classmethod
    def from_json(cls, json_str: str, operation: str = None) -> "SetOnlyResource":
        """Instantiate a SetOnlyResource from a JSON string."""
        data = json.loads(json_str or "{}")
        return cls(
            name=data.get("name", "pkg"),
            _exist=data.get("_exist", True)
        )

    def get(self) -> Dict[str, Any]:
        """Simulate current state query. For a real resource, query actual state here."""
        # For testing, assume current state is opposite of desired to show a change.
        # In production, read from system.
        return {
            "name": self.name,
            "_exist": not self._exist  # simulate current != desired
        }

    def set(self) -> Tuple[Dict[str, Any], List[str]]:
        """Apply desired state. Return (after_state, diffs)."""
        # Simulate applying the change: after state = desired state
        current_exist = not self._exist  # simulate fetching current (from get())
        after_exist = self._exist        # desired

        diffs: List[str] = []
        if after_exist != current_exist:
            diffs.append("_exist")

        after_state: Dict[str, Any] = {
            "name": self.name,
            "_exist": after_exist
        }
        return after_state, diffs

