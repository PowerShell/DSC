import sys
from cli import main  # TODO: Currently using absolute imports. Switch to relative imports if this is later used as a module in the adapter manifest; otherwise, keep as-is for direct script execution.

if __name__ == "__main__":
    sys.exit(main())
    