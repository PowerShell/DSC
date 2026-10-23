import json
from typing import Any, Dict

def parse_json(s: str) -> Dict[str, Any]:
    """Safely parse JSON string, returning empty dict on failure."""
    try:
        return json.loads(s or "{}")
    except Exception:
        return {}
