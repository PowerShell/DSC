#!/bin/env python
# Quick and Dirty Python Pip DSC V3 Resource
# MIT LIcense
# Copyright 2025 Arthur Moore & Higher Education Loan Authority of the State of Missouri (MOHELA)

import subprocess
import sys
import json

from typing import Optional

PACKAGE_SCHEMA = {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "additionalProperties": False,
        "properties": {
            "_exist": {
                "description": "Indicates whether an instance should or does exist.",
                "type": "boolean"
            },
            "name": {
                "type": "string"
            },
            "version": {
                "type": "string"
            },
            "useLatest": {
              "default": "false",
              "description": "Indicate that the latest available version of the package should be installed.",
              "type": "boolean"
            },
        },
        "required": [
            "name"
        ],
        "title": "pip_package",
        "type": "object",
    }

# See https://learn.microsoft.com/en-us/powershell/dsc/reference/schemas/overview?view=dsc-3.0
# See https://learn.microsoft.com/en-us/powershell/dsc/reference/schemas/resource/stdout/?view=dsc-3.0
DSCV3_MANIFEST = {
#        "$schema": "https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json",
        "$schema": "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/resource/manifest.json",
        "type": "Python.Pip/Package",
        "version": "0.1.0",
        "description": "Manage Python packages via pip.",
        "kind": "resource",
        "tags": [
            "windows",
            "python",
            "pip"
        ],
        "exitCodes": {
            "0": "Success",
            "1": "Generic error",
            "2": "Argument(s) Invalid"
        },
        "schema": {
            "embedded": PACKAGE_SCHEMA
# Embed directly to work around this being called for every individual item exported.
#            "command": {
#                "executable": "python.exe",
#                "args": [
#                    "pip.dsc.py",
#                    "schema"
#                ]
#            }
        },
        "get": {
            "executable": "python.exe",
            "args": [
                "pip.dsc.py",
                {
                    "jsonInputArg": "get",
                    "mandatory": True
                }
            ]
        },
        "set": {
            "executable": "python.exe",
            "args": [
                "pip.dsc.py",
                {
                    "jsonInputArg": "set",
                    "mandatory": True
                }
            ]
        },
        "delete": {
            "executable": "python.exe",
            "args": [
                "pip.dsc.py",
                {
                    "jsonInputArg": "delete",
                    "mandatory": True
                }
            ]
        },
        "export": {
            "executable": "python.exe",
            "args": [
                "pip.dsc.py",
                "export",
                {
                    "jsonInputArg": "ignored",
                    "mandatory": False
                }
            ]
        }
    }

def _install(name: str, version: Optional[str], upgrade: bool):
    to_run = [
        sys.executable,
        "-m",
        "pip",
        "--no-input",
        "--disable-pip-version-check",
        "install"
    ]
    if upgrade:
        to_run += ["--upgrade"]
    final_command = to_run + [name if version==None else f"{name}=={version}"]
    subprocess.check_call(final_command)

def _list_installed():
    return json.loads(subprocess.check_output([
        sys.executable,
        "-m",
        "pip",
        "--no-input",
        "--disable-pip-version-check",
        "list",
        "--format=json"
    ]))

def manifest():
    return json.dumps(DSCV3_MANIFEST)

def set(data):
    package = data["name"]
    if "version" in data.keys():
        package += "==" + data["version"]
    _install(data["name"], data.get("version"), data.get("useLatest", False))

def delete(name: str):
    subprocess.check_call([
        sys.executable,
        "-m",
        "pip",
        "--no-input",
        "--disable-pip-version-check",
        "uninstall",
        name
    ])

def get(name: str):
    installed = _list_installed()
    filtered = list(filter(lambda d: d["name"] == name, installed))
    package_exists = len(filtered) > 0
    package_data = filtered[0] if package_exists else {"name": name}
    package_data["_exist"] = package_exists
    return json.dumps(package_data)

def export():
    """Returns same as get, in JSONL (one JSON object per line)"""
    installed = _list_installed()
    return "\n".join(map(json.dumps, installed))

def schema():
    return json.dumps(PACKAGE_SCHEMA)

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Must have at least one argument (get, set, delete, schema, export, manifest)!")
        exit(2)
    if sys.argv[1] == "manifest":
        print(manifest())
        exit(0)
    if sys.argv[1] == "schema":
        print(schema())
        exit(0)
    if sys.argv[1] == "export":
        print(export())
        exit(0)

    if len(sys.argv) < 3:
        print("Must have at least two arguments!")
        exit(2)
    in_json = json.loads(sys.argv[2])

    if sys.argv[1] == "get":
        print(get(in_json["name"]))
    if sys.argv[1] == "set":
        set(in_json)
    if sys.argv[1] == "delete":
        delete(in_json["name"])
