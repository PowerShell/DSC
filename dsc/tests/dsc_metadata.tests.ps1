# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'metadata tests' {
    It 'metadata not provided if not declared in resource schema' {
        $configYaml = @'
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
          - name: test
            type: Microsoft.DSC.Debug/Echo
            metadata:
              ignoreKey: true
            properties:
              output: hello world
'@
        $out = dsc -l info config get -i $configYaml 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        (Get-Content $TestDrive/error.log -Raw) | Should -BeLike "*INFO Will not add '_metadata' to properties because resource schema does not support it*" -Because (Get-Content $TestDrive/error.log -Raw)
        $out.results.result.actualState.output | Should -BeExactly 'hello world'
    }

    It 'resource can provide high-level metadata for <operation>' -TestCases @(
        @{ operation = 'get' }
        @{ operation = 'set' }
        @{ operation = 'test' }
    ) {
        param($operation)

        $configYaml = @'
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
          - name: test
            type: Test/Metadata
            metadata:
              hello: world
              myNumber: 42
            properties:
'@

        $out = dsc config $operation -i $configYaml 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results.count | Should -Be 1
        $out.results[0].metadata.hello | Should -BeExactly 'world'
        $out.results[0].metadata.myNumber | Should -Be 42
    }

    It 'resource can provide high-level metadata for export' {
        $configYaml = @'
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
          - name: test
            type: Test/Metadata
            metadata:
              hello: There
              myNumber: 16
            properties:
'@
        $out = dsc config export -i $configYaml 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.resources.count | Should -Be 3
        $out.resources[0].metadata.hello | Should -BeExactly 'There'
        $out.resources[0].metadata.myNumber | Should -Be 16
        $out.resources[0].name | Should -BeExactly 'Metadata example 1'
        $out.resources[1].metadata.hello | Should -BeExactly 'There'
        $out.resources[1].metadata.myNumber | Should -Be 16
        $out.resources[1].name | Should -BeExactly 'Metadata example 2'
        $out.resources[2].metadata.hello | Should -BeExactly 'There'
        $out.resources[2].metadata.myNumber | Should -Be 16
        $out.resources[2].name | Should -BeExactly 'Metadata example 3'
    }

    It 'resource can provide metadata for <operation>' -TestCases @(
        @{ operation = 'get' }
        @{ operation = 'set' }
        @{ operation = 'test' }
    ) {
        param($operation)

        $configYaml = @'
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
          - name: test
            type: Test/Metadata
            properties:
              _metadata:
                hello: world
                myNumber: 42
'@

        $out = dsc config $operation -i $configYaml 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results.count | Should -Be 1
        $out.results[0].metadata.hello | Should -BeExactly 'world'
        $out.results[0].metadata.myNumber | Should -Be 42
    }

    It 'resource can provide metadata for export' {
        $configYaml = @'
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
          - name: test
            type: Test/Metadata
            properties:
              _metadata:
                hello: There
                myNumber: 16
'@
        $out = dsc config export -i $configYaml 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.resources.count | Should -Be 3
        $out.resources[0].metadata.hello | Should -BeExactly 'There'
        $out.resources[0].metadata.myNumber | Should -Be 16
        $out.resources[0].name | Should -BeExactly 'Metadata example 1'
        $out.resources[1].metadata.hello | Should -BeExactly 'There'
        $out.resources[1].metadata.myNumber | Should -Be 16
        $out.resources[1].name | Should -BeExactly 'Metadata example 2'
        $out.resources[2].metadata.hello | Should -BeExactly 'There'
        $out.resources[2].metadata.myNumber | Should -Be 16
        $out.resources[2].name | Should -BeExactly 'Metadata example 3'
    }

    It 'resource returning Microsoft.DSC metadata is ignored' {
        $configYaml = @'
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
          - name: test
            type: Test/Metadata
            properties:
              _metadata:
                Microsoft.DSC:
                  hello: world
                validOne: true
'@
        $out = dsc config get -i $configYaml 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results.count | Should -Be 1
        $out.results[0].metadata.validOne | Should -BeTrue
        $out.results[0].metadata.Microsoft.DSC | Should -BeNullOrEmpty
        (Get-Content $TestDrive/error.log) | Should -BeLike "*WARN*Resource returned '_metadata' property 'Microsoft.DSC' which is ignored*"
    }

    It 'resource returning _restartRequired metadata is handled' {
        $configYaml = @'
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
          - name: one
            type: Test/Metadata
            properties:
              _metadata:
                _restartRequired:
                  - system: mySystem
                  - service: myService
          - name: two
            type: Test/Metadata
            properties:
              _metadata:
                _restartRequired:
                  - service: sshd
          - name: three
            type: Test/Metadata
            properties:
              _metadata:
                _restartRequired:
                  - process:
                      name: myProcess
                      id: 1234
                  - process:
                      name: anotherProcess
                      id: 5678
'@
        $out = dsc config get -i $configYaml 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results.count | Should -Be 3
        $out.results[0].metadata._restartRequired.count | Should -Be 2
        $out.results[0].metadata._restartRequired[0].system | Should -BeExactly 'mySystem'
        $out.results[0].metadata._restartRequired[1].service | Should -BeExactly 'myService'
        $out.results[1].metadata._restartRequired.count | Should -Be 1
        $out.results[1].metadata._restartRequired[0].service | Should -BeExactly 'sshd'
        $out.results[2].metadata._restartRequired.count | Should -Be 2
        $out.results[2].metadata._restartRequired[0].process.name | Should -BeExactly 'myProcess'
        $out.results[2].metadata._restartRequired[0].process.id | Should -Be 1234
        $out.results[2].metadata._restartRequired[1].process.name | Should -BeExactly 'anotherProcess'
        $out.results[2].metadata._restartRequired[1].process.id | Should -Be 5678
        $out.metadata.'Microsoft.DSC'.restartRequired.count | Should -Be 5
        $out.metadata.'Microsoft.DSC'.restartRequired[0].system | Should -BeExactly 'mySystem'
        $out.metadata.'Microsoft.DSC'.restartRequired[1].service | Should -BeExactly 'myService'
        $out.metadata.'Microsoft.DSC'.restartRequired[2].service | Should -BeExactly 'sshd'
        $out.metadata.'Microsoft.DSC'.restartRequired[3].process.name | Should -BeExactly 'myProcess'
        $out.metadata.'Microsoft.DSC'.restartRequired[3].process.id | Should -Be 1234
        $out.metadata.'Microsoft.DSC'.restartRequired[4].process.name | Should -BeExactly 'anotherProcess'
        $out.metadata.'Microsoft.DSC'.restartRequired[4].process.id | Should -Be 5678
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
            type: Test/Metadata
            properties:
              _metadata:
                _restartRequired:
                  - invalid: item
'@
        $out = dsc config get -i $configYaml 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        (Get-Content $TestDrive/error.log) | Should -BeLike "*WARN*Resource returned '_metadata' property '_restartRequired' which contains invalid value: ``[{`"invalid`":`"item`"}]*"
        $out.results[0].metadata._restartRequired | Should -BeNullOrEmpty
    }
}
