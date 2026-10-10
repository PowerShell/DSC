import importlib.util
from pathlib import Path
from typing import Any, Dict

try:
    import tomllib as toml_parser
except ModuleNotFoundError:
    toml_parser = None


def _load_pyproject_data(pyproject_path: Path) -> Dict[str, Any]:
    """Load pyproject.toml into a dictionary when possible."""
    pyproject_path = Path(pyproject_path)
    if not pyproject_path.exists() or toml_parser is None:
        return {}

    try:
        with pyproject_path.open("rb") as f:
            data = toml_parser.load(f)
        return data if isinstance(data, dict) else {}
    except Exception:
        return {}


def get_project_metadata_from_pyproject(pyproject_path: Path) -> Dict[str, str]:
    """Parse [project] metadata from pyproject.toml."""
    pyproject_path = Path(pyproject_path)

    default_metadata = {
        "version": "",
        "description": "",
        "author": "",
    }

    if not pyproject_path.exists():
        return default_metadata

    data = _load_pyproject_data(pyproject_path)
    if data:
        project = data.get("project", {})
        if not isinstance(project, dict):
            return default_metadata

        authors = project.get("authors", [])
        author = ""
        if isinstance(authors, list):
            for item in authors:
                if isinstance(item, dict) and item.get("name"):
                    author = str(item["name"])
                    break

        return {
            "version": str(project.get("version", "") or ""),
            "description": str(project.get("description", "") or ""),
            "author": author,
        }

    try:
        content = pyproject_path.read_text(encoding="utf-8")
    except Exception:
        return default_metadata

    in_project = False
    in_authors = False
    metadata = dict(default_metadata)

    for raw_line in content.splitlines():
        stripped = raw_line.strip()
        if stripped == "[project]":
            in_project = True
            in_authors = False
            continue
        if in_project and stripped.startswith("[") and stripped != "[[project.authors]]":
            if not in_authors:
                break
        if not in_project:
            continue

        if stripped == "[[project.authors]]":
            in_authors = True
            continue

        if in_authors:
            if stripped.startswith("name") and "=" in stripped and not metadata["author"]:
                _, value = stripped.split("=", 1)
                metadata["author"] = value.strip().strip('"\'')
            elif stripped.startswith("[[") or (stripped.startswith("[") and stripped != "[[project.authors]]"):
                in_authors = False
            continue

        if stripped.startswith("version") and "=" in stripped:
            _, value = stripped.split("=", 1)
            metadata["version"] = value.strip().strip('"\'')
        elif stripped.startswith("description") and "=" in stripped:
            _, value = stripped.split("=", 1)
            metadata["description"] = value.strip().strip('"\'')

    return metadata

def get_class_map_from_pyproject(pyproject_path: Path) -> Dict[str, str]:
    """
    Parse [tool.dsc.resources] section from pyproject.toml.
    Returns: {"ResourceType": "ClassName", ...}
    No external dependencies required.
    """
    pyproject_path = Path(pyproject_path)

    if not pyproject_path.exists():
        return {}

    data = _load_pyproject_data(pyproject_path)
    if data:
        resources = (
            data.get("tool", {})
                .get("dsc", {})
                .get("resources", {})
        )
        if isinstance(resources, dict):
            class_map: Dict[str, str] = {}
            for resource_type, resource_value in resources.items():
                if isinstance(resource_value, dict):
                    class_name = resource_value.get("class", "")
                    if class_name:
                        class_map[str(resource_type)] = str(class_name)
                elif resource_value is not None:
                    class_map[str(resource_type)] = str(resource_value)
            if class_map:
                return class_map
    
    try:
        content = pyproject_path.read_text(encoding="utf-8")
    except Exception:
        return {}

    class_map = {}
    in_section = False
    
    for line in content.splitlines():
        stripped = line.strip()
        if stripped == "[tool.dsc.resources]":
            in_section = True
            continue
        if in_section:
            if stripped.startswith("["):
                break
            if "=" in stripped and not stripped.startswith("#"):
                key, val = stripped.split("=", 1)
                key = key.strip().strip('"\'')
                val = val.strip().strip('"\'')
                class_map[key] = val
    
    return class_map


def get_resource_metadata_from_pyproject(pyproject_path: Path) -> Dict[str, Dict[str, str]]:
    """Parse per-resource metadata from pyproject.toml."""
    pyproject_path = Path(pyproject_path)

    if not pyproject_path.exists():
        return {}

    data = _load_pyproject_data(pyproject_path)
    if not data:
        return {}

    resources = (
        data.get("tool", {})
            .get("dsc", {})
            .get("resources", {})
    )
    if not isinstance(resources, dict):
        return {}

    metadata: Dict[str, Dict[str, str]] = {}
    for resource_type, resource_value in resources.items():
        if not isinstance(resource_value, dict):
            continue

        metadata[str(resource_type)] = {
            "version": str(resource_value.get("version", "") or ""),
            "description": str(resource_value.get("description", "") or ""),
            "author": str(resource_value.get("author", "") or ""),
        }

    return metadata

def import_class_from_file(resource_path: Path, resource_type: str, class_name: str) -> type:
    """Dynamically import a class from a given file path."""
    module_name = f"dsc_{resource_type.replace('/', '_').replace('.', '_').lower()}" #if resource_type else f"dsc_{resource_path.stem.lower()}"
    spec = importlib.util.spec_from_file_location(module_name, str(resource_path))
    if not spec or not spec.loader:
        raise ImportError(f"Unable to load module '{resource_path}'")

    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    try:
        return getattr(mod, class_name)
    except AttributeError as e:
        raise ImportError(f"Class '{class_name}' not found in '{resource_path}': {e}")

