# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'dsc lint plug-in command tests' {
    BeforeAll {
        $script:document = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  unused:
    type: string
    defaultValue: x
resources:
- name: a
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[parameters('missing')]"
'@
        $script:documentPath = Join-Path $TestDrive 'demo.dsc.yaml'
        Set-Content -Path $documentPath -Value $document
    }

    It 'Lists dsclint as the plug-in that provides the lint command' {
        $out = dsc plugin list 'Microsoft.DSC/Lint' 2> $TestDrive/error.txt | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
        $out.command | Should -BeExactly 'lint'
        $out.shadowed | Should -BeFalse
        $out.manifest.command.executable | Should -BeExactly 'dsclint'
        $out.version | Should -BeExactly ((dsc --version) -replace '^dsc\s+', '')
    }

    It 'Lists the lint command in the top-level help' {
        $out = dsc --help 2> $TestDrive/error.txt | Out-String
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
        $out | Should -Match '(?m)^Plug-in commands:\r?$'
        $out | Should -Match '(?m)^  lint\s+Check configuration documents for problems without applying them\r?$'
    }

    It 'Returns the same result and exit code as dsclint' {
        $expected = dsclint --file $documentPath --no-discovery --output-format json 2> $TestDrive/expected-error.txt
        $expectedExitCode = $LASTEXITCODE
        $actual = dsc lint --file $documentPath --no-discovery --output-format json 2> $TestDrive/error.txt
        $LASTEXITCODE | Should -Be $expectedExitCode -Because (Get-Content -Raw $TestDrive/error.txt)
        $expectedExitCode | Should -Be 5
        $actual | Should -BeExactly $expected

        $result = $actual | ConvertFrom-Json
        $result.valid | Should -BeFalse
        $result.diagnostics.rule | Should -Contain 'undefined-parameter'
        $result.diagnostics.rule | Should -Contain 'no-unused-parameters'
    }

    It 'Passes standard input to the linter' {
        $out = $document | dsc lint --file - --no-discovery --output-format json 2> $TestDrive/error.txt | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 5 -Because (Get-Content -Raw $TestDrive/error.txt)
        $out.diagnostics.rule | Should -Contain 'undefined-parameter'
    }

    It 'Runs the linter by its type name' {
        $out = dsc Microsoft.DSC/Lint --list-rules --output-format json 2> $TestDrive/error.txt | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
        $out.id | Should -Contain 'no-unused-parameters'
    }

    It 'Shows the help of the linter for: dsc <arguments>' -TestCases @(
        @{ arguments = @('help', 'lint') }
        @{ arguments = @('lint', '--help') }
    ) {
        param($arguments)

        $out = dsc @arguments 2> $TestDrive/error.txt | Out-String
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
        $out | Should -Match '--fail-on'
        $out | Should -Match '--no-discovery'
    }

    It 'Applies the trace level given before the command to the linter' {
        $null = dsc --trace-level info lint --file $documentPath --no-discovery --output-format json 2> $TestDrive/error.txt
        $LASTEXITCODE | Should -Be 5
        Get-Content -Raw $TestDrive/error.txt | Should -Match 'Trace-level is Info'

        $null = dsc lint --file $documentPath --no-discovery --output-format json 2> $TestDrive/error.txt
        $LASTEXITCODE | Should -Be 5
        Get-Content -Raw $TestDrive/error.txt | Should -Not -Match 'Trace-level is'
    }

    It 'Lets an option after the command override the trace level given before it' {
        $null = dsc --trace-level error lint --trace-level info --file $documentPath --no-discovery --output-format json 2> $TestDrive/error.txt
        $LASTEXITCODE | Should -Be 5
        Get-Content -Raw $TestDrive/error.txt | Should -Match 'Trace-level is Info'
    }
}
