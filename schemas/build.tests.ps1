# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

#requires -Modules yayaml

Describe 'Extension schema source generation' {
    BeforeAll {
        $buildDirectory = (New-Item -ItemType Directory -Path "$TestDrive/schema-build").FullName
        Copy-Item -LiteralPath "$PSScriptRoot/src" -Destination $buildDirectory -Recurse
        Copy-Item -LiteralPath "$PSScriptRoot/build.ps1" -Destination $buildDirectory
        $config = Get-Content -LiteralPath "$PSScriptRoot/schemas.config.yaml" -Raw |
            yayaml\ConvertFrom-Yaml
        $config.version = 'v3'
        $config.bundle_schemas = @($config.bundle_schemas | Where-Object {
            $_.ConfigFilePath -eq 'extension/manifest.json'
        })
        $config | yayaml\ConvertTo-Yaml -Depth 99 |
            Set-Content -LiteralPath "$buildDirectory/schemas.config.yaml"
        & "$buildDirectory/build.ps1" -OutputFormat Json,JsonVSCode -ErrorAction Stop | Out-Null
    }

    It 'preserves <Operation> transport selection in regenerated <SchemaFile>' -ForEach @(
        @{ Operation = 'discover'; SchemaFile = 'manifest.json' }
        @{ Operation = 'secret'; SchemaFile = 'manifest.json' }
        @{ Operation = 'discover'; SchemaFile = 'manifest.vscode.json' }
        @{ Operation = 'secret'; SchemaFile = 'manifest.vscode.json' }
    ) {
        $schema = Get-Content -LiteralPath "$buildDirectory/v3/bundled/extension/$SchemaFile" -Raw
        $manifest = Get-Content -LiteralPath "$PSScriptRoot/../extensions/test/jsonrpc/$Operation/$Operation.dsc.extension.json" -Raw |
            ConvertFrom-Json -AsHashtable
        $manifest | ConvertTo-Json -Depth 30 | Test-Json -Schema $schema -ErrorAction Stop |
            Should -BeTrue
        $manifest[$Operation].executable = 'pwsh'
        $manifest[$Operation].args = @(if ($Operation -eq 'secret') {
            @{ nameArg = '--name' }
        } else {
            '--discover'
        })
        $manifest | ConvertTo-Json -Depth 30 | Test-Json -Schema $schema -ErrorAction SilentlyContinue |
            Should -BeFalse
        $manifest[$Operation].Remove('jsonrpc')
        $manifest | ConvertTo-Json -Depth 30 | Test-Json -Schema $schema -ErrorAction Stop |
            Should -BeTrue
    }
}
