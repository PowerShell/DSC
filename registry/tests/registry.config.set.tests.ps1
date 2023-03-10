Describe 'registry config set tests' {
    It 'Can set a deeply nested key and value' -Skip:(!$IsWindows) {
        $json = @'
        {
            "keyPath": "HKCU\\1\\2\\3",
            "valueName": "Hello",
            "valueData": {
                "String": "World"
            }
        }
'@
        $out = $json | registry config set
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result.keyPath | Should -Be 'HKCU\1\2\3'
        $result.valueName | Should -Be 'Hello'
        $result.valueData.String | Should -Be 'World'
        ($result.psobject.properties | Measure-Object).Count | Should -Be 4

        $out = $json | registry config get
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result.keyPath | Should -Be 'HKCU\1\2\3'
        $result.valueName | Should -Be 'Hello'
        $result.valueData.String | Should -Be 'World'
        ($result.psobject.properties | Measure-Object).Count | Should -Be 4
    }

    It 'Can set a key to be absent' -Skip:(!$IsWindows) {
        $json = @'
        {
            "keyPath": "HKCU\\1",
            "_ensure": "Absent"
        }
'@
        $out = $json | registry config set
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result.keyPath | Should -Be 'HKCU\1'
        ($result.psobject.properties | Measure-Object).Count | Should -Be 2

        $json | registry config get 2>$null
        $LASTEXITCODE | Should -Be 3
    }
}
