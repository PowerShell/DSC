# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""Python DSC adapter entry point.

Enables ``python -m pyadapter`` as an alternative to the canonical
``python -m pyadapter.cli`` form used by the DSC adapter manifests.
Python's -m invocation adds '' (CWD) to sys.path[0], so both pyadapter
and the bundled ms_dsc are importable without any path manipulation.
"""
import sys

from pyadapter.cli import main

if __name__ == "__main__":
    sys.exit(main())
