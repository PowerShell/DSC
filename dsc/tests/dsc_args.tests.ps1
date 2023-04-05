# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'config argument tests' {
    It 'input is <type>' -Skip:(!$IsWindows) -TestCases @(
        @{ type = 'yaml'; text = @'
            keyPath: HKLM\Software\Microsoft\Windows NT\CurrentVersion
            valueName: ProductName
'@ }
        @{ type = 'json'; text = @'
            {
                "keyPath": "HKLM\\Software\\Microsoft\\Windows NT\\CurrentVersion",
                "valueName": "ProductName"
            }
'@ }
    ) {
        param($text)
        $output = $text | dsc resource get -r *registry
        $output = $output | ConvertFrom-Json
        $output.actual_state.'$id' | Should -BeExactly 'https://developer.microsoft.com/json-schemas/windows/registry/20230303/Microsoft.Windows.Registry.schema.json'
        $output.actual_state.keyPath | Should -BeExactly 'HKLM\Software\Microsoft\Windows NT\CurrentVersion'
        $output.actual_state.valueName | Should -BeExactly 'ProductName'
        $output.actual_state.valueData.String | Should -Match 'Windows .*'
    }
}
