Describe 'config test tests' {
    It 'should confirm matching state' -Skip:(!$IsWindows) {
        $json = @'
        {
            "keyPath": "HKLM\\Software\\Microsoft\\Windows NT\\CurrentVersion",
            "valueName": "ProductName"
        }
'@
        $current = $json | registry config get
        $out = $current | config test -r registry
        $LASTEXITCODE | Should -Be 0
        $out = $out | ConvertFrom-Json
        $out.actual_state._inDesiredState | Should -BeTrue
        $out.diff_properties | Should -BeNullOrEmpty
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
        $out = $json | config test -r registry
        $LASTEXITCODE | Should -Be 0
        $out = $out | ConvertFrom-Json
        $out.actual_state._inDesiredState | Should -BeFalse
        $out.diff_properties.Count | Should -Be 1
        $out.diff_properties[0] | Should -BeExactly 'valueData'
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
        $out = $json | config test -r registry
        $LASTEXITCODE | Should -Be 0
        $out = $out | ConvertFrom-Json
        $out.actual_state._inDesiredState | Should -BeFalse
        $out.diff_properties.Count | Should -Be 2
        $out.diff_properties[0] | Should -BeExactly 'valueName'
        $out.diff_properties[1] | Should -BeExactly 'valueData'
    }
}