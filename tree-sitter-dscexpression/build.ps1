# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

# check if tools are installed

function Invoke-NativeCommand($cmd) {
    Invoke-Expression $cmd
    if ($LASTEXITCODE -ne 0) {
        throw "Command $cmd failed with exit code $LASTEXITCODE"
    }
}

if ($null -eq (Get-Command npm -ErrorAction Ignore)) {
    throw "npm is not installed"
}

npm list tree-sitter-cli
if ($LASTEXITCODE -ne 0) {
    npm install tree-sitter-cli
}

npm list node-gyp
if ($LASTEXITCODE -ne 0) {
    npm install node-gyp
}

Invoke-NativeCommand 'npx tree-sitter generate'
Invoke-NativeCommand 'node-gyp configure'
Invoke-NativeCommand 'node-gyp build'
Invoke-NativeCommand 'npx tree-sitter test'
