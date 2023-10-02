# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'dsc new tests' {
    It 'dsc new -t configuration' {
        $out = dsc new -t configuration | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.'$schema' | Should -BeExactly 'https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/config/document.json'
    }

    It 'dsc new -t resource-manifest' {
        $out = dsc new -t resource-manifest | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.'$schema' | Should -BeExactly 'https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/bundled/resource/manifest.json'
    }
}
