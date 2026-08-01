#!/usr/bin/env python3
# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""Entry point for invoking the Python DSC adapter.

Provides a clean entry point without relying on __main__.py sys.path
side effects that occur when invoking scripts directly.
"""
import sys
from pathlib import Path

# Add adapter root to Python path
adapter_root = Path(__file__).parent.resolve()
if str(adapter_root) not in sys.path:
    sys.path.insert(0, str(adapter_root))

from pyadapter.cli import main

if __name__ == "__main__":
    sys.exit(main())
