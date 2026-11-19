# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Registry list set tests' -Skip:(!$IsWindows) {
    BeforeEach {
        Remove-Item -Path 'HKCU:\1' -Recurse -ErrorAction Ignore
    }

    AfterEach {
        Remove-Item -Path 'HKCU:\1' -Recurse -ErrorAction Ignore
    }

    It 'Can set a registry list' -Skip:(!$IsWindows) {
        $config_yaml = @'
            $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Reg 1
              type: Microsoft.Windows/RegistryList
              properties:
                registryEntries:
                - keyPath: HKCU\1\2\3
                  valueName: Hello
                  valueData:
                    String: World
                - keyPath: HKCU\1\2\4
                  valueName: Hello2
                  valueData:
                    String: World2
                - keyPath: HKCU\1\2\5
                  _exist: false
                - keyPath: HKCU\1\2\6
                  _exist: false
                  valueName: Hello4
                  valueData:
                    String: World4
'@
        $out = dsc config set --input $config_yaml 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results.result[0].afterState.registryEntries.Count | Should -Be 4 -Because ($out | ConvertTo-Json -Depth 10)
        $out.results.result[0].afterState.registryEntries[0].keyPath | Should -BeExactly 'HKCU\1\2\3'
        $out.results.result[0].afterState.registryEntries[0].valueName | Should -BeExactly 'Hello'
        $out.results.result[0].afterState.registryEntries[0].valueData.String | Should -BeExactly 'World'
        (Get-ItemProperty -Path 'HKCU:\1\2\3' -Name 'Hello').Hello | Should -BeExactly 'World'
        $out.results.result[0].afterState.registryEntries[1].keyPath | Should -BeExactly 'HKCU\1\2\4'
        $out.results.result[0].afterState.registryEntries[1].valueName | Should -BeExactly 'Hello2'
        $out.results.result[0].afterState.registryEntries[1].valueData.String | Should -BeExactly 'World2'
        (Get-ItemProperty -Path 'HKCU:\1\2\4' -Name 'Hello2').Hello2 | Should -BeExactly 'World2'
        $out.results.result[0].afterState.registryEntries[2].keyPath | Should -BeExactly 'HKCU\1\2\5'
        $out.results.result[0].afterState.registryEntries[2]._exist | Should -BeFalse
        Get-Item -Path 'HKCU:\1\2\5' -ErrorAction Ignore | Should -BeNullOrEmpty
        $out.results.result[0].afterState.registryEntries[3].keyPath | Should -BeExactly 'HKCU\1\2\6'
        $out.results.result[0].afterState.registryEntries[3]._exist | Should -BeFalse
        $out.results.result[0].afterState.registryEntries[3].valueName | Should -BeNullOrEmpty
        $out.results.result[0].afterState.registryEntries[3].valueData | Should -BeNullOrEmpty
        (Get-ItemProperty -Path 'HKCU:\1\2\6' -Name 'Hello4' -ErrorAction Ignore).Hello4 | Should -BeNullOrEmpty
    }

    It 'Can set access control lists in a registry list' -Skip:(!$IsWindows) {
        New-Item -Path 'HKCU:\1\AclListSet' -Force | Out-Null
        $json = @{
            registryEntries = @(
                @{
                    keyPath = 'HKCU\1\AclListSet'
                    accessControlList = @{
                        discretionary          = @(
                            @{
                                principal  = $env:USERNAME
                                type       = 'allow'
                                accessMask = @('fullControl')
                            }
                        )
                        discretionaryProtected = $true
                    }
                }
            )
        } | ConvertTo-Json -Depth 10 -Compress

        registry config set --list --input $json 2>$TestDrive/error.log

        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)

        $getJson = @{
            registryEntries = @(
                @{
                    keyPath = 'HKCU\1\AclListSet'
                    accessControlList = @{ discretionary = @() }
                }
            )
        } | ConvertTo-Json -Depth 10 -Compress
        $result = registry config get --list --input $getJson 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)
        $result.registryEntries[0].accessControlList.discretionaryProtected | Should -BeTrue
        $result.registryEntries[0].accessControlList.discretionary.Count | Should -Be 1
        $result.registryEntries[0].accessControlList.discretionary[0].accessMask | Should -Contain 'fullControl'
    }
}
