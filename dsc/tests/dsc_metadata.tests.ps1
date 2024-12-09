Describe 'metadata tests' {
    BeforeAll {
        $config_yaml = @"
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
            resources:
            - name: Message
              type: Test/Metadata
              properties:
                _metadata:
                  messages:
                  - hello world
"@
    }

    It 'can pull _metadata from config set' {
        $result = $config_yaml | dsc config set | ConvertFrom-Json
        $result.results.metadata.messages[0] | Should -BeExactly 'hello world'
        $result.results.result.afterState | Should -Be ''
        $result.hadErrors | Should -BeFalse
        $result.results.Count | Should -Be 1
        $LASTEXITCODE | Should -Be 0
    }

    # It 'can pull _metadata from config get' {
    #     $result = $config_yaml | dsc config get | ConvertFrom-Json
    #     $result.results.metadata.messages[0] | Should -BeExactly 'hello world'
    #     $result.results.result.actualState | Should -Be ''
    #     $result.hadErrors | Should -BeFalse
    #     $result.results.Count | Should -Be 1
    #     $LASTEXITCODE | Should -Be 0
    # }

    #     It 'can pull _metadata from config test' {
    #     $result = $config_yaml | dsc config test | ConvertFrom-Json
    #     $result.results.metadata.messages[0] | Should -BeExactly 'hello world'
    #     $result.results.result.actualState | Should -Be ''
    #     $result.hadErrors | Should -BeFalse
    #     $result.results.Count | Should -Be 1
    #     $LASTEXITCODE | Should -Be 0
    # }
}
