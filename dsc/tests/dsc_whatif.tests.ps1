Describe 'whatif tests' {
    AfterEach {
        if ($IsWindows) {
            Remove-Item -Path 'HKCU:\1' -Recurse -ErrorAction Ignore
        }
    }

    It 'config set whatif when actual state matches desired state' {
        $config_yaml = @"
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
            resources:
            - name: Hello
              type: Test/Echo
              properties:
                output: hello
"@
        $result = $config_yaml | dsc config set -w --format pretty-json | ConvertFrom-Json
        $result.hadErrors | Should -BeFalse
        $result.results.Count | Should -Be 1
        $result.results[0].Name | Should -Be 'Hello'
        $result.results[0].type | Should -BeExactly 'Test/Echo'
        $result.results[0].result.whatIfChanges | Should -Be 'none'
        $result.metadata.'Microsoft.DSC'.executionType | Should -BeExactly 'WhatIf'
        $LASTEXITCODE | Should -Be 0
    }

    It 'config set whatif when actual state does not match desired state' -Skip:(!$IsWindows) {
        # TODO: change/create cross-plat resource that implements set without just matching desired state
        $config_yaml = @"
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
            resources:
            - name: Registry
              type: Microsoft.Windows/Registry
              properties:
                keyPath: 'HKCU\1\2'
"@
        $result = $config_yaml | dsc config set -w --format pretty-json | ConvertFrom-Json
        $result.hadErrors | Should -BeFalse
        $result.results.Count | Should -Be 1
        $result.results[0].Name | Should -Be 'Registry'
        $result.results[0].type | Should -BeExactly 'Microsoft.Windows/Registry'
        $result.results[0].result.whatIfChanges._exist.from | Should -Be 'false'
        $result.results[0].result.whatIfChanges._exist.to | Should -Be 'true'
        $result.metadata.'Microsoft.DSC'.executionType | Should -BeExactly 'WhatIf'
        $LASTEXITCODE | Should -Be 0
    }

    It 'config set whatif for delete is not supported' {
        $config_yaml = @"
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
            resources:
            - name: Delete
              type: Test/Delete
              properties:
                _exist: false
"@
        $result = $config_yaml | dsc config set -w 2>&1
        $result | Should -Match 'ERROR.*?Not implemented.*?what-if'
        $LASTEXITCODE | Should -Be 2
    }

    It 'config set whatif when there is no pre-test is not supported' {
        $result = dsc config set -p c:\dsc\dsc\examples\groups.dsc.yaml -w 2>&1
        $result | Should -Match 'ERROR.*?Not implemented.*?what-if'
        $LASTEXITCODE | Should -Be 2
    }
}
