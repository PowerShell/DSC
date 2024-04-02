# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'config get tests' {
    It 'should get from registry using <type> resource' -Skip:(!$IsWindows) -TestCases @(
        @{ type = 'string' }
    ) {
        param($type)

        switch ($type) {
            'string' {
                $resource = 'Microsoft.Windows/registry'
            }
            'json' {
                $resource = dsc resource list *registry
                $LASTEXITCODE | Should -Be 0
                $resource.Count | Should -Be 1
                ($resource | ConvertFrom-Json).Type | Should -BeExactly 'Microsoft.Windows/Registry'
                if ($PSNativeCommandArgumentPassing -ne 'Windows') {
                    # legacy mode requires double quotes to be escaped
                    $resource = $resource.Replace('"', '""')
                }
            }
        }

        $json = @'
        {
            "keyPath": "HKLM\\Software\\Microsoft\\Windows NT\\CurrentVersion",
            "valueName": "ProductName"
        }
'@
        $output = $json | dsc resource get -r $resource
        $LASTEXITCODE | Should -Be 0
        $output = $output | ConvertFrom-Json
        $output.actualState.'$id' | Should -BeExactly 'https://developer.microsoft.com/json-schemas/windows/registry/20230303/Microsoft.Windows.Registry.schema.json'
        $output.actualState.keyPath | Should -BeExactly 'HKLM\Software\Microsoft\Windows NT\CurrentVersion'
        $output.actualState.valueName | Should -BeExactly 'ProductName'
        $output.actualState.valueData.String | Should -Match 'Windows .*'
    }

    It 'invalid input is validated against schema' -Skip:(!$IsWindows) {
        $json = @'
        {
            "keyPath": "HKLM\\Software\\Microsoft\\Windows NT\\CurrentVersion",
            "Name": "ProductName"
        }
'@
        $testError = & {$json | dsc resource get -r Microsoft.Windows/registry get 2>&1}
        $testError[0] | SHould -match 'error:'
        $LASTEXITCODE | Should -Be 2
    }
}
