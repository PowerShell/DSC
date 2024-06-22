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
    Write-Host 'Installing Node'

    if ($IsWindows) {
        winget install OpenJS.NodeJS.LTS
    }
    elseif ($IsMacOS) {
        brew install node
    }
    else {
        sudo apt-get install nodejs
        sudo apt-get install npm
    }
}

npm list tree-sitter-cli
if ($LASTEXITCODE -ne 0) {
    npm ci tree-sitter-cli --omit=optional
}

Invoke-NativeCommand 'npx tree-sitter generate'
Invoke-NativeCommand 'npx tree-sitter test'
