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

if ($null -ne $env:TF_BUILD) {
    npm ci --omit:optional --registry https://pkgs.dev.azure.com/powershell/PowerShell/_packaging/powershell/npm/registry/
}
else {
    npm install --omit:optional --registry https://pkgs.dev.azure.com/powershell/PowerShell/_packaging/powershell/npm/registry/
}

#npm list tree-sitter-cli
#if ($LASTEXITCODE -ne 0) {
#    npm ci tree-sitter-cli --omit=optional
#}

#npm install -g node-gyp

if ($UpdatePackages) {
    if (!$IsWindows) {
        throw "This switch only works on Windows"
    }

    rm ./package-lock.json
    rm -r ./node_modules
    npm cache clean --force
    npm logout
    vsts-npm-auth -config .npmrc -F -V
    npm install --omit:optional --force --verbose --registry https://pkgs.dev.azure.com/powershell/PowerShell/_packaging/powershell/npm/registry/
}

Invoke-NativeCommand 'npx node-gyp configure'
Invoke-NativeCommand 'npx tree-sitter generate --build'
Invoke-NativeCommand 'npx tree-sitter test'
