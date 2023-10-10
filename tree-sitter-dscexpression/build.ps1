# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

# check if tools are installed

function Invoke-NativeCommand($cmd) {
    Invoke-Expression $cmd
    if ($LASTEXITCODE -ne 0) {
        throw "Command $cmd failed with exit code $LASTEXITCODE"
    }
}

Invoke-NativeCommand 'npx tree-sitter generate'
Invoke-NativeCommand 'node-gyp configure'
Invoke-NativeCommand 'node-gyp build'
Invoke-NativeCommand 'npx tree-sitter test'
