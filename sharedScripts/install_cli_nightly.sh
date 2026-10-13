#!/usr/bin/env bash
set -e

# Default values
INSTALL_PATH=""
REPO="PowerShell/DSC"
BRANCH="main"
RUN_ID=""

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --install-path)
      INSTALL_PATH="$2"
      shift 2
      ;;
    --repo)
      REPO="$2"
      shift 2
      ;;
    --branch)
      BRANCH="$2"
      shift 2
      ;;
    --run-id)
      RUN_ID="$2"
      shift 2
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Check for GitHub CLI
if ! command -v gh &> /dev/null; then
  echo "Error: GitHub CLI (gh) is not installed."
  echo "Please install it from: https://cli.github.com/"
  exit 1
fi

# Detect platform
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
  PLATFORM="linux"
elif [[ "$OSTYPE" == "darwin"* ]]; then
  PLATFORM="macos"
else
  echo "Error: Unsupported OS: $OSTYPE"
  exit 1
fi

# Set default install path if not provided
if [[ -z "$INSTALL_PATH" ]]; then
  INSTALL_PATH="$HOME/.dsc/bin"
fi

# Fetch run ID if not provided
if [[ -z "$RUN_ID" ]]; then
  echo "Fetching latest successful build for branch '$BRANCH'..."
  RUN_ID=$(gh run list -R "$REPO" --branch "$BRANCH" --workflow rust --status success -L 1 --json databaseId -q ".[0].databaseId")
  
  if [[ -z "$RUN_ID" ]]; then
    echo "Error: Failed to find a successful build to install from"
    exit 1
  fi
fi

echo "Downloading artifacts from run $RUN_ID..."

# Create temporary directory
TMP_DIR=$(mktemp -d)
trap "rm -rf $TMP_DIR" EXIT

# Download artifact
gh run download -R "$REPO" "$RUN_ID" -n "${PLATFORM}-bin" --dir "$TMP_DIR"

# Find the tar file
TAR_FILE=$(find "$TMP_DIR" -name "*.tar.gz" -o -name "*.tgz" | head -n 1)

if [[ -z "$TAR_FILE" ]]; then
  echo "Error: Failed to find downloaded artifact"
  exit 1
fi

# Extract
echo "Extracting archive..."
tar -xzf "$TAR_FILE" -C "$TMP_DIR"

# Install
echo "Installing to $INSTALL_PATH..."
mkdir -p "$INSTALL_PATH"
mv "$TMP_DIR/dsc" "$INSTALL_PATH/dsc"
chmod +x "$INSTALL_PATH/dsc"

# Get version
VERSION=$("$INSTALL_PATH/dsc" --version 2>&1 || echo "unknown")

echo ""
echo "Successfully installed DSC to $INSTALL_PATH/dsc"
echo "Version: $VERSION"
echo "From: https://github.com/$REPO/actions/runs/$RUN_ID"
echo ""
echo "To use DSC, add the following to your PATH:"
echo "  export PATH=\"\$PATH:$INSTALL_PATH\""
