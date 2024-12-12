# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'resource get tests' {
    It 'should get from registry using <type> resource' -Skip:(!$IsWindows) -TestCases @(
        @{ type = 'string' }
    ) {
        param($type)

        switch ($type) {
            'string' {
                $resource = 'Microsoft.Windows/Registry'
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
        $output = $json | dsc resource get -r $resource -f -
        $LASTEXITCODE | Should -Be 0
        $output = $output | ConvertFrom-Json
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
        $testError = & {$json | dsc resource get -r Microsoft.Windows/Registry get -f - 2>&1}
        $testError[0] | SHould -match 'error:'
        $LASTEXITCODE | Should -Be 2
    }
}
