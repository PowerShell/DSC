# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Registry list get tests' {
    It 'Can get a registry list' -Skip:(!$IsWindows) {
        $config_yaml = @'
            $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Reg 1
              type: Microsoft.Windows/RegistryList
              properties:
                registryEntries:
                - keyPath: HKLM\Software\Microsoft\Windows\CurrentVersion
                  valueName: ProgramFilesDir
                - keyPath: HKLM\Software\Microsoft\Windows\CurrentVersion
                  valueName: ProgramFilesPath
                - keyPath: HKLM\Software\Microsoft\Windows\CurrentVersion
                  valueName: CommonFilesDir
                - keyPath: HKLM\Software\Microsoft\Windows\CurrentVersion
                  valueName: NonExistentValue
'@
        $out = dsc config get --input $config_yaml 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results.result[0].actualState.registryEntries.Count | Should -Be 4 -Because ($out | ConvertTo-Json -Depth 10)
        $out.results.result[0].actualState.registryEntries[0].keyPath | Should -BeExactly 'HKLM\Software\Microsoft\Windows\CurrentVersion'
        $out.results.result[0].actualState.registryEntries[0].valueName | Should -BeExactly 'ProgramFilesDir'
        $out.results.result[0].actualState.registryEntries[0].valueData.String | Should -BeExactly $env:ProgramFiles
        $out.results.result[0].actualState.registryEntries[1].keyPath | Should -BeExactly 'HKLM\Software\Microsoft\Windows\CurrentVersion'
        $out.results.result[0].actualState.registryEntries[1].valueName | Should -BeExactly 'ProgramFilesPath'
        $out.results.result[0].actualState.registryEntries[1].valueData.ExpandString | Should -BeExactly '%ProgramFiles%'
        $out.results.result[0].actualState.registryEntries[2].keyPath | Should -BeExactly 'HKLM\Software\Microsoft\Windows\CurrentVersion'
        $out.results.result[0].actualState.registryEntries[2].valueName | Should -BeExactly 'CommonFilesDir'
        $out.results.result[0].actualState.registryEntries[2].valueData.String | Should -BeExactly ($env:ProgramFiles + '\Common Files')
        $out.results.result[0].actualState.registryEntries[3].keyPath | Should -BeExactly 'HKLM\Software\Microsoft\Windows\CurrentVersion'
        $out.results.result[0].actualState.registryEntries[3].valueName | Should -BeExactly 'NonExistentValue'
        $out.results.result[0].actualState.registryEntries[3]._exist | Should -BeFalse
    }
}
