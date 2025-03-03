# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

param(
    [switch]$UpdatePackages
)

function Invoke-NativeCommand($cmd) {
    Invoke-Expression $cmd
    if ($LASTEXITCODE -ne 0) {
        throw "Command $cmd failed with exit code $LASTEXITCODE"
    }
}

$env:TREE_SITTER_VERBOSE=1

if ($null -eq (Get-Command tree-sitter -ErrorAction Ignore)) {
    cargo install tree-sitter-cli
}

Invoke-NativeCommand 'tree-sitter generate --build'
Invoke-NativeCommand 'tree-sitter test'
