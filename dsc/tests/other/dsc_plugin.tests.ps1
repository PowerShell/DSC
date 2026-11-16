# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Command plug-in tests' {
    BeforeAll {
        $script:oldPath = $env:PATH
        $script:pathSeparator = [System.IO.Path]::PathSeparator
        $script:manifestSchema = 'https://aka.ms/dsc/schemas/v3/bundled/plugin/manifest.json'

        # A plug-in that reports how it was invoked as one line of JSON.
        $script:echoScript = @'
if ($args -contains '--help') {
    "Usage: dsc $env:DSC_PLUGIN_COMMAND [OPTIONS]"
    "HELP_ARGS: $($args -join '|')"
    exit 0
}

$exitCode = 0
$stdin = $null
for ($i = 0; $i -lt $args.Count; $i++) {
    if ($args[$i] -eq '--exit-code') { $exitCode = [int]$args[$i + 1] }
    if ($args[$i] -eq '--read-stdin') { $stdin = @($input) -join "`n" }
}

[ordered]@{
    args  = @($args)
    # Report a marker file instead of the path, which can differ in form from the path the test
    # used when a parent directory is a symbolic link.
    cwdHasMarker = Test-Path -Path (Join-Path (Get-Location).Path 'plugin-cwd-marker.txt')
    stdin = $stdin
    env   = [ordered]@{
        DSC_EXE                  = $env:DSC_EXE
        DSC_VERSION              = $env:DSC_VERSION
        DSC_PLUGIN_COMMAND       = $env:DSC_PLUGIN_COMMAND
        DSC_PLUGIN_DIR           = $env:DSC_PLUGIN_DIR
        DSC_TRACE_LEVEL          = $env:DSC_TRACE_LEVEL
        DSC_TRACE_FORMAT         = $env:DSC_TRACE_FORMAT
        DSC_PROGRESS_FORMAT      = $env:DSC_PROGRESS_FORMAT
        DSC_IGNORE_SETTINGS_FILE = $env:DSC_IGNORE_SETTINGS_FILE
    }
} | ConvertTo-Json -Compress -Depth 5

exit $exitCode
'@

        function New-TestPluginManifest {
            <#
                .SYNOPSIS
                Returns the manifest for a plug-in that runs the echo script next to the manifest.
            #>
            param(
                [Parameter(Mandatory)][string]$Type,
                [Parameter(Mandatory)][string]$Command,
                [string]$Version = '1.0.0',
                [string]$Description = 'Report how the plug-in was invoked',
                [hashtable]$Extra = @{}
            )

            $manifest = [ordered]@{
                '$schema'   = $script:manifestSchema
                type        = $Type
                version     = $Version
                description = $Description
                command     = [ordered]@{
                    name       = $Command
                    executable = 'pwsh'
                    args       = @(
                        '-NoLogo'
                        '-NoProfile'
                        '-NonInteractive'
                        '-File'
                        @{ manifestRelativePath = 'echo.ps1' }
                    )
                }
            }
            foreach ($key in $Extra.Keys) {
                $manifest[$key] = $Extra[$key]
            }
            $manifest
        }

        function New-TestPlugin {
            <#
                .SYNOPSIS
                Writes a plug-in manifest and the echo script to a directory.
            #>
            param(
                [Parameter(Mandatory)][string]$Directory,
                [Parameter(Mandatory)][string]$Type,
                [Parameter(Mandatory)][string]$Command,
                [string]$Version = '1.0.0',
                [string]$Description = 'Report how the plug-in was invoked',
                [hashtable]$Extra = @{}
            )

            $null = New-Item -ItemType Directory -Force -Path $Directory
            Set-Content -Path (Join-Path $Directory 'echo.ps1') -Value $script:echoScript
            $fileName = '{0}.{1}.dsc.plugin.json' -f ($Type -replace '/', '.'), $Command
            New-TestPluginManifest -Type $Type -Command $Command -Version $Version -Description $Description -Extra $Extra |
                ConvertTo-Json -Depth 10 |
                Set-Content -Path (Join-Path $Directory $fileName)
        }

        function Use-PluginDirectory {
            <#
                .SYNOPSIS
                Makes DSC search the directories for manifests, before every other directory.
            #>
            param([Parameter(Mandatory)][string[]]$Directory)

            $env:PATH = ($Directory + $script:oldPath) -join $script:pathSeparator
        }

        $script:pluginDir = Join-Path $TestDrive 'plugins'
        New-TestPlugin -Directory $script:pluginDir -Type 'Test/Echo' -Command 'echo' -Version '1.2.3'
        Use-PluginDirectory -Directory $script:pluginDir
    }

    AfterAll {
        $env:PATH = $script:oldPath
    }

    Context 'Listing plug-ins' {
        It 'Lists a plug-in with its command and manifest' {
            $out = dsc plugin list 'Test/Echo' 2> $TestDrive/error.txt | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            @($out).Count | Should -Be 1
            $out.type | Should -BeExactly 'Test/Echo'
            $out.version | Should -BeExactly '1.2.3'
            $out.command | Should -BeExactly 'echo'
            $out.description | Should -BeExactly 'Report how the plug-in was invoked'
            $out.deprecationMessage | Should -BeNullOrEmpty
            $out.path | Should -BeExactly (Join-Path $pluginDir 'Test.Echo.echo.dsc.plugin.json')
            $out.directory | Should -BeExactly $pluginDir
            $out.shadowed | Should -BeFalse
            $out.manifest.command.name | Should -BeExactly 'echo'
            $out.manifest.command.executable | Should -BeExactly 'pwsh'
        }

        It 'Filters the list with a wildcard: <filter>' -TestCases @(
            @{ filter = 'Test/*'; expected = 1 }
            @{ filter = '*echo'; expected = 1 }
            @{ filter = 'Test/DoesNotExist'; expected = 0 }
        ) {
            param($filter, $expected)

            $out = dsc plugin list $filter 2> $TestDrive/error.txt | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            @($out).Count | Should -Be $expected
        }

        It 'Writes a table when asked to' {
            $out = dsc plugin list 'Test/Echo' -o table-no-truncate 2> $TestDrive/error.txt | Out-String
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out | Should -Match 'Command\s+Type\s+Version\s+Description'
            $out | Should -Match 'echo\s+Test/Echo\s+1\.2\.3\s+Report how the plug-in was invoked'
        }

        It 'Finds plug-ins in YAML manifests and manifest lists' {
            $directory = Join-Path $TestDrive 'formats'
            New-TestPlugin -Directory $directory -Type 'Test.Formats/Json' -Command 'formats-json'
            @"
`$schema: $manifestSchema
type: Test.Formats/Yaml
version: 1.0.0
command:
  name: formats-yaml
  executable: pwsh
"@ | Set-Content -Path (Join-Path $directory 'yaml.dsc.plugin.yaml')
            @{
                plugins = @(
                    New-TestPluginManifest -Type 'Test.Formats/ListOne' -Command 'formats-list-one'
                    New-TestPluginManifest -Type 'Test.Formats/ListTwo' -Command 'formats-list-two'
                )
            } | ConvertTo-Json -Depth 10 | Set-Content -Path (Join-Path $directory 'tools.dsc.manifests.json')

            try {
                Use-PluginDirectory -Directory $directory
                $out = dsc plugin list 'Test.Formats/*' 2> $TestDrive/error.txt | ConvertFrom-Json
                $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
                $out.command | Sort-Object | Should -Be @(
                    'formats-json'
                    'formats-list-one'
                    'formats-list-two'
                    'formats-yaml'
                )

                $result = dsc formats-list-two 2> $TestDrive/error.txt | ConvertFrom-Json
                $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
                $result.env.DSC_PLUGIN_COMMAND | Should -BeExactly 'formats-list-two'
            } finally {
                Use-PluginDirectory -Directory $pluginDir
            }
        }
    }

    Context 'Running a plug-in command' {
        It 'Passes the arguments after the command name to the plug-in as written' {
            $out = dsc echo one --two 'three four' -l trace --help-me -- five 2> $TestDrive/error.txt | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out.args | Should -Be @('one', '--two', 'three four', '-l', 'trace', '--help-me', '--', 'five')
        }

        It 'Runs the plug-in in the working directory of the caller' {
            $out = dsc echo 2> $TestDrive/error.txt | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out.cwdHasMarker | Should -BeFalse

            $workingDirectory = New-Item -ItemType Directory -Force -Path (Join-Path $TestDrive 'working')
            $null = New-Item -ItemType File -Force -Path (Join-Path $workingDirectory 'plugin-cwd-marker.txt')
            Push-Location $workingDirectory
            try {
                $out = dsc echo 2> $TestDrive/error.txt | ConvertFrom-Json
                $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
                $out.cwdHasMarker | Should -BeTrue
            } finally {
                Pop-Location
            }
        }

        It 'Passes standard input to the plug-in' {
            $out = 'hello from stdin' | dsc echo --read-stdin 2> $TestDrive/error.txt | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out.stdin | Should -BeExactly 'hello from stdin'
        }

        It 'Exits with the exit code of the plug-in: <exitCode>' -TestCases @(
            @{ exitCode = 0 }
            @{ exitCode = 1 }
            @{ exitCode = 5 }
            @{ exitCode = 42 }
        ) {
            param($exitCode)

            $null = dsc echo --exit-code $exitCode 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be $exitCode -Because (Get-Content -Raw $TestDrive/error.txt)
        }

        It 'Exits with an exit code above 255 on Windows' -Skip:(!$IsWindows) {
            $null = dsc echo --exit-code 1000 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 1000 -Because (Get-Content -Raw $TestDrive/error.txt)
        }

        It 'Tells the plug-in how it was invoked through environment variables' {
            $out = dsc echo 2> $TestDrive/error.txt | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $dscVersion = (dsc --version) -replace '^dsc\s+', ''
            $out.env.DSC_EXE | Should -Not -BeNullOrEmpty
            (Split-Path -Leaf $out.env.DSC_EXE) | Should -Match '^dsc(\.exe)?$'
            Test-Path -Path $out.env.DSC_EXE -PathType Leaf | Should -BeTrue
            $out.env.DSC_VERSION | Should -BeExactly $dscVersion
            $out.env.DSC_PLUGIN_COMMAND | Should -BeExactly 'echo'
            $out.env.DSC_PLUGIN_DIR | Should -BeExactly $pluginDir
            # The values depend on the settings in effect, so only check that DSC defines them.
            $out.env.DSC_TRACE_LEVEL | Should -Not -BeNullOrEmpty
            $out.env.DSC_TRACE_FORMAT | Should -Not -BeNullOrEmpty
            $out.env.DSC_PROGRESS_FORMAT | Should -BeNullOrEmpty
        }

        It 'Hands the options before the command to the plug-in as environment variables' {
            $out = dsc --trace-level info --trace-format json --progress-format none echo --trace-level error 2> $TestDrive/error.txt |
                ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out.args | Should -Be @('--trace-level', 'error')
            $out.env.DSC_TRACE_LEVEL | Should -Be 'info'
            $out.env.DSC_TRACE_FORMAT | Should -Be 'json'
            $out.env.DSC_PROGRESS_FORMAT | Should -Be 'none'
        }

        It 'Tells the plug-in to ignore the settings files when the user asked for that' {
            $out = dsc --ignore-settings-file echo 2> $TestDrive/error.txt | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out.env.DSC_IGNORE_SETTINGS_FILE | Should -Be '1'
        }

        It 'Fails when the executable of the plug-in cannot be found' {
            $directory = Join-Path $TestDrive 'missing-executable'
            $null = New-Item -ItemType Directory -Force -Path $directory
            $manifest = New-TestPluginManifest -Type 'Test/MissingExecutable' -Command 'missing-executable'
            $manifest.command.executable = 'dsc-test-executable-that-does-not-exist'
            $manifest | ConvertTo-Json -Depth 10 | Set-Content -Path (Join-Path $directory 'missing.dsc.plugin.json')

            try {
                Use-PluginDirectory -Directory $directory
                $out = dsc missing-executable 2> $TestDrive/error.txt
                $LASTEXITCODE | Should -Be 2
                $out | Should -BeNullOrEmpty
                $stderr = Get-Content -Raw $TestDrive/error.txt
                $stderr | Should -Match "missing-executable"
                $stderr | Should -Match "dsc-test-executable-that-does-not-exist"
            } finally {
                Use-PluginDirectory -Directory $pluginDir
            }
        }

        It 'Warns when the plug-in is deprecated' {
            $directory = Join-Path $TestDrive 'deprecated'
            New-TestPlugin -Directory $directory -Type 'Test/Deprecated' -Command 'deprecated' -Extra @{
                deprecationMessage = 'Use the echo command instead.'
            }

            try {
                Use-PluginDirectory -Directory $directory
                $out = dsc deprecated 2> $TestDrive/error.txt | ConvertFrom-Json
                $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
                $out.env.DSC_PLUGIN_COMMAND | Should -BeExactly 'deprecated'
                $stderr = Get-Content -Raw $TestDrive/error.txt
                $stderr | Should -Match 'WARN'
                $stderr | Should -Match "Test/Deprecated.*Use the echo command instead\."
            } finally {
                Use-PluginDirectory -Directory $pluginDir
            }
        }
    }

    Context 'Help' {
        It 'Passes --help after the command name to the plug-in' {
            $out = dsc echo --help 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out[0] | Should -BeExactly 'Usage: dsc echo [OPTIONS]'
            $out[1] | Should -BeExactly 'HELP_ARGS: --help'
        }

        It 'Shows the help of the plug-in for: dsc help <arguments>' -TestCases @(
            @{ arguments = @('echo'); expected = '--help' }
            @{ arguments = @('echo', 'sub'); expected = 'sub|--help' }
            @{ arguments = @('echo', 'sub', 'command'); expected = 'sub|command|--help' }
        ) {
            param($arguments, $expected)

            $out = dsc help @arguments 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out[0] | Should -BeExactly 'Usage: dsc echo [OPTIONS]'
            $out[1] | Should -BeExactly "HELP_ARGS: $expected"
        }

        It 'Lists plug-in commands in the top-level help for: dsc <arguments>' -TestCases @(
            @{ arguments = @('--help') }
            @{ arguments = @('-h') }
            @{ arguments = @('help') }
            @{ arguments = @('--trace-level', 'info', '--help') }
        ) {
            param($arguments)

            $out = dsc @arguments 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            Get-Content -Raw $TestDrive/error.txt | Should -BeNullOrEmpty
            $text = $out | Out-String
            $text | Should -Match '(?m)^Commands:\r?$'
            $text | Should -Match '(?m)^Plug-in commands:\r?$'
            $text | Should -Match '(?m)^  echo\s+Report how the plug-in was invoked\r?$'
            $text | Should -Match '(?m)^Options:\r?$'
            # The plug-in commands are listed between the built-in commands and the options.
            $text.IndexOf('Commands:') | Should -BeLessThan $text.IndexOf('Plug-in commands:')
            $text.IndexOf('Plug-in commands:') | Should -BeLessThan $text.IndexOf('Options:')
        }

        It 'Keeps the help of built-in commands unchanged for: dsc <arguments>' -TestCases @(
            @{ arguments = @('config', '--help') }
            @{ arguments = @('help', 'config') }
            @{ arguments = @('plugin', 'list', '--help') }
        ) {
            param($arguments)

            $out = dsc @arguments 2> $TestDrive/error.txt | Out-String
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out | Should -Not -Match 'Plug-in commands:'
            $out | Should -Match 'Usage: dsc(\.exe)? (config|plugin list)'
        }

        It 'Lists the plugin command with the built-in commands' {
            $out = dsc --help 2> $TestDrive/error.txt | Out-String
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out | Should -Match '(?m)^  plugin\s+Operations on DSC plug-ins\r?$'
        }
    }

    Context 'Commands that no plug-in provides' {
        It 'Reports an unrecognized subcommand and suggests a similar plug-in command' {
            $out = dsc ech 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 2
            $out | Should -BeNullOrEmpty
            $stderr = Get-Content -Raw $TestDrive/error.txt
            $stderr | Should -Match "error: unrecognized subcommand 'ech'"
            $stderr | Should -Match "tip: a similar subcommand exists: 'echo'"
        }

        It 'Reports an unrecognized subcommand and suggests a similar built-in command' {
            $null = dsc confg get 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 2
            $stderr = Get-Content -Raw $TestDrive/error.txt
            $stderr | Should -Match "error: unrecognized subcommand 'confg'"
            $stderr | Should -Match "tip: a similar subcommand exists: 'config'"
        }

        It 'Reports an unrecognized subcommand for help on an unknown command' {
            $null = dsc help does-not-exist 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 2
            Get-Content -Raw $TestDrive/error.txt | Should -Match "error: unrecognized subcommand 'does-not-exist'"
        }

        It 'Compares command names case-sensitively' {
            $null = dsc Echo 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 2
            Get-Content -Raw $TestDrive/error.txt | Should -Match "error: unrecognized subcommand 'Echo'"
        }
    }

    Context 'Completion scripts' {
        BeforeAll {
            # A command name that can't appear in a completion script for any other reason.
            $script:completionDir = Join-Path $TestDrive 'completion'
            New-TestPlugin -Directory $completionDir -Type 'Test/Completion' -Command 'completion-probe' -Description 'Probe for completion scripts'
            Use-PluginDirectory -Directory $completionDir
        }

        AfterAll {
            Use-PluginDirectory -Directory $pluginDir
        }

        It 'Includes plug-in commands in the completion script for <shell>' -TestCases @(
            @{ shell = 'bash' }
            @{ shell = 'elvish' }
            @{ shell = 'fish' }
            @{ shell = 'powershell' }
            @{ shell = 'zsh' }
        ) {
            param($shell)

            $out = dsc completer $shell 2> $TestDrive/error.txt | Out-String
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out | Should -Match 'completion-probe'
        }

        It 'Describes plug-in commands in the completion script' {
            $out = dsc completer powershell 2> $TestDrive/error.txt | Out-String
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out | Should -Match "'completion-probe', 'completion-probe', \[CompletionResultType\]::ParameterValue, 'Probe for completion scripts'"
        }
    }

    Context 'Manifests that are skipped' {
        BeforeAll {
            $script:skippedDir = Join-Path $TestDrive 'skipped'
            New-TestPlugin -Directory $skippedDir -Type 'Test.Skipped/Reserved' -Command 'config'
            New-TestPlugin -Directory $skippedDir -Type 'Test.Skipped/ReservedAlias' -Command 'mcp'
            New-TestPlugin -Directory $skippedDir -Type 'Test.Skipped/ReservedHelp' -Command 'help'
            New-TestPlugin -Directory $skippedDir -Type 'Test.Skipped/ConditionFalse' -Command 'condition-false' -Extra @{
                condition = '[equals(1, 2)]'
            }
            New-TestPlugin -Directory $skippedDir -Type 'Test.Skipped/ConditionTrue' -Command 'condition-true' -Extra @{
                condition = '[equals(1, 1)]'
            }
            '{ this is not a manifest' | Set-Content -Path (Join-Path $skippedDir 'broken.dsc.plugin.json')
            Use-PluginDirectory -Directory $skippedDir
        }

        AfterAll {
            Use-PluginDirectory -Directory $pluginDir
        }

        It 'Lists only the plug-ins that can provide a command' {
            $out = dsc plugin list 'Test.Skipped/*' 2> $TestDrive/error.txt | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            @($out).Count | Should -Be 1
            $out.type | Should -BeExactly 'Test.Skipped/ConditionTrue'
        }

        It 'Warns about a plug-in that uses the name of a built-in command: <command>' -TestCases @(
            @{ command = 'config'; type = 'Test.Skipped/Reserved' }
            @{ command = 'mcp'; type = 'Test.Skipped/ReservedAlias' }
            @{ command = 'help'; type = 'Test.Skipped/ReservedHelp' }
        ) {
            param($command, $type)

            $null = dsc plugin list 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 0
            $stderr = Get-Content -Raw $TestDrive/error.txt
            $stderr | Should -Match "WARN.*'$type'.*'$command'.*built-in command"
        }

        It 'Warns about a manifest that cannot be loaded' {
            $null = dsc plugin list 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 0
            Get-Content -Raw $TestDrive/error.txt | Should -Match 'WARN.*broken\.dsc\.plugin\.json'
        }

        It 'Never runs a plug-in in place of a built-in command' {
            $out = dsc config --help 2> $TestDrive/error.txt | Out-String
            $LASTEXITCODE | Should -Be 0
            $out | Should -Match 'Apply a configuration document'
            $out | Should -Not -Match 'HELP_ARGS'
        }

        It 'Does not provide the command of a plug-in whose condition is false' {
            $null = dsc condition-false 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 2
            Get-Content -Raw $TestDrive/error.txt | Should -Match "unrecognized subcommand 'condition-false'"
        }

        It 'Provides the command of a plug-in whose condition is true' {
            $out = dsc condition-true 2> $TestDrive/error.txt | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $out.env.DSC_PLUGIN_COMMAND | Should -BeExactly 'condition-true'
        }

        It 'Keeps warnings about manifests out of the top-level help' {
            $out = dsc --help 2> $TestDrive/error.txt | Out-String
            $LASTEXITCODE | Should -Be 0
            Get-Content -Raw $TestDrive/error.txt | Should -BeNullOrEmpty
            $out | Should -Match '(?m)^  condition-true\s'
            $out | Should -Not -Match '(?m)^  condition-false\s'
        }
    }

    Context 'Precedence between plug-ins' {
        BeforeAll {
            $script:firstDir = Join-Path $TestDrive 'first'
            $script:secondDir = Join-Path $TestDrive 'second'
            # The same type in two directories: the highest version wins, wherever it is.
            New-TestPlugin -Directory $firstDir -Type 'Test.Precedence/Versioned' -Command 'versioned' -Version '1.0.0'
            New-TestPlugin -Directory $secondDir -Type 'Test.Precedence/Versioned' -Command 'versioned' -Version '2.0.0'
            # Different types with the same command: the first directory wins.
            New-TestPlugin -Directory $firstDir -Type 'Test.Precedence/Zeta' -Command 'shared' -Version '1.0.0'
            New-TestPlugin -Directory $secondDir -Type 'Test.Precedence/Alpha' -Command 'shared' -Version '9.0.0'
            Use-PluginDirectory -Directory $firstDir, $secondDir
        }

        AfterAll {
            Use-PluginDirectory -Directory $pluginDir
        }

        It 'Uses the highest version of a plug-in type' {
            $out = dsc plugin list 'Test.Precedence/Versioned' 2> $TestDrive/error.txt | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            @($out).Count | Should -Be 1
            $out.version | Should -BeExactly '2.0.0'
            $out.directory | Should -BeExactly $secondDir

            $result = dsc versioned 2> $TestDrive/error.txt | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $result.env.DSC_PLUGIN_DIR | Should -BeExactly $secondDir
        }

        It 'Reports the plug-in found later as shadowed' {
            $out = dsc plugin list 'Test.Precedence/*' 2> $TestDrive/error.txt | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            ($out | Where-Object type -EQ 'Test.Precedence/Zeta').shadowed | Should -BeFalse
            ($out | Where-Object type -EQ 'Test.Precedence/Alpha').shadowed | Should -BeTrue
        }

        It 'Marks the shadowed plug-in in the table' {
            $out = dsc plugin list 'Test.Precedence/*' -o table-no-truncate 2> $TestDrive/error.txt | Out-String
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out | Should -Match 'shared \(shadowed\)\s+Test\.Precedence/Alpha'
            $out | Should -Match 'shared\s+Test\.Precedence/Zeta'
        }

        It 'Runs the plug-in found first and warns about the shadowed plug-in' {
            $out = dsc shared 2> $TestDrive/error.txt | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out.env.DSC_PLUGIN_DIR | Should -BeExactly $firstDir
            $out.env.DSC_PLUGIN_COMMAND | Should -BeExactly 'shared'
            $stderr = Get-Content -Raw $TestDrive/error.txt
            $stderr | Should -Match "WARN.*'Test\.Precedence/Zeta'.*'Test\.Precedence/Alpha'"
            $stderr | Should -Match "'dsc Test\.Precedence/Alpha'"
        }

        It 'Runs a shadowed plug-in by its type name without a warning' {
            $out = dsc Test.Precedence/Alpha one --two 2> $TestDrive/error.txt | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out.args | Should -Be @('one', '--two')
            $out.env.DSC_PLUGIN_DIR | Should -BeExactly $secondDir
            $out.env.DSC_PLUGIN_COMMAND | Should -BeExactly 'Test.Precedence/Alpha'
            Get-Content -Raw $TestDrive/error.txt | Should -Not -Match 'shadowed'
        }

        It 'Runs the plug-in that provides the command by its type name without a warning' {
            $out = dsc Test.Precedence/Zeta 2> $TestDrive/error.txt | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out.env.DSC_PLUGIN_DIR | Should -BeExactly $firstDir
            Get-Content -Raw $TestDrive/error.txt | Should -Not -Match 'shadowed'
        }

        It 'Compares the type name case-insensitively' {
            $out = dsc test.precedence/ALPHA 2> $TestDrive/error.txt | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out.env.DSC_PLUGIN_DIR | Should -BeExactly $secondDir
        }

        It 'Shows the help of a shadowed plug-in by its type name' {
            $out = dsc help Test.Precedence/Alpha 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out[0] | Should -BeExactly 'Usage: dsc Test.Precedence/Alpha [OPTIONS]'
            $out[1] | Should -BeExactly 'HELP_ARGS: --help'
        }

        It 'Reports an unrecognized subcommand for a type name that no plug-in has' {
            $null = dsc Test.Precedence/DoesNotExist 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 2
            Get-Content -Raw $TestDrive/error.txt | Should -Match "unrecognized subcommand 'Test\.Precedence/DoesNotExist'"
        }

        It 'Lists a shadowed command once in the top-level help' {
            $out = dsc --help 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            @($out | Where-Object { $_ -match '^  shared\s' }).Count | Should -Be 1
        }
    }

    Context 'Disabling plug-ins' {
        BeforeAll {
            # A settings file in the working directory is the workspace settings file.
            $script:workspace = New-Item -ItemType Directory -Force -Path (Join-Path $TestDrive 'workspace')
            @{ plugins = @{ enabled = $false } } | ConvertTo-Json | Set-Content -Path (Join-Path $workspace 'dsc.settings.json')
            Push-Location $workspace
        }

        AfterAll {
            Pop-Location
        }

        It 'Lists no plug-ins' {
            $out = dsc plugin list 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out | Should -BeNullOrEmpty
        }

        It 'Reports a plug-in command as an unrecognized subcommand' {
            $out = dsc echo 2> $TestDrive/error.txt
            $LASTEXITCODE | Should -Be 2
            $out | Should -BeNullOrEmpty
            $stderr = Get-Content -Raw $TestDrive/error.txt
            $stderr | Should -Match "error: unrecognized subcommand 'echo'"
            $stderr | Should -Not -Match 'HELP_ARGS'
        }

        It 'Does not list plug-in commands in the top-level help' {
            $out = dsc --help 2> $TestDrive/error.txt | Out-String
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out | Should -Not -Match 'Plug-in commands:'
            $out | Should -Match '(?m)^Commands:\r?$'
        }

        It 'Does not include plug-in commands in the completion script' {
            $out = dsc completer powershell 2> $TestDrive/error.txt | Out-String
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out | Should -Not -Match "'echo'"
        }

        It 'Runs plug-ins again when the settings files are ignored' {
            $out = dsc --ignore-settings-file echo 2> $TestDrive/error.txt | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.txt)
            $out.env.DSC_PLUGIN_COMMAND | Should -BeExactly 'echo'
        }
    }
}
