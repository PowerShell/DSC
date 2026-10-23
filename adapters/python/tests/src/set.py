# tests/resources/set.py
import json
import logging
from typing import Dict, Any, List, Tuple

logger = logging.getLogger("dsc_adapter.resource.PythonTest.Set")

class SetOnlyResource:
    """A resource class that implements get and set operations for testing purposes."""
    
    def __init__(self, name: str, _exist: bool = True, **_):
        self.name = name
        # Future change: _exist will be named differently in the resource and mapped with DSC's _exist.
        self._exist = _exist  # Desired state from input

    @classmethod
    def from_json(cls, json_str: str, operation: str = None) -> "SetOnlyResource":
        """Instantiate a SetOnlyResource from a JSON string."""
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
            _exist=data.get("_exist", True)
        )

    def get(self) -> Dict[str, Any]:
        """Simulate current state query. For a real resource, query actual state here."""
        logger.info(f"SetOnlyResource.get called for name='{self.name}'")
        # For testing, assume current state is opposite of desired to show a change.
        # In production, read from system.
        state = {
            "name": self.name,
            "_exist": not self._exist  # simulate current != desired
        }
        logger.debug(f"Simulated current state: {state}")
        return state

    def set(self) -> Tuple[Dict[str, Any], List[str]]:
        """Apply desired state. Return (after_state, diffs)."""
        logger.info(f"SetOnlyResource.set called for name='{self.name}', desired _exist={self._exist}")
        # Simulate applying the change: after state = desired state
        current_exist = not self._exist  # simulate fetching current (from get())
        after_exist = self._exist        # desired

        diffs: List[str] = []
        if after_exist != current_exist:
            diffs.append("_exist")
            logger.debug(f"Drift detected: current _exist={current_exist}, desired _exist={after_exist}")
        else:
            logger.debug("No drift detected; resource already in desired state")

        after_state: Dict[str, Any] = {
            "name": self.name,
            "_exist": after_exist
        }
        logger.debug(f"Set completed. after_state={after_state}, diffs={diffs}")
        return after_state, diffs

