# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe '_restartRequired tests' {
    It 'resource returning _restartRequired metadata is handled' {
        $configYaml = @'
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
          - name: one
            type: Test/RestartRequired
            properties:
              _restartRequired:
                - system: mySystem
                - service: myService
          - name: two
            type: Test/RestartRequired
            properties:
              _restartRequired:
                - service: sshd
          - name: three
            type: Test/RestartRequired
            properties:
              _restartRequired:
                - process:
                    name: myProcess
                    id: 1234
                - process:
                    name: anotherProcess
                    id: 5678
'@
        $out = dsc -l trace config get -i $configYaml 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
        $out.results.count | Should -Be 3
        $out.results[0].executionInformation.restartRequired.count | Should -Be 2
        $out.results[0].executionInformation.restartRequired[0].system | Should -BeExactly 'mySystem'
        $out.results[0].executionInformation.restartRequired[1].service | Should -BeExactly 'myService'
        $out.results[1].executionInformation.restartRequired.count | Should -Be 1
        $out.results[1].executionInformation.restartRequired[0].service | Should -BeExactly 'sshd'
        $out.results[2].executionInformation.restartRequired.count | Should -Be 2
        $out.results[2].executionInformation.restartRequired[0].process.name | Should -BeExactly 'myProcess'
        $out.results[2].executionInformation.restartRequired[0].process.id | Should -Be 1234
        $out.results[2].executionInformation.restartRequired[1].process.name | Should -BeExactly 'anotherProcess'
        $out.results[2].executionInformation.restartRequired[1].process.id | Should -Be 5678
        $out.results[0].executionInformation.restartRequired.count | Should -Be 2
        $out.results[0].executionInformation.restartRequired[0].system | Should -BeExactly 'mySystem'
        $out.results[0].executionInformation.restartRequired[1].service | Should -BeExactly 'myService'
        $out.results[1].executionInformation.restartRequired.count | Should -Be 1
        $out.results[1].executionInformation.restartRequired[0].service | Should -BeExactly 'sshd'
        $out.results[2].executionInformation.restartRequired.count | Should -Be 2
        $out.results[2].executionInformation.restartRequired[0].process.name | Should -BeExactly 'myProcess'
        $out.results[2].executionInformation.restartRequired[0].process.id | Should -Be 1234
        $out.executionInformation.restartRequired.count | Should -Be 5
        $out.executionInformation.restartRequired[0].system | Should -BeExactly 'mySystem'
        $out.executionInformation.restartRequired[1].service | Should -BeExactly 'myService'
        $out.executionInformation.restartRequired[2].service | Should -BeExactly 'sshd'
        $out.executionInformation.restartRequired[3].process.name | Should -BeExactly 'myProcess'
        $out.executionInformation.restartRequired[3].process.id | Should -Be 1234
        $out.executionInformation.restartRequired[4].process.name | Should -BeExactly 'anotherProcess'
        $out.executionInformation.restartRequired[4].process.id | Should -Be 5678
    }

    It 'invalid item in _restartRequired metadata is a warning' {
        $configYaml = @'
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
          - name: test
            type: Test/RestartRequired
            properties:
              _restartRequired:
                - invalid: item
'@
        $out = dsc config get -i $configYaml 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
        (Get-Content $TestDrive/error.log) | Should -BeLike "*WARN*Resource returned property '_restartRequired' which contains invalid value: ``[{`"invalid`":`"item`"}]*" -Because (Get-Content $TestDrive/error.log -Raw)
        $out.results[0].metadata._restartRequired | Should -BeNullOrEmpty
    }
}
