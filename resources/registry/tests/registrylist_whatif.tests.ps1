# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Registry list set what-if tests' -Skip:(!$IsWindows) {
    BeforeEach {
        Remove-Item -Path 'HKCU:\1' -Recurse -ErrorAction Ignore
    }

    AfterEach {
        Remove-Item -Path 'HKCU:\1' -Recurse -ErrorAction Ignore
    }

    It 'Can set a registry list' -Skip:(!$IsWindows) {
        $before_config_yaml = @'
            $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Reg 1
              type: Microsoft.Windows/RegistryList
              properties:
                registryEntries:
                - keyPath: HKCU\1\2\3
                  valueName: Hello
                  valueData:
                    String: World_before
                - keyPath: HKCU\1\2\4
                  valueName: Hello2
                  valueData:
                    String: World2_before
                - keyPath: HKCU\1\2\5
                - keyPath: HKCU\1\2\6
                  valueName: Hello4
                  valueData:
                    String: World4_before
'@
        dsc config set --input $before_config_yaml 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)

        $after_config_yaml = @'
            $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Reg 1
              type: Microsoft.Windows/RegistryList
              properties:
                registryEntries:
                - keyPath: HKCU\1\2\3
                  valueName: Hello
                  valueData:
                    String: World_after
                - keyPath: HKCU\1\2\4
                  valueName: Hello2
                  valueData:
                    String: World2_after
                - keyPath: HKCU\1\2\5
                  _exist: false
                - keyPath: HKCU\1\2\6
                  _exist: false
                  valueName: Hello4
                  valueData:
                    String: World4_after
'@

        $out = dsc config set --what-if --input $after_config_yaml 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)
        $out.results.result[0].afterState.registryEntries.Count | Should -Be 4 -Because ($out | ConvertTo-Json -Depth 10)
        $out.results.result[0].afterState.registryEntries[0].keyPath | Should -BeExactly 'HKCU\1\2\3'
        $out.results.result[0].afterState.registryEntries[0].valueName | Should -BeExactly 'Hello'
        $out.results.result[0].afterState.registryEntries[0].valueData.String | Should -BeExactly 'World_after'
        $out.results.result[0].afterState.registryEntries[1].keyPath | Should -BeExactly 'HKCU\1\2\4'
        $out.results.result[0].afterState.registryEntries[1].valueName | Should -BeExactly 'Hello2'
        $out.results.result[0].afterState.registryEntries[1].valueData.String | Should -BeExactly 'World2_after'
        $out.results.result[0].afterState.registryEntries[2].keyPath | Should -BeExactly 'HKCU\1\2\5'
        $out.results.result[0].afterState.registryEntries[2]._metadata.whatIf[0] | Should -Match "Would delete subkey '5'"
        $out.results.result[0].afterState.registryEntries[3].keyPath | Should -BeExactly 'HKCU\1\2\6'
        $out.results.result[0].afterState.registryEntries[3]._metadata.whatIf[0] | Should -Match "Would delete value 'Hello4'"
        $out.results.result[0].afterState.registryEntries[3].valueName | Should -BeExactly 'Hello4'
    }
}
