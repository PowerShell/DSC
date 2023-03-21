Describe 'config set tests' {
    BeforeEach {
        $json = @'
        {
            "keyPath": "HKCU\\1\\2\\3",
            "_ensure": "Absent"
        }
'@
        $json | registry config set
    }

    AfterEach {
        $json = @'
        {
            "keyPath": "HKCU\\1",
            "_ensure": "Absent"
        }
'@
        $json | registry config set
    }

    It 'can set and remove a registry value' -Skip:(!$IsWindows) {
        $json = @'
        {
            "keyPath": "HKCU\\1\\2\\3",
            "valueName": "Hello",
            "valueData": {
                "String": "World"
            }
        }
'@
        $out = $json | config set -r registry
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result.after_state.keyPath | Should -Be 'HKCU\1\2\3'
        $result.after_state.valueName | Should -Be 'Hello'
        $result.after_state.valueData.String | Should -Be 'World'
        $result.changed_properties | Should -Be @('keyPath', 'valueName', 'valueData')
        ($result.psobject.properties | Measure-Object).Count | Should -Be 3

        $out = $json | config get -r registry
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result.actual_state.keyPath | Should -Be 'HKCU\1\2\3'
        $result.actual_state.valueName | Should -Be 'Hello'
        $result.actual_state.valueData.String | Should -Be 'World'
        ($result.psobject.properties | Measure-Object).Count | Should -Be 1

        $json = @'
        {
            "keyPath": "HKCU\\1",
            "_ensure": "Absent"
        }
'@
        $out = $json | config set -r registry
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result.after_state.keyPath | Should -BeNullOrEmpty
        $result.changed_properties | Should -Be @('keyPath')
        ($result.psobject.properties | Measure-Object).Count | Should -Be 3
    }
}
