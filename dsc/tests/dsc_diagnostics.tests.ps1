# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Schema validation diagnostics' {
    BeforeAll {
        $schemaUri = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
        $configYaml = @"
`$schema: $schemaUri
resources:
- name: first
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: fine
- name: second
  type: Microsoft.DSC.Debug/Echo
  properties:
    otput: hi
"@
        $configPath = Join-Path $TestDrive 'typo.dsc.yaml'
        Set-Content -Path $configPath -Value $configYaml
    }

    It 'shows the invalid property of resource input in a snippet with a suggestion' {
        $resourceInput = @'
{
  "otput": "hi"
}
'@
        $null = dsc resource get --resource Microsoft.DSC.Debug/Echo --input $resourceInput 2> $TestDrive/error.log
        $LASTEXITCODE | Should -Be 2
        $log = Get-Content -Path $TestDrive/error.log -Raw
        $log | Should -Match "Schema: input for 'Microsoft.DSC.Debug/Echo'" -Because $log
        $log | Should -Match "'otput' was unexpected" -Because $log
        # the snippet opens at the first label, which marks the object missing its required property
        $log | Should -Match '\[input:1:1\]' -Because $log
        $log | Should -Match "'otput' isn't a known property" -Because $log
        $log | Should -Match "missing required property 'output'" -Because $log
        $log | Should -Match "Did you mean 'output' instead of 'otput'" -Because $log
    }

    It 'points at the resource declaration in the configuration file for: <operation>' -TestCases @(
        @{ operation = 'get'; exitCode = 2 }
        @{ operation = 'test'; exitCode = 5 }
    ) {
        param($operation, $exitCode)

        $null = dsc config $operation --file $configPath 2> $TestDrive/error.log
        $LASTEXITCODE | Should -Be $exitCode
        $log = Get-Content -Path $TestDrive/error.log -Raw
        $log | Should -Match "Schema: input for 'Microsoft.DSC.Debug/Echo'" -Because $log
        # the snippet is the author's YAML, not the evaluated properties
        $log | Should -Match ([regex]::Escape($configPath) + ':9:3') -Because $log
        $log | Should -Match '- name: second' -Because $log
        $log | Should -Match 'otput: hi' -Because $log
        $log | Should -Match "'otput' isn't a known property" -Because $log
        $log | Should -Match "Did you mean 'output' instead of 'otput'" -Because $log
        $log | Should -Not -Match 'name: first' -Because 'only the failing resource is shown'
    }

    It 'names inline input as input in the snippet' {
        $null = dsc config get --input $configYaml 2> $TestDrive/error.log
        $LASTEXITCODE | Should -Be 2
        $log = Get-Content -Path $TestDrive/error.log -Raw
        $log | Should -Match '\[input:9:3\]' -Because $log
    }

    It 'keeps the message on one line for the json trace format' {
        $null = dsc --trace-format json config get --file $configPath 2> $TestDrive/error.log
        $LASTEXITCODE | Should -Be 2
        $errors = @(Get-Content -Path $TestDrive/error.log | Where-Object { $_ -like '*"level":"ERROR"*' })
        $errors.Count | Should -Be 1 -Because (Get-Content -Path $TestDrive/error.log -Raw)
        $entry = $errors[0] | ConvertFrom-Json
        $entry.fields.message | Should -Match "^Schema: input for 'Microsoft.DSC.Debug/Echo': Additional properties are not allowed \('otput' was unexpected\)" -Because $entry.fields.message
        $entry.fields.message | Should -Not -Match 'otput: hi' -Because 'the snippet is only rendered for the default and plaintext formats'
    }

    It 'renders the snippet without colors for the plaintext trace format' {
        $null = dsc --trace-format plaintext config get --file $configPath 2> $TestDrive/error.log
        $LASTEXITCODE | Should -Be 2
        $log = Get-Content -Path $TestDrive/error.log -Raw
        $log | Should -Match "'otput' isn't a known property" -Because $log
        $log | Should -Not -Match "`e\[" -Because 'plaintext output has no ANSI escape sequences'
    }
}
