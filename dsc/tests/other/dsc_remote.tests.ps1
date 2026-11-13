# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

# These tests cover `dsc --target ssh://...`. No SSH daemon is needed: the DSC_SSH_EXECUTABLE
# environment variable points the client at a script that takes the place of `ssh` and connects
# the client's stdin and stdout to a local process, which is what `ssh -s <host> dsc` does.

Describe 'Tests for remote configuration over SSH' {
    BeforeAll {
        $script:fakeDir = Join-Path $TestDrive 'fakessh'
        $null = New-Item -ItemType Directory -Path $fakeDir

        # Creates a stand-in for the ssh client. It records its arguments in '<name>.args.txt'
        # and then runs the given commands.
        function New-FakeSsh {
            param(
                [string]$Name,
                [string[]]$WindowsCommands,
                [string[]]$UnixCommands
            )

            if ($IsWindows) {
                $path = Join-Path $fakeDir "$Name.cmd"
                $lines = @('@echo off', "echo %*> `"$fakeDir\$Name.args.txt`"") + $WindowsCommands
                Set-Content -Path $path -Value ($lines -join "`r`n")
            } else {
                $path = Join-Path $fakeDir $Name
                $lines = @('#!/bin/sh', "echo `"`$*`" > '$fakeDir/$Name.args.txt'") + $UnixCommands
                Set-Content -Path $path -Value ($lines -join "`n")
                chmod +x $path
            }
            $path
        }

        $serverCommand = 'dsc --trace-format json --trace-level warn server'
        $script:workingSsh = New-FakeSsh -Name 'working' -WindowsCommands $serverCommand -UnixCommands $serverCommand

        $refusedMessage = 'ssh: connect to host web01 port 22: Connection refused'
        $script:refusedSsh = New-FakeSsh -Name 'refused' `
            -WindowsCommands "echo $refusedMessage 1>&2", 'exit /b 255' `
            -UnixCommands "echo '$refusedMessage' >&2", 'exit 255'

        # Answers the MCP handshake the way a DSC server from before the identity change does.
        $oldServerScript = Join-Path $fakeDir 'old-server.ps1'
        Set-Content -Path $oldServerScript -Value @'
$request = [Console]::In.ReadLine() | ConvertFrom-Json
$response = @{
    jsonrpc = '2.0'
    id      = $request.id
    result  = @{
        protocolVersion = $request.params.protocolVersion
        capabilities    = @{ tools = @{} }
        serverInfo      = @{ name = 'rmcp'; version = '3.8.0' }
    }
}
[Console]::Out.WriteLine(($response | ConvertTo-Json -Compress -Depth 5))
[Console]::Out.Flush()
while ($null -ne [Console]::In.ReadLine()) { }
'@
        $oldServerCommand = "pwsh -NoProfile -NoLogo -File `"$oldServerScript`""
        $script:oldServerSsh = New-FakeSsh -Name 'old' -WindowsCommands $oldServerCommand -UnixCommands $oldServerCommand

        $script:echoConfig = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  greeting:
    type: string
    defaultValue: hello
resources:
- name: echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[parameters('greeting')]"
'@
        $script:errorFile = Join-Path $TestDrive 'stderr.txt'
    }

    BeforeEach {
        $env:DSC_SSH_EXECUTABLE = $workingSsh
    }

    AfterAll {
        $env:DSC_SSH_EXECUTABLE = $null
    }

    Context 'Running operations' {
        It 'Runs config get on the target and requests the dsc subsystem' {
            $out = dsc --target ssh://admin@web01:2222 config get -i $echoConfig 2> $errorFile | ConvertFrom-Json -Depth 20
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content $errorFile -Raw)
            $out.hadErrors | Should -BeFalse
            $out.results[0].type | Should -BeExactly 'Microsoft.DSC.Debug/Echo'
            $out.results[0].result.actualState.output | Should -BeExactly 'hello'
            $out.metadata.'Microsoft.DSC'.operation | Should -BeExactly 'get'
            (Get-Content (Join-Path $fakeDir 'working.args.txt')).Trim() | Should -BeExactly '-s -p 2222 -l admin -- web01 dsc'
        }

        It 'Leaves port and user to the ssh client when the target has none' {
            $null = dsc --target ssh://web01 config get -i $echoConfig 2> $errorFile
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content $errorFile -Raw)
            (Get-Content (Join-Path $fakeDir 'working.args.txt')).Trim() | Should -BeExactly '-s -- web01 dsc'
        }

        It 'Sends parameters from the command line and from a file' {
            $parametersFile = Join-Path $TestDrive 'remote.parameters.yaml'
            Set-Content -Path $parametersFile -Value "parameters:`n  greeting: from file"

            $out = dsc --target ssh://web01 config --parameters-file $parametersFile get -i $echoConfig 2> $errorFile | ConvertFrom-Json -Depth 20
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content $errorFile -Raw)
            $out.results[0].result.actualState.output | Should -BeExactly 'from file'

            $inline = '{"parameters":{"greeting":"from inline"}}'
            $out = dsc --target ssh://web01 config --parameters-file $parametersFile --parameters $inline get -i $echoConfig 2> $errorFile | ConvertFrom-Json -Depth 20
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content $errorFile -Raw)
            $out.results[0].result.actualState.output | Should -BeExactly 'from inline'
        }

        It 'Reads the document from a file and from STDIN on the client' {
            $configFile = Join-Path $TestDrive 'remote.dsc.yaml'
            Set-Content -Path $configFile -Value $echoConfig

            $out = dsc --target ssh://web01 config test -f $configFile 2> $errorFile | ConvertFrom-Json -Depth 20
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content $errorFile -Raw)
            $out.results[0].result.inDesiredState | Should -BeTrue

            $out = $echoConfig | dsc --target ssh://web01 config test -f - 2> $errorFile | ConvertFrom-Json -Depth 20
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content $errorFile -Raw)
            $out.results[0].result.inDesiredState | Should -BeTrue
        }

        It 'Runs config set as what-if when asked' {
            $out = dsc --target ssh://web01 config set --what-if -i $echoConfig 2> $errorFile | ConvertFrom-Json -Depth 20
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content $errorFile -Raw)
            $out.metadata.'Microsoft.DSC'.executionType | Should -BeExactly 'whatIf'

            $out = dsc --target ssh://web01 config set -i $echoConfig 2> $errorFile | ConvertFrom-Json -Depth 20
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content $errorFile -Raw)
            $out.metadata.'Microsoft.DSC'.executionType | Should -BeExactly 'actual'
        }

        It 'Prints the exported configuration like a local export' {
            $exportConfig = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: os
  type: Microsoft/OSInfo
'@
            $remote = dsc --target ssh://web01 config export -i $exportConfig 2> $errorFile | ConvertFrom-Json -Depth 20
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content $errorFile -Raw)
            $local = dsc config export -i $exportConfig 2> $null | ConvertFrom-Json -Depth 20

            $remote.resources[0].type | Should -BeExactly 'Microsoft/OSInfo'
            $remote.PSObject.Properties.Name | Should -Be $local.PSObject.Properties.Name
        }

        It 'Honors the output format' {
            $out = dsc --target ssh://web01 config get -i $echoConfig -o yaml 2> $errorFile
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content $errorFile -Raw)
            ($out -join "`n") | Should -Match '(?m)^hadErrors: false$'
        }

        It 'Returns the exit code of a local run when the target rejects the parameters' {
            $parameters = '{"parameters":{"notDefined":"value"}}'
            $null = dsc --target ssh://web01 config --parameters $parameters get -i $echoConfig 2> $errorFile
            $LASTEXITCODE | Should -Be 4
            $remoteError = Get-Content $errorFile -Raw

            $null = dsc config --parameters $parameters get -i $echoConfig 2> $errorFile
            $LASTEXITCODE | Should -Be 4
            $remoteError | Should -Match "notDefined"
        }

        It 'Warns that included files are resolved on the target' {
            $includeConfig = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: included
  type: Microsoft.DSC/Include
  properties:
    configurationContent: |
      $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
      resources:
      - name: echo
        type: Microsoft.DSC.Debug/Echo
        properties:
          output: included
'@
            $null = dsc -l warn -t plaintext --target ssh://web01 config get -i $includeConfig 2> $errorFile
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content $errorFile -Raw)
            Get-Content $errorFile -Raw | Should -Match "Microsoft.DSC/Include.*resolved on 'ssh://web01'"
        }
    }

    Context 'Rejecting commands and targets before connecting' {
        BeforeEach {
            Remove-Item -Path (Join-Path $fakeDir 'working.args.txt') -ErrorAction Ignore
        }

        It 'Rejects <name> with --target' -TestCases @(
            @{ name = 'resource list'; arguments = @('resource', 'list') }
            @{ name = 'extension list'; arguments = @('extension', 'list') }
            @{ name = 'function list'; arguments = @('function', 'list') }
            @{ name = 'schema'; arguments = @('schema', '--type', 'configuration') }
            @{ name = 'server'; arguments = @('server') }
            @{ name = 'config with --system-root'; arguments = @('config', '--system-root', '.', 'get', '-i', '{}') }
            @{ name = 'config validate'; arguments = @('config', 'validate', '-i', '{}') }
        ) {
            param($arguments)

            $null = dsc --target ssh://web01 @arguments 2> $errorFile
            $LASTEXITCODE | Should -Be 1
            Get-Content $errorFile -Raw | Should -Match "'--target' option is only supported"
            Join-Path $fakeDir 'working.args.txt' | Should -Not -Exist
        }

        It 'Rejects the target <target>' -TestCases @(
            @{ target = 'web01'; reason = 'not a URI' }
            @{ target = 'https://web01'; reason = "scheme 'https' is not supported" }
            @{ target = 'ssh://-oProxyCommand=calc'; reason = 'not a valid host' }
            @{ target = 'ssh://-oProxyCommand=calc@web01'; reason = 'not a valid user' }
            @{ target = 'ssh://admin:secret@web01'; reason = 'must not contain a password' }
            @{ target = 'ssh://web01/etc/dsc'; reason = 'must not contain a path' }
        ) {
            param($target, $reason)

            $null = dsc --target $target config get -i $echoConfig 2> $errorFile
            $LASTEXITCODE | Should -Be 1
            Get-Content $errorFile -Raw | Should -Match ([regex]::Escape($reason))
            Join-Path $fakeDir 'working.args.txt' | Should -Not -Exist
        }
    }

    Context 'Connection failures' {
        It 'Reports what ssh wrote when the connection fails' {
            $env:DSC_SSH_EXECUTABLE = $refusedSsh
            $null = dsc --target ssh://web01 config get -i $echoConfig 2> $errorFile
            $LASTEXITCODE | Should -Be 11
            Get-Content $errorFile -Raw | Should -Match "Could not connect to 'ssh://web01'.*Connection refused"
        }

        It 'Fails when the ssh client cannot be started' {
            $env:DSC_SSH_EXECUTABLE = Join-Path $fakeDir 'does-not-exist'
            $null = dsc --target ssh://web01 config get -i $echoConfig 2> $errorFile
            $LASTEXITCODE | Should -Be 11
            Get-Content $errorFile -Raw | Should -Match 'Failed to start the SSH client'
        }

        It 'Refuses a server that does not identify itself as a supported DSC version' {
            $env:DSC_SSH_EXECUTABLE = $oldServerSsh
            $null = dsc --target ssh://web01 config set --what-if -i $echoConfig 2> $errorFile
            $LASTEXITCODE | Should -Be 11
            Get-Content $errorFile -Raw | Should -Match "the server reports 'rmcp 3.8.0'"
        }
    }

    Context 'Remote traces' {
        It 'Emits traces from the target with the host as prefix' {
            # The server logs a warning when a tool call fails; an undefined parameter makes the
            # call fail without needing any resource.
            $parameters = '{"parameters":{"notDefined":"value"}}'
            $null = dsc -l warn -t plaintext --target ssh://web01 config --parameters $parameters get -i $echoConfig 2> $errorFile
            $LASTEXITCODE | Should -Be 4
            Get-Content $errorFile -Raw | Should -Match 'WARN web01: '
        }
    }
}
