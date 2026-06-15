# tests/resources/get.py
import json
from typing import Dict, Any

class GetOnlyResource:
    """A resource class that only implements the get operation for testing purposes."""
    
    def __init__(self, name: str = "pkg", version: str = None, _exist: bool = True, **_):
        self.name = name
        self.version = version
        #Future change: _exist will be named differently in the resource and mapped with DSC's _exist.
        self._exist = _exist

    @classmethod
    def from_json(cls, json_str: str, operation: str = None) -> "GetOnlyResource":
        """Instantiate a GetOnlyResource from a JSON string."""
        data = json.loads(json_str or "{}")
        return cls(
            name=data.get("name", "pkg"),
            version=data.get("version"),
            _exist=data.get("_exist", True)
        )

    def get(self) -> Dict[str, Any]:
        """Simulate current state query. For a real resource, query actual state here."""
        # Minimal, deterministic state (what your adapter expects to embed into actualState)
        state = {
            "name": self.name,
            "_exist": bool(self._exist)
        }
        if self.version:
            state["version"] = self.version
        return state

