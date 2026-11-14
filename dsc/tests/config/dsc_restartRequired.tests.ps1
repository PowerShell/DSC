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
        outputs:
          system:
            type: bool
            value: "[restartRequired('system')]"
          service:
            type: bool
            value: "[restartRequired('service', 'sshd')]"
          process:
            type: bool
            value: "[restartRequired('process', 'myProcess')]"
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
        $out.executionInformation.restartRequired.count | Should -Be 5
        $out.executionInformation.restartRequired[0].system | Should -BeExactly 'mySystem'
        $out.executionInformation.restartRequired[1].service | Should -BeExactly 'myService'
        $out.executionInformation.restartRequired[2].service | Should -BeExactly 'sshd'
        $out.executionInformation.restartRequired[3].process.name | Should -BeExactly 'myProcess'
        $out.executionInformation.restartRequired[3].process.id | Should -Be 1234
        $out.executionInformation.restartRequired[4].process.name | Should -BeExactly 'anotherProcess'
        $out.executionInformation.restartRequired[4].process.id | Should -Be 5678
        $out.outputs.system | Should -Be $true -Because ($out | ConvertTo-Json -Depth 10)
        $out.outputs.service | Should -Be $true -Because ($out | ConvertTo-Json -Depth 10)
        $out.outputs.process | Should -Be $true -Because ($out | ConvertTo-Json -Depth 10)
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
        $out.results[0].executionInformation.restartRequired | Should -BeNullOrEmpty
        $out.executionInformation.restartRequired | Should -BeNullOrEmpty
    }

    It 'restartRequired function returns false for unknown resource: <type>' -TestCases @(
      @{ type = 'system' }
      @{ type = 'service'; name = ", 'unknown'" }
      @{ type = 'process'; name = ", 'unknown'" }
    ){
        param($type, $name)

        $configYaml = @"
        `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
          - name: test
            type: Test/RestartRequired
            properties:
              _restartRequired:
                - service: myService
                - process:
                    name: myProcess
                    id: 1234
        outputs:
          unknown:
            type: bool
            value: "[restartRequired('$type'$name)]"
"@
        $out = dsc config get -i $configYaml 2>$TestDrive/error.log | ConvertFrom-Json
        $errorContent = Get-Content $TestDrive/error.log -Raw
        $LASTEXITCODE | Should -Be 0 -Because $errorContent
        $out.outputs.unknown | Should -Be $false -Because ($out | ConvertTo-Json -Depth 10)
    }

    It 'restartRequired function returns error if name not specified for: <type>' -TestCases @(
      @{ type = 'service' }
      @{ type = 'process' }
    ){
        param($type)

        $configYaml = @"
        `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
          - name: test
            type: Test/RestartRequired
            properties:
              _restartRequired:
                - service: myService
                - process:
                    name: myProcess
                    id: 1234
        outputs:
          unknown:
            type: bool
            value: "[restartRequired('$type')]"
"@
        $null = dsc config get -i $configYaml 2>$TestDrive/error.log | ConvertFrom-Json
        $errorContent = Get-Content $TestDrive/error.log -Raw
        $LASTEXITCODE | Should -Be 2 -Because $errorContent
        $errorContent | Should -BeLike "*ERROR*The 'name' argument is required for kind '$type'*" -Because $errorContent
    }

    It 'restartRequired function returns error if invalid kind specified' {
        $configYaml = @"
        `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
          - name: test
            type: Test/RestartRequired
            properties:
              _restartRequired:
                - service: myService
                - process:
                    name: myProcess
                    id: 1234
        outputs:
          unknown:
            type: bool
            value: "[restartRequired('invalidKind')]"
"@
        $null = dsc config get -i $configYaml 2>$TestDrive/error.log | ConvertFrom-Json
        $errorContent = Get-Content $TestDrive/error.log -Raw
        $LASTEXITCODE | Should -Be 2 -Because $errorContent
        $errorContent | Should -BeLike "*ERROR*Invalid kind 'invalidKind', must be one of: process, service, system*" -Because $errorContent
    }

    It 'restartRequired function returns an error if name used with system kind' {
        $configYaml = @"
        `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
          - name: test
            type: Test/RestartRequired
            properties:
              _restartRequired:
                - system: mySystem
        outputs:
          unknown:
            type: bool
            value: "[restartRequired('system', 'nameNotAllowed')]"
"@
        $null = dsc config get -i $configYaml 2>$TestDrive/error.log | ConvertFrom-Json
        $errorContent = Get-Content $TestDrive/error.log -Raw
        $LASTEXITCODE | Should -Be 2 -Because $errorContent
        $errorContent | Should -BeLike "*ERROR*The 'name' argument is not allowed for kind 'system'*" -Because $errorContent
    }
}
