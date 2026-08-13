# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Synthetic test uses schema defaults' {
    It 'Property matching schema default is not reported as differing' {
        $out = '{"name":"test","enabled":true}' | dsc resource test -r Test/SchemaDefault -f - | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.inDesiredState | Should -Be $true
        $out.differingProperties | Should -BeNullOrEmpty
    }

    It 'Property differing from schema default is reported as differing' {
        $out = '{"name":"test","enabled":false}' | dsc resource test -r Test/SchemaDefault -f - | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.inDesiredState | Should -Be $false
        $out.differingProperties | Should -Contain 'enabled'
    }

    It 'Integer property matching schema default is not reported as differing' {
        $out = '{"name":"test","count":5}' | dsc resource test -r Test/SchemaDefault -f - | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.inDesiredState | Should -Be $true
        $out.differingProperties | Should -BeNullOrEmpty
    }

    It 'Integer property differing from schema default is reported as differing' {
        $out = '{"name":"test","count":10}' | dsc resource test -r Test/SchemaDefault -f - | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.inDesiredState | Should -Be $false
        $out.differingProperties | Should -Contain 'count'
    }

    It 'Multiple properties matching schema defaults are not reported as differing' {
        $out = '{"name":"test","enabled":true,"count":5}' | dsc resource test -r Test/SchemaDefault -f - | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.inDesiredState | Should -Be $true
        $out.differingProperties | Should -BeNullOrEmpty
    }

    It 'Mix of matching and non-matching defaults reports only non-matching' {
        $out = '{"name":"test","enabled":true,"count":10}' | dsc resource test -r Test/SchemaDefault -f - | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.inDesiredState | Should -Be $false
        $out.differingProperties | Should -Contain 'count'
        $out.differingProperties | Should -Not -Contain 'enabled'
    }

    It 'Property present in both expected and actual is compared normally' {
        $out = '{"name":"test"}' | dsc resource test -r Test/SchemaDefault -f - | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.inDesiredState | Should -Be $true
        $out.differingProperties | Should -BeNullOrEmpty
    }
}
