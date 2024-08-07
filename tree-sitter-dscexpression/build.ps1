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

if ($UpdatePackages) {
    if (!$IsWindows) {
        throw "This switch only works on Windows"
    }

    rm ./package-lock.json
    rm -r ./node_modules
    npm cache clean --force
    npm logout
    vsts-npm-auth -config .npmrc -F -V
    npm install --force --verbose
}

Invoke-NativeCommand 'npx tree-sitter generate --build'
Invoke-NativeCommand 'npx tree-sitter test'
