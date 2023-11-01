# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'config test tests' {
    It 'should confirm matching state' -Skip:(!$IsWindows) {
        $json = @'
        {
            "keyPath": "HKLM\\Software\\Microsoft\\Windows NT\\CurrentVersion",
            "valueName": "ProductName"
        }
'@
        $current = $json | registry config get
        $out = $current | dsc resource test -r Microsoft.Windows/registry
        $LASTEXITCODE | Should -Be 0
        $out = $out | ConvertFrom-Json
        $out.inDesiredState | Should -BeTrue
        $out.differingProperties | Should -BeNullOrEmpty
    }

    It 'should confirm non-matching state' -Skip:(!$IsWindows) {
        $json = @'
        {
            "keyPath": "HKLM\\Software\\Microsoft\\Windows NT\\CurrentVersion",
            "valueName": "CurrentMajorVersionNumber",
            "valueData": {
                "DWord": 7
            }
        }
'@
        $out = $json | dsc resource test -r Microsoft.Windows/registry
        $LASTEXITCODE | Should -Be 0
        $out = $out | ConvertFrom-Json
        $out.inDesiredState | Should -BeFalse
        $out.differingProperties.Count | Should -Be 1
        $out.differingProperties[0] | Should -BeExactly 'valueData'
    }

    It 'should confirm non-matching multiple state' -Skip:(!$IsWindows) {
        $json = @'
        {
            "keyPath": "HKLM\\Software\\Microsoft\\Windows NT\\CurrentVersion",
            "valueName": "DoesNotExist",
            "valueData": {
                "DWord": 7
            }
        }
'@
        $out = $json | dsc resource test -r Microsoft.Windows/registry
        $LASTEXITCODE | Should -Be 0
        $out = $out | ConvertFrom-Json
        $out.inDesiredState | Should -BeFalse
        $out.differingProperties.Count | Should -Be 3
        $out.differingProperties[0] | Should -BeExactly 'valueName'
        $out.differingProperties[1] | Should -BeExactly 'valueData'
        $out.differingProperties[2] | Should -BeExactly '_exist'
    }
}