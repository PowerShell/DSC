Describe 'config get tests' {
    It 'should get from registry using <type> resource' -Skip:(!$IsWindows) -TestCases @(
        @{ type = 'string' }
        @{ type = 'json' }
    ) {
        param($type)

        switch ($type) {
            'string' {
                $resource = 'registry'
            }
            'json' {
                $resource = dsc resource list registry
                $LASTEXITCODE | Should -Be 0
                $resource.Count | Should -Be 1
                ($resource | ConvertFrom-Json).Name | Should -BeExactly 'Registry'
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
        $output.actual_state.'$id' | Should -BeExactly 'https://developer.microsoft.com/json-schemas/windows/registry/20230303/Microsoft.Windows.Registry.schema.json'
        $output.actual_state.keyPath | Should -BeExactly 'HKLM\Software\Microsoft\Windows NT\CurrentVersion'
        $output.actual_state.valueName | Should -BeExactly 'ProductName'
        $output.actual_state.valueData.String | Should -Match 'Windows .*'
    }
}
