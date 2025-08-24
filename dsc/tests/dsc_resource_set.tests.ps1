# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Invoke a resource set directly' {
    It 'set returns proper error code if no input is provided' {
        $out = dsc resource set -r Test/Version 2>&1
        $LASTEXITCODE | Should -Be 1
        $out | Should -BeLike '*ERROR*'
    }

     It 'version works' {
        $out = dsc resource set -r Test/Version --version 1.1.2 --input '{"version":"1.1.2"}' | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.afterState.version | Should -BeExactly '1.1.2'
        $out.changedProperties | Should -BeNullOrEmpty
    }
}