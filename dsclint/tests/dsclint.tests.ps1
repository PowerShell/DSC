# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'dsclint tests' {
    BeforeAll {
        $schemaUri = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
        $cleanConfig = @"
`$schema: $schemaUri
parameters:
  greeting:
    type: string
    defaultValue: hello
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[parameters('greeting')]"
"@
        # The document from https://github.com/PowerShell/DSC/issues/842
        $duplicateConfig = @"
`$schema: $schemaUri
resources:
- name: EnsureTest1Value
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: one
- name: EnsureTest2Value
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: two
- name: EnsureTest1Value
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: three
"@
        $unusedParameterConfig = @"
`$schema: $schemaUri
parameters:
  unused:
    type: string
    defaultValue: x
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: hello
"@
    }

    It 'reports no diagnostics for a clean document' {
        $result = $cleanConfig | dsclint --file - 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
        $result.valid | Should -BeTrue
        $result.discovery | Should -BeTrue
        $result.diagnostics.Count | Should -Be 0
        $result.summary.errors | Should -Be 0
        $result.summary.warnings | Should -Be 0
    }

    It 'reports a duplicate resource and exits with code 5' {
        $result = $duplicateConfig | dsclint --file - --no-discovery 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 5 -Because (Get-Content $TestDrive/error.log -Raw)
        $result.valid | Should -BeFalse
        $result.discovery | Should -BeFalse
        $result.summary.errors | Should -Be 1
        $result.diagnostics.Count | Should -Be 1
        $result.diagnostics[0].rule | Should -BeExactly 'duplicate-resource'
        $result.diagnostics[0].level | Should -BeExactly 'error'
        $result.diagnostics[0].category | Should -BeExactly 'document'
        $result.diagnostics[0].path | Should -BeExactly '/resources/2'
        $result.diagnostics[0].message | Should -BeLike "*EnsureTest1Value*"
        # the third resource starts on line 11 of the document, after the leading `- `
        $result.diagnostics[0].location.line | Should -Be 11
        $result.diagnostics[0].location.column | Should -Be 3
    }

    It 'records the line and column of a diagnostic inside a value' {
        $result = $unusedParameterConfig | dsclint --file - --no-discovery 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
        $result.diagnostics[0].path | Should -BeExactly '/parameters/unused'
        $result.diagnostics[0].location.line | Should -Be 3
        $result.diagnostics[0].location.column | Should -Be 3
    }

    It 'accepts the document with --input' {
        $result = dsclint --input $duplicateConfig --no-discovery 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 5 -Because (Get-Content $TestDrive/error.log -Raw)
        $result.diagnostics[0].rule | Should -BeExactly 'duplicate-resource'
    }

    It 'treats warnings as non-fatal unless --fail-on warning is set' {
        $result = $unusedParameterConfig | dsclint --file - --no-discovery 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
        $result.valid | Should -BeTrue
        $result.summary.warnings | Should -Be 1
        $result.diagnostics[0].rule | Should -BeExactly 'no-unused-parameters'
        $result.diagnostics[0].level | Should -BeExactly 'warning'
        $result.diagnostics[0].path | Should -BeExactly '/parameters/unused'

        $null = $unusedParameterConfig | dsclint --file - --no-discovery --fail-on warning 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 5 -Because (Get-Content $TestDrive/error.log -Raw)
    }

    It 'can change the level of a rule with --rule' {
        $result = $unusedParameterConfig | dsclint --file - --no-discovery --rule no-unused-parameters=off 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
        $result.diagnostics.Count | Should -Be 0

        $result = $unusedParameterConfig | dsclint --file - --no-discovery --rule no-unused-parameters=error 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 5 -Because (Get-Content $TestDrive/error.log -Raw)
        $result.valid | Should -BeFalse
        $result.diagnostics[0].level | Should -BeExactly 'error'
    }

    It 'rejects an unknown rule with exit code 1' {
        $null = $cleanConfig | dsclint --file - --no-discovery --rule no-such-rule=off 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 1
        (Get-Content $TestDrive/error.log -Raw) | Should -BeLike '*no-such-rule*'
    }

    It 'reports a document that is not valid YAML as a diagnostic' {
        $result = "resources:`n- name: a`n  type: [unclosed" | dsclint --file - --no-discovery 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 5 -Because (Get-Content $TestDrive/error.log -Raw)
        $result.valid | Should -BeFalse
        $result.diagnostics.Count | Should -Be 1
        $result.diagnostics[0].rule | Should -BeExactly 'parse-error'
        $result.diagnostics[0].path | Should -BeExactly ''
        $result.diagnostics[0].location.line | Should -BeGreaterThan 0
    }

    It 'reports schema violations with the path of the problem' {
        $config = @"
`$schema: $schemaUri
resources:
- name: a
  type: Microsoft.DSC.Debug/Echo
  unknownKey: true
"@
        $result = $config | dsclint --file - --no-discovery 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 5 -Because (Get-Content $TestDrive/error.log -Raw)
        $violations = @($result.diagnostics | Where-Object rule -eq 'schema-invalid')
        $violations.Count | Should -BeGreaterThan 0
        $violations.path | Should -Contain '/resources/0'
    }

    It 'reports <rule> at <path>' -TestCases @(
        @{ rule = 'undefined-parameter'; path = '/resources/0/properties/output'; property = "[parameters('missing')]" }
        @{ rule = 'undefined-variable'; path = '/resources/0/properties/output'; property = "[variables('missing')]" }
        @{ rule = 'unknown-function'; path = '/resources/0/properties/output'; property = "[noSuchFunction()]" }
        @{ rule = 'invalid-argument-count'; path = '/resources/0/properties/output'; property = "[int('1', '2')]" }
        @{ rule = 'invalid-argument-type'; path = '/resources/0/properties/output'; property = "[int(true)]" }
        @{ rule = 'invalid-expression'; path = '/resources/0/properties/output'; property = "[concat('unterminated)]" }
        @{ rule = 'unresolved-dependency'; path = '/resources/0/properties/output'; property = "[reference(resourceId('Microsoft.DSC.Debug/Echo', 'nobody'))]" }
    ) {
        param($rule, $path, $property)
        $config = @"
`$schema: $schemaUri
resources:
- name: a
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "$property"
"@
        $result = $config | dsclint --file - --no-discovery 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 5 -Because (Get-Content $TestDrive/error.log -Raw)
        $diagnostic = $result.diagnostics | Where-Object rule -eq $rule
        $diagnostic | Should -Not -BeNullOrEmpty -Because ($result.diagnostics | ConvertTo-Json -Depth 5)
        $diagnostic.path | Should -BeExactly $path
        $diagnostic.level | Should -BeExactly 'error'
    }

    It 'reports circular dependencies' {
        $config = @"
`$schema: $schemaUri
resources:
- name: a
  type: Microsoft.DSC.Debug/Echo
  dependsOn:
  - "[resourceId('Microsoft.DSC.Debug/Echo', 'b')]"
- name: b
  type: Microsoft.DSC.Debug/Echo
  dependsOn:
  - "[resourceId('Microsoft.DSC.Debug/Echo', 'a')]"
"@
        $result = $config | dsclint --file - --no-discovery 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 5 -Because (Get-Content $TestDrive/error.log -Raw)
        $diagnostic = $result.diagnostics | Where-Object rule -eq 'circular-dependency'
        $diagnostic.path | Should -BeExactly '/resources/0/dependsOn'
        $diagnostic.message | Should -BeLike '*a -> b -> a*'
    }

    It 'warns when --system-root is given but systemRoot() is not used' {
        $result = $cleanConfig | dsclint --file - --no-discovery --system-root $TestDrive 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
        $diagnostic = $result.diagnostics | Where-Object rule -eq 'system-root-unused'
        $diagnostic | Should -Not -BeNullOrEmpty
        $diagnostic.level | Should -BeExactly 'warning'
        $diagnostic.path | Should -BeExactly ''

        $config = @"
`$schema: $schemaUri
resources:
- name: a
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[systemRoot()]"
"@
        $result = $config | dsclint --file - --no-discovery --system-root $TestDrive 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
        $result.diagnostics | Where-Object rule -eq 'system-root-unused' | Should -BeNullOrEmpty
    }

    It 'checks supplied parameter values' {
        $config = @"
`$schema: $schemaUri
parameters:
  count:
    type: int
  required:
    type: string
resources:
- name: a
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[concat(string(parameters('count')), parameters('required'))]"
"@
        $result = $config | dsclint --file - --no-discovery --parameters '{"parameters": {"count": "five", "extra": 1}}' 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 5 -Because (Get-Content $TestDrive/error.log -Raw)
        $result.diagnostics.rule | Should -Contain 'parameter-value-type'
        $result.diagnostics.rule | Should -Contain 'missing-parameter-value'
        $result.diagnostics.rule | Should -Contain 'undeclared-parameter-value'
    }

    It 'reports unknown resource types only when discovery runs' {
        $config = @"
`$schema: $schemaUri
resources:
- name: a
  type: Not.Installed/Anywhere
"@
        $result = $config | dsclint --file - 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
        $result.discovery | Should -BeTrue
        $diagnostic = $result.diagnostics | Where-Object rule -eq 'unknown-resource-type'
        $diagnostic.level | Should -BeExactly 'warning'
        $diagnostic.path | Should -BeExactly '/resources/0/type'

        $result = $config | dsclint --file - --no-discovery 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
        $result.discovery | Should -BeFalse
        $result.diagnostics | Where-Object rule -eq 'unknown-resource-type' | Should -BeNullOrEmpty
    }

    It 'lints several files in one run and records the file on each diagnostic' {
        Set-Content -Path "$TestDrive/clean.dsc.yaml" -Value $cleanConfig
        Set-Content -Path "$TestDrive/duplicate.dsc.yaml" -Value $duplicateConfig
        $result = dsclint --file "$TestDrive/clean.dsc.yaml" --file "$TestDrive/duplicate.dsc.yaml" --no-discovery 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 5 -Because (Get-Content $TestDrive/error.log -Raw)
        $result.diagnostics.Count | Should -Be 1
        $result.diagnostics[0].file | Should -BeLike '*duplicate.dsc.yaml'
    }

    It 'rejects a resource manifest' {
        $manifest = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json
type: Test/Example
version: 1.0.0
"@
        $result = $manifest | dsclint --file - --no-discovery 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 5 -Because (Get-Content $TestDrive/error.log -Raw)
        $result.diagnostics.Count | Should -Be 1
        $result.diagnostics[0].rule | Should -BeExactly 'unsupported-document-type'
    }

    It 'lists the rules with their levels' {
        $rules = dsclint --list-rules --output-format json 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
        $rules.id | Should -Contain 'duplicate-resource'
        $rules.id | Should -Contain 'parse-error'
        ($rules | Where-Object id -eq 'parse-error').fixed | Should -BeTrue
        $duplicate = $rules | Where-Object id -eq 'duplicate-resource'
        $duplicate.defaultLevel | Should -BeExactly 'error'
        $duplicate.level | Should -BeExactly 'error'
        $duplicate.tier | Should -BeExactly 'static'
        ($rules | Where-Object id -eq 'unknown-resource-type').tier | Should -BeExactly 'resource'

        $rules = dsclint --list-rules --rule duplicate-resource=warning --output-format json 2>$TestDrive/error.log | ConvertFrom-Json
        ($rules | Where-Object id -eq 'duplicate-resource').level | Should -BeExactly 'warning'
    }

    It 'lints the shipped example without errors: <name>' -TestCases (
        Get-ChildItem (Join-Path $PSScriptRoot '../../dsc/examples') -Filter '*.dsc.*' |
            Where-Object { $_.Extension -in @('.yaml', '.json') -and $_.Name -ne 'invalid_schema.dsc.yaml' } |
            ForEach-Object { @{ name = $_.Name; path = $_.FullName } }
    ) {
        param($name, $path)
        $result = dsclint --file $path --no-discovery 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because ($result.diagnostics | ConvertTo-Json -Depth 5)
        $result.summary.errors | Should -Be 0
    }
}
