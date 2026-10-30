# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

BeforeDiscovery {
    $script:isElevated = if ($IsWindows) {
        ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
            [Security.Principal.WindowsBuiltInRole]::Administrator)
    } else {
        (id -u) -eq 0
    }
    $script:skipSystem = -not $IsWindows -or -not $script:isElevated
}

BeforeAll {
    $script:isElevated = if ($IsWindows) {
        ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
            [Security.Principal.WindowsBuiltInRole]::Administrator)
    } else {
        (id -u) -eq 0
    }
    $script:skipSystem = -not $IsWindows -or -not $script:isElevated
    Write-Verbose -Message "IsElevated: $script:isElevated"
    Write-Verbose -Message "SkipSystem: $script:skipSystem"
    $script:dscHome = (Get-Command dsc).Path | Split-Path
    Write-Verbose -Message "DSC Home: $script:dscHome"
    $script:dscBuiltInResources = dsc resource list | ConvertFrom-Json -Depth 20

    function Get-SettingsFolderPath {
        <#
            .SYNOPSIS
            Gets the folder path for the specified DSC scope.
        #>
        [CmdletBinding()]
        [OutputType([string])]
        param(
            [Parameter(Mandatory)]
            [ValidateSet("Machine", "User", "Workspace", "Policy")]
            [string]$Scope
        )

        process {
            switch ($Scope) {
                "User" {
                    if (-not [string]::IsNullOrEmpty($env:XDG_CONFIG_HOME)) {
                        Join-Path -Path $env:XDG_CONFIG_HOME -ChildPath "dsc"
                    } elseif ($IsWindows) {
                        Join-Path -Path $env:APPDATA -ChildPath "dsc"
                    } elseif ($IsMacOS) {
                        "$HOME/Library/Application Support/dsc"
                    } else {
                        "$HOME/.config/dsc"
                    }
                }
                "Workspace" {
                    Get-Location | Select-Object -ExpandProperty Path
                }
                default {
                    if ($IsWindows) {
                        Join-Path -Path $env:ProgramData -ChildPath "dsc"
                    } elseif ($IsMacOS) {
                        "/Library/Application Support/dsc"
                    } else {
                        "/etc/dsc"
                    }
                }
            }
        }
    }

    function Get-SettingsFilePath {
        <#
            .SYNOPSIS
            Gets the full path for the settings file in the specified scope.
        #>
        [CmdletBinding()]
        param(
            [Parameter(Mandatory)]
            [ValidateSet("Machine", "User", "Workspace", "Policy")]
            [string]$Scope
        )

        process {
            $folder = Get-SettingsFolderPath @PSBoundParameters
            if ($Scope -eq 'Policy') {
                Join-Path -Path $folder -ChildPath 'dsc.policy.json'
            } else {
                Join-Path -Path $folder -ChildPath 'dsc.settings.json'
            }
        }
    }

    function Test-SettingsFilePath {
        <#
            .SYNOPSIS
            Tests whether the settings file exists in the specified scope.
        #>
        [CmdletBinding()]
        param(
            [Parameter(Mandatory)]
            [ValidateSet("Machine", "User", "Workspace", "Policy")]
            [string]$Scope
        )

        process {
            Test-Path -Path (Get-SettingsFilePath @PSBoundParameters)
        }
    }

    function ConvertTo-OrderedDictionary {
        <#
            .SYNOPSIS
            Converts output for `ConvertFrom-Json` into an OrderedDictionary.
            .DESCRIPTION
            This function is for converting output from `ConvertFrom-Json` into
            an ordered dictionary. It recursively converts PSCustomObjects or
            Hashtables into an OrderedDictionary, preserving the original
            property order from JSON.
        #>
        [CmdletBinding()]
        param(
            [Parameter(ValueFromPipeline, Mandatory)]
            $InputObject
        )

        begin {
            function Test-ShouldRecurse {
                param($InputObject)

                $InputObject -is [PSCustomObject] -or
                $InputObject -is [System.Collections.IDictionary]
            }
            function Test-IsCollection {
                param($InputObject)

                $InputObject -is [System.Collections.IEnumerable] -and
                -not ($InputObject -is [string])
            }
        }

        process {
            # Create an ordered dictionary
            $dict = [System.Collections.Specialized.OrderedDictionary]::new()

            # Get keys to iterate over
            $keys = @()
            if ($InputObject -is [System.Collections.IDictionary]) {
                $keys = $InputObject.Keys
            } elseif ($InputObject -is [PSCustomObject]) {
                $keys = $InputObject.PSObject.Properties.Name
            } else {
                return $InputObject  # Return non-object types unchanged
            }

            foreach ($key in $keys) {
                $value = if ($InputObject -is [System.Collections.IDictionary]) {
                    $InputObject[$key]
                } elseif ($InputObject -is [PSCustomObject]) {
                    $InputObject.$key
                }

                # Recursively convert nested objects
                if (Test-ShouldRecurse $value) {
                    $dict[$key] = ConvertTo-OrderedDictionary $value
                } elseif (Test-IsCollection $value) {
                    $dict[$key] = @(
                        foreach ($item in $value) {
                            if (Test-ShouldRecurse $item) {
                                ConvertTo-OrderedDictionary $item
                            } else {
                                $item
                            }
                        }
                    )
                } else {
                    $dict[$key] = $value
                }
            }

            $dict
        }
    }

    function Get-SettingsFile {
        <#
            .SYNOPSIS
            Retrieves the settings from the specified scope settings file.
        #>
        [CmdletBinding()]
        [OutputType([System.Collections.Specialized.OrderedDictionary])]
        param(
            [Parameter(Mandatory)]
            [ValidateSet("Machine", "User", "Workspace", "Policy")]
            [string]$Scope
        )

        process {
            $filePath = Get-SettingsFilePath @PSBoundParameters
            if (Test-Path -Path $filePath) {
                Get-Content -Path $filePath -Raw
                | ConvertFrom-Json -Depth 10
                | ConvertTo-OrderedDictionary
            }
        }
    }

    function Remove-SettingsFile {
        <#
            .SYNOPSIS
            Removes the settings file for the specified scope.
        #>
        [CmdletBinding()]
        param(
            [Parameter(Mandatory)]
            [ValidateSet("Machine", "User", "Workspace", "Policy")]
            [string]$Scope
        )

        process {
            $filePath = Get-SettingsFilePath @PSBoundParameters
            if (Test-Path -Path $filePath) {
                Remove-Item -Path $filePath -Force
            }
        }
    }

    function Set-SettingsFile {
        <#
            .SYNOPSIS
            Ensures the settings file for a scope exists with the defined
            settings.

            .DESCRIPTION
            This function ensures that the settings file for the specified scope
            exists and contains the provided settings. If the file doesn't
            exist, the function creates it. It overwrites any previous content.

            .PARAMETER Scope
            Specify the scope to define a settings file for. Valid values are:

            - `Machine` - The machine scope settings preference file.
            - `User` - The user scope settings preference file.
            - `Workspace` - The workspace scope settings preference file.
            - `Policy` - The system-wide policy settings file.

            .PARAMETER Settings
            An ordered dictionary containing the settings to write to the file.
        #>
        [CmdletBinding()]
        param(
            [Parameter(Mandatory)]
            [ValidateSet("Machine", "User", "Workspace", "Policy")]
            [string]$Scope,
            [Parameter(Mandatory)]
            [System.Collections.Specialized.OrderedDictionary]$Settings
        )

        process {
            $folder = Get-SettingsFolderPath -Scope $Scope
            if (-not (Test-Path -Path $folder)) {
                $null = New-Item -Path $folder -ItemType Directory -Force
            }

            $params = @{
                Path  = Get-SettingsFilePath -Scope $Scope
                Value = $Settings | ConvertTo-Json -Depth 10
                Force = $true
            }
            Set-Content @params
        }
    }

    function Initialize-SettingsFileContext {
        <#
            .SYNOPSIS
            Ensures no settings files exist for any scope and XDG_CONFIG_HOME
            isn't set.

            .DESCRIPTION
            This function checks for existing settings files in all file-source
            scopes (Machine, User, Workspace, Policy) and removes them if they
            exist after caching their contents.

            When processing the User scope, the function checks whether the
            `XDG_CONFIG_HOME` environment variable is set. If it is, the
            function caches the value before unsetting the variable.
        #>
        [CmdletBinding()]
        param (
        )

        begin {
            #region    Machine   scope
            if (Test-SettingsFilePath -Scope 'Machine') {
                $filePath = Get-SettingsFilePath -Scope 'Machine'
                $script:MachineSettingsContent = Get-Content -Path $filePath -Raw
                Remove-Item -Path $filePath -Force
            }
            #endregion Machine   scope
            #region    User      scope
            if (Test-SettingsFilePath -Scope 'User') {
                $filePath = Get-SettingsFilePath -Scope 'User'
                if (-not [string]::IsNullOrEmpty($env:XDG_CONFIG_HOME)) {
                    $script:XDG_CONFIG_HOME = $env:XDG_CONFIG_HOME
                    Remove-Item Env:\XDG_CONFIG_HOME
                } else {
                    $script:UserSettingsContent = Get-Content -Path $filePath -Raw
                    Remove-Item -Path $filePath -Force
                    $parentFolder = Split-Path -Path $filePath -Parent
                    if (-not (Test-Path $parentFolder)) {
                        $script:RemoveUserSettingsParentFolder = $true
                        New-Item -Path $parentFolder -ItemType Directory -Force
                    }
                }
            }
            #endregion User      scope
            #region    Workspace scope
            if (Test-SettingsFilePath -Scope 'Workspace') {
                $filePath = Get-SettingsFilePath -Scope 'Workspace'
                $script:WorkspaceSettingsContent = Get-Content -Path $filePath -Raw
                Remove-Item -Path $filePath -Force
            }
            #endregion Workspace scope
            #region    Policy    scope
            if (-not $script:skipSystem -and (Test-SettingsFilePath -Scope 'Policy')) {
                $filePath = Get-SettingsFilePath -Scope 'Policy'
                $script:PolicySettingsContent = Get-Content -Path $filePath -Raw
                Remove-Item -Path $filePath -Force
            }
            #endregion Policy    scope
        }
    }

    function Reset-SettingsFileContext {
        <#
            .SYNOPSIS
            Removes the settings files for every scope.

            .DESCRIPTION
            This function resets the settings file context by removing the
            settings files for all scopes. This should be used between tests.
        #>
        [CmdletBinding()]
        param()

        process {
            Remove-SettingsFile -Scope 'Machine'
            Remove-SettingsFile -Scope 'User'
            Remove-SettingsFile -Scope 'Workspace'
            if (-not $script:skipSystem) {
                Remove-SettingsFile -Scope 'Policy'
            }
        }
    }

    function Restore-SettingsFileContext {
        <#
            .SYNOPSIS
            Restores the settings file context to before testing.

            .DESCRIPTION
            This function restores the settings file context by recreating the
            settings files for all scopes using the cached data from the
            `Initialize-SettingsFileContext` function.

            It restores any settings files and content and returns the ACL for
            the system settings folder to its original state if it existed.
        #>
        [CmdletBinding()]
        param()

        process {
            #region    Machine   scope
            if (-not [string]::IsNullOrEmpty($script:MachineSettingsContent)) {
                $filePath = Get-SettingsFilePath -Scope 'Machine'
                Set-Content -Path $filePath -Value $script:MachineSettingsContent
            } else {
                Remove-SettingsFile -Scope 'Machine'
            }
            #endregion Machine   scope
            #region    User      scope
            if (-not [string]::IsNullOrEmpty($script:XDG_CONFIG_HOME)) {
                $env:XDG_CONFIG_HOME = $script:XDG_CONFIG_HOME
            } elseif (-not [string]::IsNullOrEmpty($script:UserSettingsContent)) {
                $filePath = Get-SettingsFilePath -Scope 'User'
                Set-Content -Path $filePath -Value $script:UserSettingsContent
            } else {
                Remove-SettingsFile -Scope 'User'
            }
            #endregion User      scope
            #region    Workspace scope
            if (-not [string]::IsNullOrEmpty($script:WorkspaceSettingsContent)) {
                $filePath = Get-SettingsFilePath -Scope 'Workspace'
                Set-Content -Path $filePath -Value $script:WorkspaceSettingsContent
            } else {
                Remove-SettingsFile -Scope 'Workspace'
            }
            #endregion Workspace scope
            #region    Policy    scope
            if (-not $script:skipSystem) {
                if (-not [string]::IsNullOrEmpty($script:PolicySettingsContent)) {
                    $filePath = Get-SettingsFilePath -Scope 'Policy'
                    Set-Content -Path $filePath -Value $script:PolicySettingsContent
                } else {
                    Remove-SettingsFile -Scope 'Policy'
                }
            }
            #endregion Policy    scope
        }
    }

    function Initialize-PolicyFolder {
        <#
            .SYNOPSIS
            Initializes the policy folder so it exists with secure ACL settings.

            .DESCRIPTION
            This function checks whether the policy folder exists.

            If the policy folder exists, the function caches the original ACL
            so it can be restored later with the `Reset-PolicyFolder` function.
            If the folder doesn't exist, the function creates it.

            In either case, the function then sets the ACL for the folder to
            the expected secure state.
        #>
        [CmdletBinding()]
        param()

        process {
            if ($script:skipSystem) {
                return
            }

            $folder = Get-SettingsFolderPath -Scope 'Policy'
            $script:existedBefore = Test-Path -Path $folder

            if ($script:existedBefore) {
                $script:originalAcl = Get-Acl -Path $folder
            } else {
                $null = New-Item -ItemType Directory -Path $folder -Force
            }

            $null = Set-PolicyFolderAcl -Secure
        }
    }

    function Restore-PolicyFolder {
        <#
            .SYNOPSIS
            Resets the policy folder to its original state.

            .DESCRIPTION
            This function restores the original ACL for the policy folder if it
            existed before initialization. If `Initialize-PolicyFolder` created
            the folder, this function removes it.
        #>
        [CmdletBinding()]
        param()

        process {
            if ($script:skipSystem) {
                return
            }
            $folder = Get-SettingsFolderPath -Scope 'Policy'
            if ($script:existedBefore) {
                # Restore the original ACL for the policy folder
                Set-Acl -Path $folder -AclObject $script:originalAcl
            } else {
                Remove-Item -Path $folder -Recurse -Force
            }
        }
    }

    function Get-PolicyFolderAcl {
        [CmdletBinding()]
        [OutputType([System.Security.AccessControl.DirectorySecurity])]
        param()

        process {
            $folder = Get-SettingsFolderPath -Scope 'Policy'
            Get-Acl -Path $folder
        }
    }

    function Set-PolicyFolderAcl {
        [CmdletBinding(DefaultParameterSetName='Default')]
        param(
            [Parameter(Mandatory, ParameterSetName='Secure')]
            [switch]$Secure,
            [Parameter(Mandatory, ParameterSetName='Insecure')]
            [switch]$Insecure,
            [Parameter(Mandatory, ParameterSetName='AccessRule')]
            [System.Security.AccessControl.FileSystemAccessRule[]]$AccessRule
        )

        begin {
            $folder = Get-SettingsFolderPath -Scope 'Policy'
            $acl = Get-Acl -Path $folder
            $acl.SetAccessRuleProtection($true, $false)
            $acl.Access | ForEach-Object { $acl.RemoveAccessRule($_) } | Out-Null
        }

        process {
            if ($script:skipSystem) {
                return
            }

            if ($Secure) {
                $systemRule = New-Object System.Security.AccessControl.FileSystemAccessRule(
                    "NT AUTHORITY\SYSTEM", "FullControl", "ContainerInherit,ObjectInherit", "None", "Allow"
                )
                $adminsRule = New-Object System.Security.AccessControl.FileSystemAccessRule(
                    "BUILTIN\Administrators", "FullControl", "ContainerInherit,ObjectInherit", "None", "Allow"
                )
                $usersReadRule = New-Object System.Security.AccessControl.FileSystemAccessRule(
                    "BUILTIN\Users", "ReadAndExecute", "ContainerInherit,ObjectInherit", "None", "Allow"
                )
                $acl.AddAccessRule($systemRule)
                $acl.AddAccessRule($adminsRule)
                $acl.AddAccessRule($usersReadRule)
            } elseif ($Insecure) {
                # todo
            } elseif ($AccessRule.Count -gt 0) {
                $AccessRule | ForEach-Object { $acl.AddAccessRule($_) }
            }

            Set-Acl -Path $folder -AclObject $acl -Passthru
        }
    }

    function Initialize-SettingsEnvironmentContext {
        [CmdletBinding()]
        param()

        process {
            if (-not [string]::IsNullOrEmpty($env:DSC_TRACE_LEVEL)) {
                $script:dscTraceLevel = $env:DSC_TRACE_LEVEL
                Remove-Item env:DSC_TRACE_LEVEL
            }
            if (-not [string]::IsNullOrEmpty($env:DSC_TRACE_FORMAT)) {
                $script:dscTraceFormat = $env:DSC_TRACE_FORMAT
                Remove-Item env:DSC_TRACE_FORMAT
            }
            if (-not [string]::IsNullOrEmpty($env:DSC_RESOURCE_PATH)) {
                $script:dscResourcePath = $env:DSC_RESOURCE_PATH
                Remove-Item env:DSC_RESOURCE_PATH
            }
            if (-not [string]::IsNullOrEmpty($env:DSC_RESTRICTED_PATH)) {
                $script:dscRestrictedPath = $env:DSC_RESTRICTED_PATH
                Remove-Item env:DSC_RESTRICTED_PATH
            }
            if (-not [string]::IsNullOrEmpty($env:DSC_IGNORE_SETTING_FILE)) {
                $script:dscIgnoreSettingFile = $env:DSC_IGNORE_SETTING_FILE
                Remove-Item env:DSC_IGNORE_SETTING_FILE
            }
        }
    }

    function Restore-SettingsEnvironmentContext {
        [CmdletBinding()]
        param()

        process {
            if (-not [string]::IsNullOrEmpty($script:dscTraceLevel)) {
                $env:DSC_TRACE_LEVEL = $script:dscTraceLevel
            }
            if (-not [string]::IsNullOrEmpty($script:dscTraceFormat)) {
                $env:DSC_TRACE_FORMAT = $script:dscTraceFormat
            }
            if (-not [string]::IsNullOrEmpty($script:dscResourcePath)) {
                $env:DSC_RESOURCE_PATH = $script:dscResourcePath
            }
            if (-not [string]::IsNullOrEmpty($script:dscRestrictedPath)) {
                $env:DSC_RESTRICTED_PATH = $script:dscRestrictedPath
            }
            if (-not [string]::IsNullOrEmpty($script:dscIgnoreSettingFile)) {
                $env:DSC_IGNORE_SETTING_FILE = $script:dscIgnoreSettingFile
            }
        }
    }

    function Reset-SettingsEnvironment {
        [CmdletBinding()]
        param()

        process {
            if (-not [string]::IsNullOrEmpty($env:DSC_TRACE_LEVEL)) {
                $env:DSC_TRACE_LEVEL = $null
            }
            if (-not [string]::IsNullOrEmpty($env:DSC_TRACE_FORMAT)) {
                $env:DSC_TRACE_FORMAT = $null
            }
            if (-not [string]::IsNullOrEmpty($env:DSC_RESOURCE_PATH)) {
                $env:DSC_RESOURCE_PATH = $null
            }
            if (-not [string]::IsNullOrEmpty($env:DSC_RESTRICTED_PATH)) {
                $env:DSC_RESTRICTED_PATH = $null
            }
            if (-not [string]::IsNullOrEmpty($env:DSC_IGNORE_SETTINGS_FILE)) {
                $env:DSC_IGNORE_SETTINGS_FILE = $null
            }
        }
    }

    function Get-SettingsEnvironment {
        [CmdletBinding()]
        param()

        begin {
            $settings = [ordered]@{}
        }

        process {
            if (-not [string]::IsNullOrEmpty($env:DSC_TRACE_LEVEL)) {
                $settings['DSC_TRACE_LEVEL'] = $env:DSC_TRACE_LEVEL
            }
            if (-not [string]::IsNullOrEmpty($env:DSC_TRACE_FORMAT)) {
                $settings['DSC_TRACE_FORMAT'] = $env:DSC_TRACE_FORMAT
            }
            if (-not [string]::IsNullOrEmpty($env:DSC_RESOURCE_PATH)) {
                $settings['DSC_RESOURCE_PATH'] = $env:DSC_RESOURCE_PATH
            }
            if (-not [string]::IsNullOrEmpty($env:DSC_RESTRICTED_PATH)) {
                $settings['DSC_RESTRICTED_PATH'] = $env:DSC_RESTRICTED_PATH
            }
            if (-not [string]::IsNullOrEmpty($env:DSC_IGNORE_SETTING_FILE)) {
                $settings['DSC_IGNORE_SETTING_FILE'] = $env:DSC_IGNORE_SETTING_FILE
            }
        }

        end {
            $settings
        }
    }

    function Set-SettingsEnvironment {
        [CmdletBinding(DefaultParameterSetName='SeparateSettings')]
        param(
            [Parameter(ParameterSetName='SeparateSettings')]
            [ValidateNotNullOrEmpty()] [string] $TraceLevel,
            [Parameter(ParameterSetName='SeparateSettings')]
            [ValidateNotNullOrEmpty()] [string] $TraceFormat,
            [Parameter(ParameterSetName='SeparateSettings')]
            [ValidateNotNullOrEmpty()] [string] $ResourcePath,
            [Parameter(ParameterSetName='SeparateSettings')]
            [ValidateNotNullOrEmpty()] [string] $RestrictedPath,
            [Parameter(ParameterSetName='SeparateSettings')]
            [ValidateNotNullOrEmpty()] [string] $IgnoreSettingsFile,

            [Parameter(ParameterSetName="SettingsDictionary")]
            [System.Collections.Specialized.OrderedDictionary] $Settings
        )
        begin {
            if ($null -ne $Settings) {
                if ($Settings.Contains('DSC_TRACE_LEVEL')) {
                    $TraceLevel = $Settings['DSC_TRACE_LEVEL']
                }
                if ($Settings.Contains('DSC_TRACE_FORMAT')) {
                    $TraceFormat = $Settings['DSC_TRACE_FORMAT']
                }
                if ($Settings.Contains('DSC_RESOURCE_PATH')) {
                    $ResourcePath = $Settings['DSC_RESOURCE_PATH']
                }
                if ($Settings.Contains('DSC_RESTRICTED_PATH')) {
                    $RestrictedPath = $Settings['DSC_RESTRICTED_PATH']
                }
                if ($Settings.Contains('DSC_IGNORE_SETTINGS_FILE')) {
                    $IgnoreSettingsFile = $Settings['DSC_IGNORE_SETTINGS_FILE']
                }
            }
        }

        process {
            if ($TraceLevel) {
                $env:DSC_TRACE_LEVEL = $TraceLevel
            }
            if ($TraceFormat) {
                $env:DSC_TRACE_FORMAT = $TraceFormat
            }
            if ($ResourcePath) {
                $env:DSC_RESOURCE_PATH = $ResourcePath
            }
            if ($RestrictedPath) {
                $env:DSC_RESTRICTED_PATH = $RestrictedPath
            }
            if ($IgnoreSettingsFile) {
                $env:DSC_IGNORE_SETTINGS_FILE = $IgnoreSettingsFile
            }
        }
    }

    function Initialize-SettingsContext {
        [CmdletBinding()]
        param()

        process {
            Initialize-PolicyFolder
            Initialize-SettingsFileContext
            Initialize-SettingsEnvironmentContext
        }
    }

    function Restore-SettingsContext {
        [CmdletBinding()]
        param()

        process {
            Restore-SettingsFileContext
            Restore-PolicyFolder
            Restore-SettingsEnvironmentContext
        }
    }

    function Get-SettingsContext {
        [CmdletBinding()]
        param()

        process {
            [pscustomobject]@{
                SettingsFiles = [pscustomobject]@{
                    Machine = [pscustomobject]@{
                        FilePath = Get-SettingsFilePath -Scope Machine
                        Exists   = Test-SettingsFilePath -Scope Machine
                        Data     = Get-SettingsFile -Scope Machine
                    }
                    User = [pscustomobject]@{
                        FilePath = Get-SettingsFilePath -Scope User
                        Exists   = Test-SettingsFilePath -Scope User
                        Data     = Get-SettingsFile -Scope User
                    }
                    Workspace = [pscustomobject]@{
                        FilePath = Get-SettingsFilePath -Scope Workspace
                        Exists   = Test-SettingsFilePath -Scope Workspace
                        Data     = Get-SettingsFile -Scope Workspace
                    }
                    Policy = [pscustomobject]@{
                        FilePath = Get-SettingsFilePath -Scope Policy
                        Exists   = Test-SettingsFilePath -Scope Policy
                        Data     = Get-SettingsFile -Scope Policy
                    }
                }
                Environment = Get-SettingsEnvironment
            }
        }
    }

    class TestContext {
        [string]$Message
        [object]$Result
        [int]$ExitCode
        [pscustomobject]$SettingsContext
        [string[]]$Traces

        [System.Collections.Specialized.OrderedDictionary] ToData() {
            $data = [ordered]@{}
            if ($null -ne $this.Message) {
                $data.message = $this.Message
            }
            if ($null -ne $this.Result) {
                $data.result = $this.Result
            }
            $data.exitCode = $this.ExitCode
            $data.SettingsContext = $this.SettingsContext
            $data.Traces = $this.Traces
            return $data
        }

        [string] ToString() {
            return $this.ToData() | ConvertTo-Json -Depth 10
        }
    }

    function New-TestContext {
        [CmdletBinding()]
        [OutputType('TestContext')]
        param(
            [string]$Message,
            [object]$Result,
            [string]$TraceFilePath
        )

        begin {
            $context = [TestContext]@{
                ExitCode = $LASTEXITCODE
            }
        }

        process {
            if (-not [string]::IsNullOrEmpty($Message)) {
                $context.Message = $Message
            }
            if ($null -ne $Result) {
                $context.Result = $Result
            }
            $context.ExitCode = $LASTEXITCODE
            $context.SettingsContext = Get-SettingsContext
            $context.Traces  = Get-Content -Path $TraceFilePath

            $context
        }
    }
}

Describe 'dsc settings tests' {
    BeforeAll {
        Write-Verbose -Message "Initial settings context: $(Get-SettingsContext | ConvertTo-Json -Depth 10)"
        Initialize-SettingsContext
        Write-Verbose -Message "Settings context after initialization: $(Get-SettingsContext | ConvertTo-Json -Depth 10)"
    }
    AfterAll {
        Write-Verbose -Message "Restoring settings context..."
        Restore-SettingsContext
        Write-Verbose -Message "Settings context after restoration: $(Get-SettingsContext | ConvertTo-Json -Depth 10)"
    }
    AfterEach {
        Reset-SettingsEnvironment
        Reset-SettingsFileContext
    }

    Context 'cli settings' {
        Context '--trace-level' {
            it "emits expected messaages for '<_>' trace level" -ForEach @("error", "warn", "info", "debug", "trace") {
                $level = $_
                $messages = [pscustomobject]@{
                    Trace   = "trace resource message for level: $level"
                    Debug   = "debug message for level: $level"
                    Info    = "info message for level: $level"
                    Warning = "warning message for level: $level"
                    Error   = "error message for level: $level"
                }
                $params = @(
                    '--trace-level', $level
                    'resource', 'get'
                    '--resource', 'Microsoft.DSC.Transitional/PowerShellScript'
                    '--input', ([ordered]@{
                        getScript = @(
                            "Write-Information '$($messages.Trace)'"
                            "Write-Debug       '$($messages.Debug)'"
                            "Write-Verbose     '$($messages.Info)'"
                            "Write-Warning     '$($messages.Warning)'"
                            "Write-Error       '$($messages.Error)'"
                        ) -join "`n"
                    } | ConvertTo-Json)
                )
                dsc @params 2> $TestDrive/tracing.txt
                $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt

                # All levels should find the error message
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Error -Because $testContext
                if ($level -ne "error") {
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Warning -Because $testContext
                }
                if ($level -notin @('error', 'warn')) {
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Info -Because $testContext
                }
                if ($level -notin ('error', 'warn', 'info')) {
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Debug -Because $testContext
                }
                if ($level -eq 'trace') {
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Trace -Because $testContext
                }
            }
            It "errors for an invalid value" {
                $value = 'invalid'
                $expected = [regex]::escape("error: invalid value '$value' for '--trace-level <TRACE_LEVEL>'")
                dsc --trace-level $value resource list *echo* 2> $TestDrive/tracing.txt
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $expected -Because (
                    New-TestContext -TraceFilePath $TestDrive/tracing.txt
                )
            }
        }
        Context '--trace-format' {
            It "emits expected format for '<_>' trace format" -ForEach @('default', 'plaintext', 'json') {
                $format = $_
                $invalidKey = "trace_format.$_"
                $params = @(
                    '--trace-format', $format
                    'resource', 'get'
                    '--resource', 'Microsoft.DSC.Debug/Echo'
                    '--input', (@{ $invalidKey = $format} | ConvertTo-Json -Compress)
                )
                dsc @params 2> $TestDrive/tracing.txt
                $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt

                $expectedMessage = "Schema: Additional properties are not allowed ('$invalidKey' was unexpected)"
                if ($format -eq 'json') {
                    $traces = Get-Content $TestDrive/tracing.txt | ConvertFrom-Json
                    $traces.count | Should -Be 1 -Because $testContext
                    $traces[0].level | Should -Be 'Error' -Because $testContext
                    $traces[0].fields.message | Should -Be $expectedMessage -Because $testContext
                } else {
                    $expected = [regex]::Escape("ERROR $expectedMessage")
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $expected -Because $testContext
                }
            }

            It "errors for an invalid value" {
                $value = 'invalid'
                $expected = [regex]::escape("error: invalid value '$value' for '--trace-format <TRACE_FORMAT>'")
                dsc --trace-format $value resource list *echo* 2> $TestDrive/tracing.txt
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $expected -Because (
                    New-TestContext -TraceFilePath $TestDrive/tracing.txt
                )
            }
        }
        Context '--ignore-settings-file' {
            BeforeAll {
                $userPreferenceData = [ordered]@{
                    tracing = [ordered]@{
                        level = 'info'
                    }
                }
            }
            BeforeEach {
                Set-SettingsFile -Scope User -Settings $userPreferenceData
            }
            It "loads settings files when the flag is not specified" {
                dsc resource list Microsoft.DSC.Debug/Echo 2> $TestDrive/tracing.txt
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Trace-level is Info' -Because (
                    New-TestContext -Message 'expected user scope tracing.level as info to emit trace message' -TraceFilePath $TestDrive/tracing.txt
                )
            }
            It "loads settings files when the flag is specified as false" {
                dsc --ignore-settings-file=false resource list Microsoft.DSC.Debug/Echo 2> $TestDrive/tracing.txt
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Trace-level is Info' -Because (
                    New-TestContext -Message 'expected user scope tracing.level as info to emit trace message' -TraceFilePath $TestDrive/tracing.txt
                )
            }
            It "does not load settings files when the flag is specified without a value" {
                dsc --ignore-settings-file resource list Microsoft.DSC.Debug/Echo 2> $TestDrive/tracing.txt
                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly 'Trace-level is Info' -Because (
                    New-TestContext -Message 'expected user scope tracing.level as info to be ignored' -TraceFilePath $TestDrive/tracing.txt
                )
            }
            It "does not load settings files when the flag is specified as true" {
                dsc --ignore-settings-file=true resource list Microsoft.DSC.Debug/Echo 2> $TestDrive/tracing.txt
                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly 'Trace-level is Info' -Because (
                    New-TestContext -Message 'expected user scope tracing.level as info to be ignored' -TraceFilePath $TestDrive/tracing.txt
                )
            }
        }
    }

    Context 'environment settings' {
        Context 'DSC_TRACE_LEVEL' {
            it "emits expected messaages for '<_>' trace level" -ForEach @("error", "warn", "info", "debug", "trace") {
                $level = $_
                Set-SettingsEnvironment -TraceLevel $level
                $messages = [pscustomobject]@{
                    Trace   = "trace resource message for level: $level"
                    Debug   = "debug message for level: $level"
                    Info    = "info message for level: $level"
                    Warning = "warning message for level: $level"
                    Error   = "error message for level: $level"
                }
                $params = @(
                    'resource', 'get'
                    '--resource', 'Microsoft.DSC.Transitional/PowerShellScript'
                    '--input', ([ordered]@{
                        getScript = @(
                            "Write-Information '$($messages.Trace)'"
                            "Write-Debug       '$($messages.Debug)'"
                            "Write-Verbose     '$($messages.Info)'"
                            "Write-Warning     '$($messages.Warning)'"
                            "Write-Error       '$($messages.Error)'"
                        ) -join "`n"
                    } | ConvertTo-Json)
                )
                dsc @params 2> $TestDrive/tracing.txt
                $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt

                # All levels should find the error message
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Error -Because $testContext
                if ($level -ne "error") {
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Warning -Because $testContext
                }
                if ($level -notin @('error', 'warn')) {
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Info -Because $testContext
                }
                if ($level -notin ('error', 'warn', 'info')) {
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Debug -Because $testContext
                }
                if ($level -eq 'trace') {
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Trace -Because $testContext
                }
            }
            It "raises a warning for an invalid value" {
                $value = 'invalid'
                $expected = "unable to case-insensitively parse '$value' as a valid DSC settings trace level"
                Set-SettingsEnvironment -TraceLevel $value
                dsc resource list *echo* 2> $TestDrive/tracing.txt
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $expected -Because (
                    New-TestContext -TraceFilePath $TestDrive/tracing.txt
                )
            }
        }
        Context 'DSC_TRACE_FORMAT' {
            It "emits expected format for '<_>' trace format" -ForEach @('default', 'plaintext', 'json') {
                $format = $_
                $invalidKey = "trace_format.$_"
                Set-SettingsEnvironment -TraceFormat $format -TraceLevel Error
                $params = @(
                    'resource', 'get'
                    '--resource', 'Microsoft.DSC.Debug/Echo'
                    '--input', (@{ $invalidKey = $format} | ConvertTo-Json -Compress)
                )
                dsc @params 2> $TestDrive/tracing.txt
                $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt

                $expectedMessage = "Schema: Additional properties are not allowed ('$invalidKey' was unexpected)"
                if ($format -eq 'json') {
                    $traces = Get-Content $TestDrive/tracing.txt | ConvertFrom-Json
                    $traces.count | Should -Be 1 -Because $testContext
                    $traces[0].level | Should -Be 'Error' -Because $testContext
                    $traces[0].fields.message | Should -Be $expectedMessage -Because $testContext
                } else {
                    $expected = [regex]::Escape("ERROR $expectedMessage")
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $expected -Because $testContext
                }
            }
            It "raises a warning for an invalid value" {
                $value = 'invalid'
                $expected = "unable to case-insensitively parse '$value' as a valid DSC settings trace format"
                Set-SettingsEnvironment -TraceFormat $value
                dsc resource list *echo* 2> $TestDrive/tracing.txt
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $expected -Because (
                    New-TestContext -TraceFilePath $TestDrive/tracing.txt
                )
            }
        }
        Context 'DSC_RESOURCE_PATH' {
            BeforeAll {
                $resourcePath = "$TestDrive/resources"
                $resourceManifest = @{
                    '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json'
                    type      = 'DSC.Test/ResourcePath'
                    version   = '0.1.0'
                    get       = [ordered]@{
                        executable = 'dsc'
                        args       = @(@{
                            jsonInputArg = '--input'
                            mandatory    = $true
                        })
                    }
                    schema = @{
                        command = @{
                            executable = 'dscecho'
                        }
                    }
                }
                $resourceManifestJson = $resourceManifest | ConvertTo-Json -Depth 10
            }
            BeforeEach {
                New-Item -ItemType Directory -Path $resourcePath -Force
                $resourceManifestJson | Set-Content -Path (Join-Path $resourcePath "test.dsc.resource.json") -Force
            }
            AfterEach {
                Remove-Item -Path $resourcePath -Recurse -Force
            }

            It "adds discoverable resource paths" {
                Set-SettingsEnvironment -ResourcePath $resourcePath
                $result = dsc resource list $resourceManifest.type 2> $TestDrive/tracing.txt
                $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                $result | Should -Not -BeNullOrEmpty -Because $testContext
                $result.Count | Should -Be 1 -Because $testContext
                $data = $result | ConvertFrom-Json -Depth 10
                $data.type | Should -Be $resourceManifest.type -Because $testContext
            }
        }
        Context "DSC_RESTRICTED_PATH" {
            BeforeAll {
                $escapedDscHome = [Regex]::Escape($script:dscHome)
                $resourcePath = Join-Path -Path $TestDrive -ChildPath 'resources'
                $escapedResourcePath = [Regex]::Escape($resourcePath)
                $escapedResourcePathJson = $IsWindows ? [Regex]::Escape($escapedResourcePath) : $escapedResourcePath
                $resourceManifest = @{
                    '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json'
                    type      = 'DSC.Test/ResourcePath'
                    version   = '0.1.0'
                    get       = [ordered]@{
                        executable = 'dscecho'
                        args       = @(@{
                            jsonInputArg = '--input'
                            mandatory    = $true
                        })
                    }
                    schema = @{
                        command = @{
                            executable = 'dscecho'
                        }
                    }
                }
                $resourceManifestJson = $resourceManifest | ConvertTo-Json -Depth 10
            }
            BeforeEach {
                New-Item -ItemType Directory -Path $resourcePath -Force
                $resourceManifestJson | Set-Content -Path (Join-Path $resourcePath "test.dsc.resource.json") -Force
            }
            AfterEach {
                Remove-Item -Path $resourcePath -Recurse -Force
            }

            It "adds discoverable resource paths" {
                Set-SettingsEnvironment -RestrictedPath $resourcePath
                $result = dsc resource list $resourceManifest.type 2> $TestDrive/tracing.txt
                $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                $result | Should -Not -BeNullOrEmpty -Because $testContext
                $result.Count | Should -Be 1 -Because $testContext
                $data = $result | ConvertFrom-Json -Depth 10
                $data.type | Should -Be $resourceManifest.type -Because $testContext
            }
            It "restricts discovery for other paths" {
                Set-SettingsEnvironment -RestrictedPath $resourcePath
                $result = dsc resource list 2> $TestDrive/tracing.txt
                $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                $result.Count | Should -Be 1 -Because $testContext
                $data = $result | ConvertFrom-Json -Depth 10
                $data.type | Should -Be $resourceManifest.type -Because $testContext
            }
            It "raises a warning when DSC executable home is not in the restricted path" {
                Set-SettingsEnvironment -RestrictedPath $resourcePath
                dsc resource list $resourceManifest.type 2> $TestDrive/tracing.txt
                $expectedWarning = @(
                    'WARN cannot discover or invoke built-in resources or extensions'
                    "- DSC executable home '{0}' is not included" -f $escapedDscHome
                    'in the restricted resource path directories \(\["{0}"\]\)' -f $escapedResourcePathJson
                ) -join ' '
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $expectedWarning -Because (
                    New-TestContext -TraceFilePath $TestDrive/tracing.txt
                )
            }
            It "errors when invoking an executable outside of the specified paths" {
                Set-SettingsEnvironment -RestrictedPath $resourcePath
                $params = @(
                    'resource', 'get'
                    '--resource', $resourceManifest.type
                    '--input', '{"output": "test"}'
                )
                $result = dsc @params 2> $TestDrive/tracing.txt
                $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                $LASTEXITCODE | Should -Be 2 -Because $testContext
                $result | Should -BeNullOrEmpty -Because $testContext
                $expectedError = @(
                    "ERROR Command: Operation Executable '{0}' not found" -f $resourceManifest.get.executable
                    "with working directory '{0}'" -f $escapedResourcePath
                ) -join ' '
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $expectedError -Because $testContext
            }
        }
        Context 'DSC_IGNORE_SETTINGS_FILE' {
            BeforeAll {
                $userPreferenceData = [ordered]@{
                    tracing = [ordered]@{
                        level = 'info'
                    }
                }
            }
            BeforeEach {
                Set-SettingsFile -Scope User -Settings $userPreferenceData
            }

            It "loads the settings file when variable isn't set" {
                dsc resource list Microsoft.DSC.Debug/Echo 2> $TestDrive/tracing.txt
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Trace-level is Info' -Because (
                    New-TestContext -Message 'expected user scope tracing.level as info to emit trace message' -TraceFilePath $TestDrive/tracing.txt
                )
            }
            It "loads settings files when variable is '<_>'" -ForEach @('false', 'FALSE', 'fAlSe', '0') {
                Set-SettingsEnvironment -IgnoreSettingsFile $_
                dsc resource list Microsoft.DSC.Debug/Echo 2> $TestDrive/tracing.txt
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Trace-level is Info' -Because (
                    New-TestContext -Message 'expected user scope tracing.level as info to emit trace message' -TraceFilePath $TestDrive/tracing.txt
                )
            }
            It "skips loading settings files when variable is '<_>'" -ForEach @('true', 'TRUE', 'tRuE', '1') {
                Set-SettingsEnvironment -IgnoreSettingsFile $_
                dsc resource list Microsoft.DSC.Debug/Echo 2> $TestDrive/tracing.txt
                Get-Content -Raw -Path "$TestDrive/tracing.txt" | Should -BeNullOrEmpty -Because (
                    New-TestContext -Message 'expected no trace message when ignoring settings file' -TraceFilePath $TestDrive/tracing.txt
                )
            }
        }
    }

    Context "preference settings" {
        BeforeDiscovery {
            [string[]]$scopes = if ($script:skipSystem) {
                @('User', 'Workspace')
            } else {
                @('Machine', 'User', 'Workspace')
            }
        }

        Context '<_> scope' -ForEach $scopes {
            BeforeAll {
                $scope = $_
            }

            Context 'tracing.level' {
                It "emits expected messaages for '<_>' trace level" -ForEach @("error", "warn", "info", "debug", "trace") {
                    $level = $_
                    $data = [ordered]@{
                        tracing = [ordered]@{
                            level = $level
                        }
                    }
                    Set-SettingsFile -Scope $scope -Settings $data
                    $messages = [pscustomobject]@{
                        Trace   = "trace resource message for level: $level"
                        Debug   = "debug message for level: $level"
                        Info    = "info message for level: $level"
                        Warning = "warning message for level: $level"
                        Error   = "error message for level: $level"
                    }
                    $params = @(
                        'resource', 'get'
                        '--resource', 'Microsoft.DSC.Transitional/PowerShellScript'
                        '--input', ([ordered]@{
                            getScript = @(
                                "Write-Information '$($messages.Trace)'"
                                "Write-Debug       '$($messages.Debug)'"
                                "Write-Verbose     '$($messages.Info)'"
                                "Write-Warning     '$($messages.Warning)'"
                                "Write-Error       '$($messages.Error)'"
                            ) -join "`n"
                        } | ConvertTo-Json)
                    )
                    dsc @params 2> $TestDrive/tracing.txt
                    $testContext = New-TestContext  -TraceFilePath $TestDrive/tracing.txt

                    # All levels should find the error message
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Error -Because $testContext
                    if ($level -ne "error") {
                        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Warning -Because $testContext
                    }
                    if ($level -notin @('error', 'warn')) {
                        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Info -Because $testContext
                    }
                    if ($level -notin ('error', 'warn', 'info')) {
                        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Debug -Because $testContext
                    }
                    if ($level -eq 'trace') {
                        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Trace -Because $testContext
                    }
                }
                It "raises a warning for an invalid value" {
                    $value = 'invalid'
                    $expected = "unable to case-insensitively parse '$value' as a valid DSC settings trace level"
                    $data = [ordered]@{
                        tracing = [ordered]@{
                            level = $value
                        }
                    }
                    Set-SettingsFile -Scope $scope -Settings $data
                    dsc resource list *echo* 2> $TestDrive/tracing.txt
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $expected -Because {
                        New-TestContext -TraceFilePath $TestDrive/tracing.txt
                    }
                }
            }

            Context 'tracing.format' {
                It "emits expected format for '<_>' trace format" -ForEach @('default', 'plaintext', 'json') {
                    $format = $_
                    $invalidKey = "trace_format.$_"
                    $data = [ordered]@{
                        tracing = [ordered]@{
                            format = $format
                        }
                    }
                    Set-SettingsFile -Scope $scope -Settings $data
                    Set-SettingsEnvironment -TraceLevel Error
                    $params = @(
                        'resource', 'get'
                        '--resource', 'Microsoft.DSC.Debug/Echo'
                        '--input', (@{ $invalidKey = $format} | ConvertTo-Json -Compress)
                    )
                    dsc @params 2> $TestDrive/tracing.txt
                    $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt

                    $expectedMessage = "Schema: Additional properties are not allowed ('$invalidKey' was unexpected)"
                    if ($format -eq 'json') {
                        $traces = Get-Content $TestDrive/tracing.txt | ConvertFrom-Json
                        $traces.count | Should -Be 1 -Because $testContext
                        $traces[0].level | Should -Be 'Error' -Because $testContext
                        $traces[0].fields.message | Should -Be $expectedMessage -Because $testContext
                    } else {
                        $expected = [regex]::Escape("ERROR $expectedMessage")
                        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $expected -Because $testContext
                    }
                }
                It "raises a warning for an invalid value" {
                    $value = 'invalid'
                    $expected = "unable to case-insensitively parse '$value' as a valid DSC settings trace format"
                    $data = [ordered]@{
                        tracing = [ordered]@{
                            format = $value
                        }
                    }
                    Set-SettingsFile -Scope $scope -Settings $data
                    dsc resource list *echo* 2> $TestDrive/tracing.txt
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $expected -Because (
                        New-TestContext -TraceFilePath $TestDrive/tracing.txt
                    )
                }
            }

            Context "resourcePath" {
                BeforeAll {
                    $escapedDscHome = [Regex]::Escape($script:dscHome)
                    $resourcePath = Join-Path -Path $TestDrive -ChildPath 'resources'
                    $escapedResourcePath = [Regex]::Escape($resourcePath)
                    $escapedResourcePathJson = $IsWindows ? [Regex]::Escape($escapedResourcePath) : $escapedResourcePath
                    [string[]]$directoriesWithoutDscHome = @($resourcePath)
                    [string[]]$directoriesWithDscHome    = @($resourcePath, $script:dscHome)
                    $data = [ordered]@{
                        resourcePath = [ordered]@{}
                    }
                    $dscechoPath = Get-Command dscecho | Select-Object -ExpandProperty Path
                    [string]$binPath = Join-Path -Path $TestDrive -ChildPath 'bin'
                    [string[]]$binPaths = @(
                        $binPath
                        (Get-Command pwsh).Path | Split-Path -Parent
                    )
                    if ($IsWindows) {
                        $binPaths += (Get-Command powershell).Path | Split-Path -Parent
                    }
                    $resourceManifest = @{
                        '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json'
                        type      = 'DSC.Test/ResourcePath'
                        version   = '0.1.0'
                        get       = [ordered]@{
                            executable = Join-Path $binPath 'dscecho'
                            args       = @(@{
                                jsonInputArg = '--input'
                                mandatory    = $true
                            })
                        }
                        schema = @{
                            command = @{
                                executable = Join-Path $binPath 'dscecho'
                            }
                        }
                    }
                    $resourceManifestJson = $resourceManifest | ConvertTo-Json -Depth 10
                }

                BeforeEach {
                    New-Item -ItemType Directory -Path $resourcePath -Force
                    $resourceManifestJson | Set-Content -Path (Join-Path $resourcePath "test.dsc.resource.json") -Force
                    New-Item -ItemType Directory -Path $binPath -Force
                    Copy-Item -Path $dscechoPath -Destination $binPath -Force
                }
                AfterEach {
                    # Avoid flaky race condition
                    Start-Sleep -Seconds 1
                    Remove-Item -Path $resourcePath -Recurse -Force
                    Remove-Item -Path $binPath -Recurse -Force
                }

                Context "when restricted is true" {
                    BeforeAll {
                        $data.resourcePath.restricted = $true
                    }

                    Context "when directories is undefined" {
                        BeforeAll {
                            if ($data.resourcePath.Keys -contains 'directories') {
                                $data.resourcePath.Remove('directories')
                            }
                        }

                        It "raises an error during discovery" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            dsc resource list 2> $TestDrive/tracing.txt
                            $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt

                            $expectedMessage = [regex]::Escape((@(
                                'unable to discover any manifests -'
                                'DSC setting `resourcePath.restricted` is `true`'
                                'and `resourcePath.directories` is empty'
                            ) -join ' '))
                            "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $expectedMessage -Because $testContext
                        }
                    }
                    Context "when directories is empty" {
                        BeforeAll {
                            $data.resourcePath.directories = @()
                        }

                        It "raises an error during discovery" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            dsc resource list 2> $TestDrive/tracing.txt
                            $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt

                            $expectedMessage = [regex]::Escape((@(
                                'unable to discover any manifests -'
                                'DSC setting `resourcePath.restricted` is `true`'
                                'and `resourcePath.directories` is empty'
                            ) -join ' '))
                            "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $expectedMessage -Because $testContext
                        }
                    }
                    Context "when directories doesn't include DSC exe home" {
                        BeforeAll {
                            $data.resourcePath.directories = $directoriesWithoutDscHome
                        }
                        AfterAll {
                            $data.resourcePath.remove('directories')
                        }

                        It "adds discoverable resource paths" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            $result = dsc resource list $resourceManifest.type 2> $TestDrive/tracing.txt
                            $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                            $result | Should -Not -BeNullOrEmpty -Because $testContext
                            $result.Count | Should -Be 1 -Because $testContext
                            $data = $result | ConvertFrom-Json -Depth 10
                            $data.type | Should -Be $resourceManifest.type -Because $testContext
                        }
                        It "restricts discovery for other paths" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            $result = dsc resource list 2> $TestDrive/tracing.txt
                            $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                            $result.Count | Should -Be 1 -Because $testContext
                            $data = $result | ConvertFrom-Json -Depth 10
                            $data.type | Should -Be $resourceManifest.type -Because $testContext
                        }
                        It "raises a warning for inability to use built-in resources and extensions" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            dsc resource list $resourceManifest.type 2> $TestDrive/tracing.txt
                            $expectedWarning = @(
                                'WARN cannot discover or invoke built-in resources or extensions'
                                "- DSC executable home '{0}' is not included" -f $escapedDscHome
                                'in the restricted resource path directories \(\["{0}"\]\)' -f $escapedResourcePathJson
                            ) -join ' '
                            "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $expectedWarning -Because (
                                New-TestContext -TraceFilePath $TestDrive/tracing.txt
                            )
                        }
                        It "errors when invoking an executable outside of the specified paths" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            $params = @(
                                'resource', 'get'
                                '--resource', $resourceManifest.type
                                '--input', '{"output": "test"}'
                            )
                            $result = dsc @params 2> $TestDrive/tracing.txt
                            $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                            $testContext.exitCode | Should -BeExactly 2 -Because $testContext
                            $result | Should -BeNullOrEmpty -Because $testContext

                            $exe = Split-Path -Path $resourceManifest.get.executable -Leaf
                            $parent = Split-Path -Path $resourceManifest.get.executable -Parent
                            $expectedError = @(
                                'ERROR unable to invoke `{0}` -' -f $exe
                                'DSC setting `resourcePath.restricted` is `true` and executable'
                                'parent folder `{0}` is not in `resourcePath.directories`' -f [regex]::Escape($parent)
                                '\(\[".+"\]\)'
                            ) -join ' '
                            "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $expectedError -Because $testContext
                        }
                        It "allows invoking an executable within the specified paths" {
                            $data.resourcePath.directories += $binPaths
                            Set-SettingsFile -Scope $scope -Settings $data
                            $params = @(
                                'resource', 'get'
                                '--resource', $resourceManifest.type
                                '--input', '{"output": "test"}'
                            )
                            $result = dsc @params 2> $TestDrive/tracing.txt
                            $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                            $testContext.exitCode | Should -Be 0 -Because $testContext
                            $result | Should -Not -BeNullOrEmpty -Because $testContext
                            $resultData = $result | ConvertFrom-Json -Depth 10
                            $resultData.actualState.output | Should -Be 'test' -Because $testContext
                        }
                    }
                    Context "when directories includes DSC exe home" {
                        BeforeAll {
                            $data.resourcePath.directories = $directoriesWithDscHome
                        }

                        It "adds discoverable resource paths" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            $result = dsc resource list $resourceManifest.type 2> $TestDrive/tracing.txt
                            $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                            $result | Should -Not -BeNullOrEmpty -Because $testContext
                            $result.Count | Should -Be 1 -Because $testContext
                            $data = $result | ConvertFrom-Json -Depth 10
                            $data.type | Should -Be $resourceManifest.type -Because $testContext
                        }
                        It "restricts discovery for other paths" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            $result = dsc resource list 2> $TestDrive/tracing.txt
                            $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt

                            $expectedResources = $script:dscBuiltInResources.Where({
                                $pattern = @(
                                    '^'             # Anchor to start of string
                                    $escapedDscHome # Start with DSC home folder
                                    '(\\|\/)'       # Allow exactly one directory separator
                                    '[^\\\/]+'      # Exclude resources in subfolders
                                ) -join ''
                                # Count only resources that match the pattern and have no condition,
                                # Since the condition is generally a which statement for resources
                                # that ship a manifest separately from the command to invoke, like
                                # the PowerShell adapter - these are _not_ discovered when the path
                                # is restricted. Ignore adapted resources.
                                (
                                    $_.path -match $pattern -and
                                    [string]::IsNullOrEmpty($_.manifest.condition) -and
                                    [string]::IsNullOrEmpty($_.requireAdapter)
                                )
                            })
                            $actualResources = $result | ConvertFrom-Json -Depth 99 | Where-Object {
                                [string]::IsNullOrEmpty($_.requireAdapter)
                            }
                            $actualResources.Count | Should -Be ($expectedResources.Count + 1) -Because $testContext
                        }
                        It "does not raise a warning for inability to use built-in resources and extensions" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            dsc resource list $resourceManifest.type 2> $TestDrive/tracing.txt
                            $expectedWarning = @(
                                'WARN cannot discover or invoke built-in resources or extensions'
                                "- DSC executable home '{0}' is not included" -f $escapedDscHome
                                'in the restricted resource path directories \(\["{0}"\]\)' -f $escapedResourcePathJson
                            ) -join ' '
                            "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $expectedWarning -Because (
                                New-TestContext -TraceFilePath $TestDrive/tracing.txt
                            )
                        }
                        It "errors when invoking an executable outside of the specified paths" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            $params = @(
                                'resource', 'get'
                                '--resource', $resourceManifest.type
                                '--input', '{"output": "test"}'
                            )
                            $result = dsc @params 2> $TestDrive/tracing.txt
                            $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                            $testContext.exitCode | Should -BeExactly 2 -Because $testContext
                            $result | Should -BeNullOrEmpty -Because $testContext

                            $exe = Split-Path -Path $resourceManifest.get.executable -Leaf
                            $parent = Split-Path -Path $resourceManifest.get.executable -Parent
                            $expectedError = @(
                                'ERROR unable to invoke `{0}` -' -f $exe
                                'DSC setting `resourcePath.restricted` is `true` and executable'
                                'parent folder `{0}` is not in `resourcePath.directories`' -f [regex]::Escape($parent)
                                '\(\[".+"\]\)'
                            ) -join ' '
                            "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $expectedError -Because $testContext
                        }
                        It "allows invoking an executable within the specified paths" {
                            $data.resourcePath.directories += $binPaths
                            Set-SettingsFile -Scope $scope -Settings $data
                            $params = @(
                                'resource', 'get'
                                '--resource', $resourceManifest.type
                                '--input', '{"output": "test"}'
                            )
                            $result = dsc @params 2> $TestDrive/tracing.txt
                            $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                            $testContext.exitCode | Should -Be 0 -Because $testContext
                            $result | Should -Not -BeNullOrEmpty -Because $testContext
                            $resultData = $result | ConvertFrom-Json -Depth 10
                            $resultData.actualState.output | Should -Be 'test' -Because $testContext
                        }
                    }
                }

                Context "when restricted is false" {
                    BeforeAll {
                        $data.resourcePath.restricted = $false
                    }

                    Context "when appendEnvPath is true" {
                        BeforeAll {
                            $data.resourcePath.appendEnvPath = $true
                        }

                        Context "when directories is undefined" {
                            BeforeAll {
                                if ($data.resourcePath.Keys -contains 'directories') {
                                    $data.resourcePath.Remove('directories')
                                }
                            }

                            It "discovers all builtin resources" {
                                Set-SettingsFile -Scope $scope -Settings $data
                                $result = dsc resource list 2> $TestDrive/tracing.txt
                                $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt
                                $testContext.exitCode | Should -Be 0 -Because $testContext
                                $result.Count | Should -Be $script:dscBuiltInResources.Count -Because $testContext
                            }
                            It "writes a debug message for appending PATH to resourcePath" {
                                Set-SettingsFile -Scope $scope -Settings $data
                                dsc --trace-level debug resource list 2> $TestDrive/tracing.txt
                                $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt
                                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Appending PATH to resourcePath' -Because $testContext
                            }
                            It "can't discover resources outside of DSC exe home and PATH" {
                                Set-SettingsFile -Scope $scope -Settings $data
                                $result = dsc resource list $resourceManifest.type 2> $TestDrive/tracing.txt
                                $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                                $result | Should -BeNullOrEmpty -Because $testContext
                            }
                        }
                        Context "when directories doesn't include DSC exe home" {
                            BeforeAll {
                                $data.resourcePath.directories = $directoriesWithoutDscHome
                            }

                            It "discovers all builtin resources" {
                                Set-SettingsFile -Scope $scope -Settings $data
                                $result = dsc resource list 2> $TestDrive/tracing.txt
                                $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt
                                $testContext.exitCode | Should -Be 0 -Because $testContext
                                $expectedCount = $script:dscBuiltInResources.Count + 1
                                $result.Count | Should -Be $expectedCount -Because $testContext
                            }

                            It "writes a debug message for appending PATH to resourcePath" {
                                Set-SettingsFile -Scope $scope -Settings $data
                                dsc --trace-level debug resource list 2> $TestDrive/tracing.txt
                                $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt
                                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Appending PATH to resourcePath' -Because $testContext
                            }
                            It "doesn't raise a warning about missing DSC exe home in directories" {
                                Set-SettingsFile -Scope $scope -Settings $data
                                dsc resource list $resourceManifest.type 2> $TestDrive/tracing.txt
                                $expectedWarning = @(
                                    'WARN cannot discover or invoke built-in resources or extensions'
                                    "- DSC executable home '{0}' is not included" -f $escapedDscHome
                                    'in the restricted resource path directories \(\["{0}"\]\)' -f $escapedResourcePathJson
                                ) -join ' '
                                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $expectedWarning -Because (
                                    New-TestContext -TraceFilePath $TestDrive/tracing.txt
                                )
                            }
                            It "adds discoverable resource paths" {
                                Set-SettingsFile -Scope $scope -Settings $data
                                $result = dsc resource list $resourceManifest.type 2> $TestDrive/tracing.txt
                                $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                                $result | Should -Not -BeNullOrEmpty -Because $testContext
                                $result.Count | Should -Be 1 -Because $testContext
                                $data = $result | ConvertFrom-Json -Depth 10
                                $data.type | Should -Be $resourceManifest.type -Because $testContext
                            }
                            It "allows invoking an executable outside of the specified paths" {
                                Set-SettingsFile -Scope $scope -Settings $data
                                $params = @(
                                    'resource', 'get'
                                    '--resource', $resourceManifest.type
                                    '--input', '{"output": "test"}'
                                )
                                $result = dsc @params 2> $TestDrive/tracing.txt
                                $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                                $testContext.exitCode | Should -Be 0 -Because $testContext
                                $result | Should -Not -BeNullOrEmpty -Because $testContext
                                $resultData = $result | ConvertFrom-Json -Depth 10
                                $resultData.actualState.output | Should -Be 'test' -Because $testContext
                            }
                        }
                        Context "when directories includes DSC exe home" {
                            BeforeAll {
                                $data.resourcePath.directories = $directoriesWithDscHome
                            }
                            It "discovers all builtin resources" {
                                Set-SettingsFile -Scope $scope -Settings $data
                                $result = dsc resource list 2> $TestDrive/tracing.txt
                                $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt
                                $testContext.exitCode | Should -Be 0 -Because $testContext
                                $expectedCount = $script:dscBuiltInResources.Count + 1
                                $result.Count | Should -Be $expectedCount -Because $testContext
                            }

                            It "writes a debug message for appending PATH to resourcePath" {
                                Set-SettingsFile -Scope $scope -Settings $data
                                dsc --trace-level debug resource list 2> $TestDrive/tracing.txt
                                $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt
                                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Appending PATH to resourcePath' -Because $testContext
                            }
                            It "doesn't raise a warning about missing DSC exe home in directories" {
                                Set-SettingsFile -Scope $scope -Settings $data
                                dsc resource list $resourceManifest.type 2> $TestDrive/tracing.txt
                                $expectedWarning = @(
                                    'WARN cannot discover or invoke built-in resources or extensions'
                                    "- DSC executable home '{0}' is not included" -f $escapedDscHome
                                    'in the restricted resource path directories \(\["{0}"\]\)' -f $escapedResourcePathJson
                                ) -join ' '
                                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $expectedWarning -Because (
                                    New-TestContext -TraceFilePath $TestDrive/tracing.txt
                                )
                            }
                            It "adds discoverable resource paths" {
                                Set-SettingsFile -Scope $scope -Settings $data
                                $result = dsc resource list $resourceManifest.type 2> $TestDrive/tracing.txt
                                $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                                $result | Should -Not -BeNullOrEmpty -Because $testContext
                                $result.Count | Should -Be 1 -Because $testContext
                                $data = $result | ConvertFrom-Json -Depth 10
                                $data.type | Should -Be $resourceManifest.type -Because $testContext
                            }
                            It "allows invoking an executable outside of the specified paths" {
                                Set-SettingsFile -Scope $scope -Settings $data
                                $params = @(
                                    'resource', 'get'
                                    '--resource', $resourceManifest.type
                                    '--input', '{"output": "test"}'
                                )
                                $result = dsc @params 2> $TestDrive/tracing.txt
                                $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                                $testContext.exitCode | Should -Be 0 -Because $testContext
                                $result | Should -Not -BeNullOrEmpty -Because $testContext
                                $resultData = $result | ConvertFrom-Json -Depth 10
                                $resultData.actualState.output | Should -Be 'test' -Because $testContext
                            }
                        }
                    }
                    Context "when appendEnvPath is false" {
                        BeforeAll {
                            $data.resourcePath.appendEnvPath = $false
                        }
                        Context "when directories is undefined" {
                            BeforeAll {
                                if ($data.resourcePath.Keys -contains 'directories') {
                                    $data.resourcePath.Remove('directories')
                                }
                            }

                            It "discovers all builtin resources in DSC exe home" {
                                Set-SettingsFile -Scope $scope -Settings $data
                                $result = dsc resource list 2> $TestDrive/tracing.txt
                                $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt
                                $testContext.exitCode | Should -Be 0 -Because $testContext

                                $builtInInHome = $script:dscBuiltInResources.Where({$_.Path.StartsWith($script:dscHome)})
                                $actualResources = $result | ConvertFrom-Json -Depth 99
                                # Ensure all builtin resources in DSC exe home are returned
                                foreach ($resource in $builtInInHome) {
                                    $resource.type | Should -BeIn $actualResources.type -Because (
                                        $resource
                                        | Select-Object -Property type, path, requireAdapter, @{n='condition'; e={$_.manifest.condition}}
                                        | ConvertTo-Json
                                    )
                                }
                                # Count must be greater than or equal, depending on resources from discovery extensions
                                $result.Count | Should -BeGreaterOrEqual $builtInInHome.Count -Because $testContext
                            }
                            It "doesn't write a debug message for appending PATH to resourcePath" {
                                Set-SettingsFile -Scope $scope -Settings $data
                                dsc --trace-level debug resource list 2> $TestDrive/tracing.txt
                                $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt
                                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly 'Appending PATH to resourcePath' -Because $testContext
                            }
                            It "can't discover resources outside of DSC exe home or discovery extensions" {
                                Set-SettingsFile -Scope $scope -Settings $data
                                $result = dsc resource list $resourceManifest.type 2> $TestDrive/tracing.txt
                                $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                                $result | Should -BeNullOrEmpty -Because $testContext
                            }
                        }
                        Context "when directories is defined" {
                            BeforeAll {
                                $data.resourcePath.directories = $directoriesWithoutDscHome
                            }

                            It "discovers all builtin resources in DSC exe home" {
                                Set-SettingsFile -Scope $scope -Settings $data
                                $result = dsc resource list 2> $TestDrive/tracing.txt
                                $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt
                                $testContext.exitCode | Should -Be 0 -Because $testContext

                                $builtInInHome = $script:dscBuiltInResources.Where({$_.Path.StartsWith($script:dscHome)})
                                $actualResources = $result | ConvertFrom-Json -Depth 99
                                # Ensure all builtin resources in DSC exe home are returned
                                foreach ($resource in $builtInInHome) {
                                    $resource.type | Should -BeIn $actualResources.type -Because (
                                        $resource
                                        | Select-Object -Property type, path, requireAdapter, @{n='condition'; e={$_.manifest.condition}}
                                        | ConvertTo-Json
                                    )
                                }
                                # Count must be greater than or equal, depending on resources from discovery extensions
                                $result.Count | Should -BeGreaterOrEqual $builtInInHome.Count -Because $testContext
                            }
                            It "doesn't write a debug message for appending PATH to resourcePath" {
                                Set-SettingsFile -Scope $scope -Settings $data
                                dsc --trace-level debug resource list 2> $TestDrive/tracing.txt
                                $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt
                                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly 'Appending PATH to resourcePath' -Because $testContext
                            }
                            It "discovers resources in specified directories" {
                                Set-SettingsFile -Scope $scope -Settings $data
                                $result = dsc resource list $resourceManifest.type 2> $TestDrive/tracing.txt
                                $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                                $result | Should -Not -BeNullOrEmpty -Because $testContext
                                $result.Count | Should -Be 1 -Because $testContext
                            }
                            It "allows invoking an executable outside of the specified paths" {
                                Set-SettingsFile -Scope $scope -Settings $data
                                $params = @(
                                    'resource', 'get'
                                    '--resource', $resourceManifest.type
                                    '--input', '{"output": "test"}'
                                )
                                $result = dsc @params 2> $TestDrive/tracing.txt
                                $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                                $testContext.exitCode | Should -Be 0 -Because $testContext
                                $result | Should -Not -BeNullOrEmpty -Because $testContext
                                $resultData = $result | ConvertFrom-Json -Depth 10
                                $resultData.actualState.output | Should -Be 'test' -Because $testContext
                            }
                        }
                    }
                }
            }
        }
    }

    Context 'policy settings' -Skip:$script:skipSystem {
        BeforeAll {
            $scope = 'policy'
        }

        Context 'forbidIgnoreSettingsFile' {
            BeforeAll {
                $userData = [ordered]@{
                    tracing = [ordered]@{
                        level = 'debug'
                    }
                }
                $policyData = [ordered]@{}
                $messages = [pscustomobject]@{
                    Debug   = "debug message for test"
                    Info    = "info message for test"
                    Warning = "warning message for test"
                    Error   = "error message for test"
                }
                $resourceInput = [ordered]@{
                    getScript =  @(
                        "Write-Debug       '$($messages.Debug)'"
                        "Write-Verbose     '$($messages.Info)'"
                        "Write-Warning     '$($messages.Warning)'"
                        "Write-Error       '$($messages.Error)'"
                    ) -join "`n"
                } | ConvertTo-Json
                $resourceType = 'Microsoft.DSC.Transitional/PowerShellScript'
            }

            BeforeEach {
                Set-SettingsFile -Scope user -Settings $userData
            }

            Context 'when undefined' {
                it "loads settings from preference files when ``ignoreSettingsFile`` is not ``true``" {
                    Set-SettingsFile -Scope $scope -Settings $policyData
                    $params = @(
                        'resource', 'get'
                        '--resource', $resourceType
                        '--input', $resourceInput
                    )
                    dsc @params 2> $TestDrive/tracing.txt
                    $testContext = New-TestContext  -TraceFilePath $TestDrive/tracing.txt

                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Error -Because $testContext
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Warning -Because $testContext
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Info -Because $testContext
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Debug -Because $testContext
                }
                It "allows non-policy scope '<_>' to ignore settings files" -ForEach @('environment', 'cli') {
                    Set-SettingsFile -Scope $scope -Settings $policyData
                    $params = @(
                        'resource', 'get'
                        '--resource', $resourceType
                        '--input', $resourceInput
                    )
                    switch ($_) {
                        'environment' {
                            Set-SettingsEnvironment -IgnoreSettingsFile 'true'
                        }
                        'cli' {
                            $params = @('--ignore-settings-file') + $params
                        }
                    }

                    dsc @params 2> $TestDrive/tracing.txt
                    $testContext = New-TestContext  -TraceFilePath $TestDrive/tracing.txt

                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Error -Because $testContext
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Warning -Because $testContext
                    "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $messages.Info -Because $testContext
                    "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $messages.Debug -Because $testContext
                }
            }
            Context 'when false' {
                BeforeAll {
                    $policyData.forbidIgnoreSettingsFile = $false
                }
                AfterEach {
                    if ($policyData.Keys -contains 'ignoreSettingsFile') {
                        $policyData.Remove('ignoreSettingsFile')
                    }
                }
                it "loads settings from preference files when ``ignoreSettingsFile`` is not ``true``" {
                    Set-SettingsFile -Scope $scope -Settings $policyData
                    $params = @(
                        'resource', 'get'
                        '--resource', $resourceType
                        '--input', $resourceInput
                    )
                    dsc @params 2> $TestDrive/tracing.txt
                    $testContext = New-TestContext  -TraceFilePath $TestDrive/tracing.txt

                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Error -Because $testContext
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Warning -Because $testContext
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Info -Because $testContext
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Debug -Because $testContext
                }
                It "allows ignoring settings files with the <setting>" -ForEach @(
                    @{ id = 'env'; setting = 'DSC_IGNORE_SETTINGS_FILE env var' }
                    @{ id = 'cli'; setting = '--ignore-settings-file cli arg' }
                    @{ id = 'policy'; setting = 'ignoreSettingsFile policy field'}
                ) {
                    $params = @(
                        'resource', 'get'
                        '--resource', $resourceType
                        '--input', $resourceInput
                    )
                    switch ($id) {
                        'env' {
                            Set-SettingsEnvironment -IgnoreSettingsFile 'true'
                        }
                        'cli' {
                            $params = @('--ignore-settings-file') + $params
                        }
                        'policy' {
                            $policyData.ignoreSettingsFile = $true
                        }
                    }
                    Set-SettingsFile -Scope $scope -Settings $policyData

                    dsc @params 2> $TestDrive/tracing.txt
                    $testContext = New-TestContext  -TraceFilePath $TestDrive/tracing.txt

                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Error -Because $testContext
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Warning -Because $testContext
                    "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $messages.Info -Because $testContext
                    "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $messages.Debug -Because $testContext
                }
            }
            Context "when true" {
                BeforeAll {
                    $policyData.forbidIgnoreSettingsFile = $true
                }
                AfterAll {
                    $policyData = [ordered]@{}
                }

                it 'loads settings from preference files when `ignoreSettingsFile` is not `true`'  {
                    Set-SettingsFile -Scope $scope -Settings $policyData
                    $params = @(
                        'resource', 'get'
                        '--resource', $resourceType
                        '--input', $resourceInput
                    )
                    dsc @params 2> $TestDrive/tracing.txt
                    $testContext = New-TestContext  -TraceFilePath $TestDrive/tracing.txt

                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Error -Because $testContext
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Warning -Because $testContext
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Info -Because $testContext
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Debug -Because $testContext
                }
                It 'forbids ignoring settings files with the <setting>' -ForEach @(
                    @{ id = 'env'; setting = 'DSC_IGNORE_SETTINGS_FILE env var' }
                    @{ id = 'cli'; setting = '--ignore-settings-file cli arg' }
                    @{ id = 'policy'; setting = 'ignoreSettingsFile policy field'}
                ) {
                    $params = @(
                        'resource', 'get'
                        '--resource', $resourceType
                        '--input', $resourceInput
                    )
                    $messagePrefix = 'WARN\s+.+:\s+\d+:'
                    $messageSuffix = '.+`forbidIgnoreSettingsFile`'
                    switch ($id) {
                        'env' {
                            Set-SettingsEnvironment -IgnoreSettingsFile 'true'
                            $message = 'ignoring the environment variable `DSC_IGNORE_SETTINGS_FILE`'
                        }
                        'cli' {
                            $params = @('--ignore-settings-file') + $params
                            $message = 'ignoring the command line argument `--ignore-settings-file`'
                        }
                        'policy' {
                            $policyData.ignoreSettingsFile = $true
                            $message = 'ignoring the policy setting `ignoreSettingsFile`'
                        }
                    }
                    Set-SettingsFile -Scope $scope -Settings $policyData

                    dsc @params 2> $TestDrive/tracing.txt
                    $testContext = New-TestContext  -TraceFilePath $TestDrive/tracing.txt

                    $expectedMessage = @(
                        $messagePrefix
                        $message
                        $messageSuffix
                    ) -join '\s+'

                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Error -Because $testContext
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Warning -Because $testContext
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Info -Because $testContext
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Debug -Because $testContext
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $expectedMessage -Because $testContext
                }
            }
        }

        Context "ignoreSettingsFile" {
            BeforeAll {
                $policyData = [ordered]@{}
                $userPreferenceData = [ordered]@{
                    tracing = [ordered]@{
                        level = 'info'
                    }
                }
            }
            BeforeEach {
                Set-SettingsFile -Scope User -Settings $userPreferenceData
            }

            Context "when undefined" {
                BeforeAll {
                    if ($policyData.Keys -contains 'ignoreSettingsFile') {
                        $policyData.Remove('ignoreSettingsFile')
                    }
                }

                it "loads settings files" {
                    Set-SettingsFile -Scope $scope -Settings $policyData
                    dsc resource list Microsoft.DSC.Debug/Echo 2> $TestDrive/tracing.txt
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Trace-level is Info' -Because (
                        New-TestContext -Message 'expected user scope tracing.level as info to emit trace message' -TraceFilePath $TestDrive/tracing.txt
                    )
                }
            }
            Context "when false" {
                BeforeAll {
                    $policyData.ignoreSettingsFile = $false
                }

                It "loads settings files" {
                    Set-SettingsFile -Scope $scope -Settings $policyData
                    dsc resource list Microsoft.DSC.Debug/Echo 2> $TestDrive/tracing.txt
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Trace-level is Info' -Because (
                        New-TestContext -Message 'expected user scope tracing.level as info to emit trace message' -TraceFilePath $TestDrive/tracing.txt
                    )
                }
            }
            Context "when true" {
                BeforeAll {
                    $policyData.ignoreSettingsFile = $true
                }

                It "ignores settings files" {
                    Set-SettingsFile -Scope $scope -Settings $policyData
                    dsc resource list Microsoft.DSC.Debug/Echo 2> $TestDrive/tracing.txt
                    Get-Content -Raw -Path "$TestDrive/tracing.txt" | Should -BeNullOrEmpty -Because (
                    New-TestContext -Message 'expected no trace message when ignoring settings file' -TraceFilePath $TestDrive/tracing.txt
                )
                }
            }
        }

        Context 'tracing.level' {
            It "emits expected messaages for '<_>' trace level" -ForEach @("error", "warn", "info", "debug", "trace") {
                $level = $_
                $data = [ordered]@{
                    tracing = [ordered]@{
                        level = $level
                    }
                }
                Set-SettingsFile -Scope $scope -Settings $data
                $messages = [pscustomobject]@{
                    Trace   = "trace resource message for level: $level"
                    Debug   = "debug message for level: $level"
                    Info    = "info message for level: $level"
                    Warning = "warning message for level: $level"
                    Error   = "error message for level: $level"
                }
                $params = @(
                    'resource', 'get'
                    '--resource', 'Microsoft.DSC.Transitional/PowerShellScript'
                    '--input', ([ordered]@{
                        getScript = @(
                            "Write-Information '$($messages.Trace)'"
                            "Write-Debug       '$($messages.Debug)'"
                            "Write-Verbose     '$($messages.Info)'"
                            "Write-Warning     '$($messages.Warning)'"
                            "Write-Error       '$($messages.Error)'"
                        ) -join "`n"
                    } | ConvertTo-Json)
                )
                dsc @params 2> $TestDrive/tracing.txt
                $testContext = New-TestContext  -TraceFilePath $TestDrive/tracing.txt

                # All levels should find the error message
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Error -Because $testContext
                if ($level -ne "error") {
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Warning -Because $testContext
                }
                if ($level -notin @('error', 'warn')) {
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Info -Because $testContext
                }
                if ($level -notin ('error', 'warn', 'info')) {
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Debug -Because $testContext
                }
                if ($level -eq 'trace') {
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Trace -Because $testContext
                }
            }

            It "raises a warning for an invalid value" {
                $value = 'invalid'
                $expected = "unable to case-insensitively parse '$value' as a valid DSC settings trace level"
                $data = [ordered]@{
                    tracing = [ordered]@{
                        level = $value
                    }
                }
                Set-SettingsFile -Scope $scope -Settings $data
                dsc resource list *echo* 2> $TestDrive/tracing.txt
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $expected -Because {
                    New-TestContext -TraceFilePath $TestDrive/tracing.txt
                }
            }
        }

        Context 'tracing.format' { 
            It "emits expected format for '<_>' trace format" -ForEach @('default', 'plaintext', 'json') {
                $format = $_
                $invalidKey = "trace_format.$_"
                $data = [ordered]@{
                    tracing = [ordered]@{
                        format = $format
                    }
                }
                Set-SettingsFile -Scope $scope -Settings $data
                Set-SettingsEnvironment -TraceLevel Error
                $params = @(
                    'resource', 'get'
                    '--resource', 'Microsoft.DSC.Debug/Echo'
                    '--input', (@{ $invalidKey = $format} | ConvertTo-Json -Compress)
                )
                dsc @params 2> $TestDrive/tracing.txt
                $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt

                $expectedMessage = "Schema: Additional properties are not allowed ('$invalidKey' was unexpected)"
                if ($format -eq 'json') {
                    $traces = Get-Content $TestDrive/tracing.txt | ConvertFrom-Json
                    $traces.count | Should -Be 1 -Because $testContext
                    $traces[0].level | Should -Be 'Error' -Because $testContext
                    $traces[0].fields.message | Should -Be $expectedMessage -Because $testContext
                } else {
                    $expected = [regex]::Escape("ERROR $expectedMessage")
                    "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $expected -Because $testContext
                }
            }

            It "raises a warning for an invalid value" {
                $value = 'invalid'
                $expected = "unable to case-insensitively parse '$value' as a valid DSC settings trace format"
                $data = [ordered]@{
                    tracing = [ordered]@{
                        format = $value
                    }
                }
                Set-SettingsFile -Scope $scope -Settings $data
                dsc resource list *echo* 2> $TestDrive/tracing.txt
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $expected -Because (
                    New-TestContext -TraceFilePath $TestDrive/tracing.txt
                )
            }
        }

        Context "resourcePath" {
            BeforeAll {
                $escapedDscHome = [Regex]::Escape($script:dscHome)
                $resourcePath = Join-Path -Path $TestDrive -ChildPath 'resources'
                $escapedResourcePath = [Regex]::Escape($resourcePath)
                $escapedResourcePathJson = $IsWindows ? [Regex]::Escape($escapedResourcePath) : $escapedResourcePath
                [string[]]$directoriesWithoutDscHome = @($resourcePath)
                [string[]]$directoriesWithDscHome    = @($resourcePath, $script:dscHome)
                $data = [ordered]@{
                    resourcePath = [ordered]@{}
                }
                $dscechoPath = Get-Command dscecho | Select-Object -ExpandProperty Path
                [string]$binPath = Join-Path -Path $TestDrive -ChildPath 'bin'
                [string[]]$binPaths = @(
                    $binPath
                    (Get-Command powershell).Path | Split-Path -Parent
                    (Get-Command pwsh).Path | Split-Path -Parent
                )
                $resourceManifest = @{
                    '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json'
                    type      = 'DSC.Test/ResourcePath'
                    version   = '0.1.0'
                    get       = [ordered]@{
                        executable = Join-Path $binPath 'dscecho'
                        args       = @(@{
                            jsonInputArg = '--input'
                            mandatory    = $true
                        })
                    }
                    schema = @{
                        command = @{
                            executable = Join-Path $binPath 'dscecho'
                        }
                    }
                }
                $resourceManifestJson = $resourceManifest | ConvertTo-Json -Depth 10
            }

            BeforeEach {
                New-Item -ItemType Directory -Path $resourcePath -Force
                $resourceManifestJson | Set-Content -Path (Join-Path $resourcePath "test.dsc.resource.json") -Force
                New-Item -ItemType Directory -Path $binPath -Force
                Copy-Item -Path $dscechoPath -Destination $binPath -Force
            }
            AfterEach {
                # Avoid flaky race condition
                Start-Sleep -Seconds 1
                Remove-Item -Path $resourcePath -Recurse -Force
                Remove-Item -Path $binPath -Recurse -Force
            }

            Context "when restricted is true" {
                BeforeAll {
                    $data.resourcePath.restricted = $true
                }

                Context "when directories is undefined" {
                    BeforeAll {
                        if ($data.resourcePath.Keys -contains 'directories') {
                            $data.resourcePath.Remove('directories')
                        }
                    }

                    It "raises an error during discovery" {
                        Set-SettingsFile -Scope $scope -Settings $data
                        dsc resource list 2> $TestDrive/tracing.txt
                        $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt

                        $expectedMessage = [regex]::Escape((@(
                            'unable to discover any manifests -'
                            'DSC setting `resourcePath.restricted` is `true`'
                            'and `resourcePath.directories` is empty'
                        ) -join ' '))
                        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $expectedMessage -Because $testContext
                    }
                }
                Context "when directories is empty" {
                    BeforeAll {
                        $data.resourcePath.directories = @()
                    }

                    It "raises an error during discovery" {
                        Set-SettingsFile -Scope $scope -Settings $data
                        dsc resource list 2> $TestDrive/tracing.txt
                        $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt

                        $expectedMessage = [regex]::Escape((@(
                            'unable to discover any manifests -'
                            'DSC setting `resourcePath.restricted` is `true`'
                            'and `resourcePath.directories` is empty'
                        ) -join ' '))
                        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $expectedMessage -Because $testContext
                    }
                }
                Context "when directories doesn't include DSC exe home" {
                    BeforeAll {
                        $data.resourcePath.directories = $directoriesWithoutDscHome
                    }
                    AfterAll {
                        $data.resourcePath.remove('directories')
                    }

                    It "adds discoverable resource paths" {
                        Set-SettingsFile -Scope $scope -Settings $data
                        $result = dsc resource list $resourceManifest.type 2> $TestDrive/tracing.txt
                        $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                        $result | Should -Not -BeNullOrEmpty -Because $testContext
                        $result.Count | Should -Be 1 -Because $testContext
                        $data = $result | ConvertFrom-Json -Depth 10
                        $data.type | Should -Be $resourceManifest.type -Because $testContext
                    }
                    It "restricts discovery for other paths" {
                        Set-SettingsFile -Scope $scope -Settings $data
                        $result = dsc resource list 2> $TestDrive/tracing.txt
                        $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                        $result.Count | Should -Be 1 -Because $testContext
                        $data = $result | ConvertFrom-Json -Depth 10
                        $data.type | Should -Be $resourceManifest.type -Because $testContext
                    }
                    It "raises a warning for inability to use built-in resources and extensions" {
                        Set-SettingsFile -Scope $scope -Settings $data
                        dsc resource list $resourceManifest.type 2> $TestDrive/tracing.txt
                        $expectedWarning = @(
                            'WARN cannot discover or invoke built-in resources or extensions'
                            "- DSC executable home '{0}' is not included" -f $escapedDscHome
                            'in the restricted resource path directories \(\["{0}"\]\)' -f $escapedResourcePathJson
                        ) -join ' '
                        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $expectedWarning -Because (
                            New-TestContext -TraceFilePath $TestDrive/tracing.txt
                        )
                    }
                    It "errors when invoking an executable outside of the specified paths" {
                        Set-SettingsFile -Scope $scope -Settings $data
                        $params = @(
                            'resource', 'get'
                            '--resource', $resourceManifest.type
                            '--input', '{"output": "test"}'
                        )
                        $result = dsc @params 2> $TestDrive/tracing.txt
                        $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                        $testContext.exitCode | Should -BeExactly 2 -Because $testContext
                        $result | Should -BeNullOrEmpty -Because $testContext

                        $exe = Split-Path -Path $resourceManifest.get.executable -Leaf
                        $parent = Split-Path -Path $resourceManifest.get.executable -Parent
                        $expectedError = @(
                            'ERROR unable to invoke `{0}` -' -f $exe
                            'DSC setting `resourcePath.restricted` is `true` and executable'
                            'parent folder `{0}` is not in `resourcePath.directories`' -f [regex]::Escape($parent)
                            '\(\[".+"\]\)'
                        ) -join ' '
                        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $expectedError -Because $testContext
                    }
                    It "allows invoking an executable within the specified paths" {
                        $data.resourcePath.directories += $binPaths
                        Set-SettingsFile -Scope $scope -Settings $data
                        $params = @(
                            'resource', 'get'
                            '--resource', $resourceManifest.type
                            '--input', '{"output": "test"}'
                        )
                        $result = dsc @params 2> $TestDrive/tracing.txt
                        $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                        $testContext.exitCode | Should -Be 0 -Because $testContext
                        $result | Should -Not -BeNullOrEmpty -Because $testContext
                        $resultData = $result | ConvertFrom-Json -Depth 10
                        $resultData.actualState.output | Should -Be 'test' -Because $testContext
                    }
                }
                Context "when directories includes DSC exe home" {
                    BeforeAll {
                        $data.resourcePath.directories = $directoriesWithDscHome
                    }

                    It "adds discoverable resource paths" {
                        Set-SettingsFile -Scope $scope -Settings $data
                        $result = dsc resource list $resourceManifest.type 2> $TestDrive/tracing.txt
                        $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                        $result | Should -Not -BeNullOrEmpty -Because $testContext
                        $result.Count | Should -Be 1 -Because $testContext
                        $data = $result | ConvertFrom-Json -Depth 10
                        $data.type | Should -Be $resourceManifest.type -Because $testContext
                    }
                    It "restricts discovery for other paths" {
                        Set-SettingsFile -Scope $scope -Settings $data
                        $result = dsc resource list 2> $TestDrive/tracing.txt
                        $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt

                        $expectedResources = $script:dscBuiltInResources.Where({
                            $pattern = @(
                                '^'             # Anchor to start of string
                                $escapedDscHome # Start with DSC home folder
                                '(\\|\/)'       # Allow exactly one directory separator
                                '[^\\\/]+'      # Exclude resources in subfolders
                            ) -join ''
                            # Count only resources that match the pattern and have no condition,
                            # Since the condition is generally a which statement for resources
                            # that ship a manifest separately from the command to invoke, like
                            # the PowerShell adapter - these are _not_ discovered when the path
                            # is restricted.
                            (
                                $_.path -match $pattern -and
                                [string]::IsNullOrEmpty($_.manifest.condition) -and
                                [string]::IsNullOrEmpty($_.requireAdapter)
                            )
                        })
                        $actualResources = $result | ConvertFrom-Json -Depth 99 | Where-Object {
                            [string]::IsNullOrEmpty($_.requireAdapter)
                        }
                        $actualResources.Count | Should -Be ($expectedResources.Count + 1) -Because $testContext
                    }
                    It "does not raise a warning for inability to use built-in resources and extensions" {
                        Set-SettingsFile -Scope $scope -Settings $data
                        dsc resource list $resourceManifest.type 2> $TestDrive/tracing.txt
                        $expectedWarning = @(
                            'WARN cannot discover or invoke built-in resources or extensions'
                            "- DSC executable home '{0}' is not included" -f $escapedDscHome
                            'in the restricted resource path directories \(\["{0}"\]\)' -f $escapedResourcePathJson
                        ) -join ' '
                        "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $expectedWarning -Because (
                            New-TestContext -TraceFilePath $TestDrive/tracing.txt
                        )
                    }
                    It "errors when invoking an executable outside of the specified paths" {
                        Set-SettingsFile -Scope $scope -Settings $data
                        $params = @(
                            'resource', 'get'
                            '--resource', $resourceManifest.type
                            '--input', '{"output": "test"}'
                        )
                        $result = dsc @params 2> $TestDrive/tracing.txt
                        $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                        $testContext.exitCode | Should -BeExactly 2 -Because $testContext
                        $result | Should -BeNullOrEmpty -Because $testContext

                        $exe = Split-Path -Path $resourceManifest.get.executable -Leaf
                        $parent = Split-Path -Path $resourceManifest.get.executable -Parent
                        $expectedError = @(
                            'ERROR unable to invoke `{0}` -' -f $exe
                            'DSC setting `resourcePath.restricted` is `true` and executable'
                            'parent folder `{0}` is not in `resourcePath.directories`' -f [regex]::Escape($parent)
                            '\(\[".+"\]\)'
                        ) -join ' '
                        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $expectedError -Because $testContext
                    }
                    It "allows invoking an executable within the specified paths" {
                        $data.resourcePath.directories += $binPaths
                        Set-SettingsFile -Scope $scope -Settings $data
                        $params = @(
                            'resource', 'get'
                            '--resource', $resourceManifest.type
                            '--input', '{"output": "test"}'
                        )
                        $result = dsc @params 2> $TestDrive/tracing.txt
                        $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                        $testContext.exitCode | Should -Be 0 -Because $testContext
                        $result | Should -Not -BeNullOrEmpty -Because $testContext
                        $resultData = $result | ConvertFrom-Json -Depth 10
                        $resultData.actualState.output | Should -Be 'test' -Because $testContext
                    }
                }
            }

            Context "when restricted is false" {
                BeforeAll {
                    $data.resourcePath.restricted = $false
                }

                Context "when appendEnvPath is true" {
                    BeforeAll {
                        $data.resourcePath.appendEnvPath = $true
                    }

                    Context "when directories is undefined" {
                        BeforeAll {
                            if ($data.resourcePath.Keys -contains 'directories') {
                                $data.resourcePath.Remove('directories')
                            }
                        }

                        It "discovers all builtin resources" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            $result = dsc resource list 2> $TestDrive/tracing.txt
                            $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt
                            $testContext.exitCode | Should -Be 0 -Because $testContext
                            $result.Count | Should -Be $script:dscBuiltInResources.Count -Because $testContext
                        }
                        It "writes a debug message for appending PATH to resourcePath" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            dsc --trace-level debug resource list 2> $TestDrive/tracing.txt
                            $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt
                            "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Appending PATH to resourcePath' -Because $testContext
                        }
                        It "can't discover resources outside of DSC exe home and PATH" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            $result = dsc resource list $resourceManifest.type 2> $TestDrive/tracing.txt
                            $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                            $result | Should -BeNullOrEmpty -Because $testContext
                        }
                    }
                    Context "when directories doesn't include DSC exe home" {
                        BeforeAll {
                            $data.resourcePath.directories = $directoriesWithoutDscHome
                        }

                        It "discovers all builtin resources" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            $result = dsc resource list 2> $TestDrive/tracing.txt
                            $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt
                            $testContext.exitCode | Should -Be 0 -Because $testContext
                            $expectedCount = $script:dscBuiltInResources.Count + 1
                            $result.Count | Should -Be $expectedCount -Because $testContext
                        }

                        It "writes a debug message for appending PATH to resourcePath" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            dsc --trace-level debug resource list 2> $TestDrive/tracing.txt
                            $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt
                            "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Appending PATH to resourcePath' -Because $testContext
                        }
                        It "doesn't raise a warning about missing DSC exe home in directories" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            dsc resource list $resourceManifest.type 2> $TestDrive/tracing.txt
                            $expectedWarning = @(
                                'WARN cannot discover or invoke built-in resources or extensions'
                                "- DSC executable home '{0}' is not included" -f $escapedDscHome
                                'in the restricted resource path directories \(\["{0}"\]\)' -f $escapedResourcePathJson
                            ) -join ' '
                            "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $expectedWarning -Because (
                                New-TestContext -TraceFilePath $TestDrive/tracing.txt
                            )
                        }
                        It "adds discoverable resource paths" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            $result = dsc resource list $resourceManifest.type 2> $TestDrive/tracing.txt
                            $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                            $result | Should -Not -BeNullOrEmpty -Because $testContext
                            $result.Count | Should -Be 1 -Because $testContext
                            $data = $result | ConvertFrom-Json -Depth 10
                            $data.type | Should -Be $resourceManifest.type -Because $testContext
                        }
                        It "allows invoking an executable outside of the specified paths" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            $params = @(
                                'resource', 'get'
                                '--resource', $resourceManifest.type
                                '--input', '{"output": "test"}'
                            )
                            $result = dsc @params 2> $TestDrive/tracing.txt
                            $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                            $testContext.exitCode | Should -Be 0 -Because $testContext
                            $result | Should -Not -BeNullOrEmpty -Because $testContext
                            $resultData = $result | ConvertFrom-Json -Depth 10
                            $resultData.actualState.output | Should -Be 'test' -Because $testContext
                        }
                    }
                    Context "when directories includes DSC exe home" {
                        BeforeAll {
                            $data.resourcePath.directories = $directoriesWithDscHome
                        }
                        It "discovers all builtin resources" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            $result = dsc resource list 2> $TestDrive/tracing.txt
                            $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt
                            $testContext.exitCode | Should -Be 0 -Because $testContext
                            $expectedCount = $script:dscBuiltInResources.Count + 1
                            $result.Count | Should -Be $expectedCount -Because $testContext
                        }

                        It "writes a debug message for appending PATH to resourcePath" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            dsc --trace-level debug resource list 2> $TestDrive/tracing.txt
                            $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt
                            "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Appending PATH to resourcePath' -Because $testContext
                        }
                        It "doesn't raise a warning about missing DSC exe home in directories" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            dsc resource list $resourceManifest.type 2> $TestDrive/tracing.txt
                            $expectedWarning = @(
                                'WARN cannot discover or invoke built-in resources or extensions'
                                "- DSC executable home '{0}' is not included" -f $escapedDscHome
                                'in the restricted resource path directories \(\["{0}"\]\)' -f $escapedResourcePathJson
                            ) -join ' '
                            "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $expectedWarning -Because (
                                New-TestContext -TraceFilePath $TestDrive/tracing.txt
                            )
                        }
                        It "adds discoverable resource paths" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            $result = dsc resource list $resourceManifest.type 2> $TestDrive/tracing.txt
                            $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                            $result | Should -Not -BeNullOrEmpty -Because $testContext
                            $result.Count | Should -Be 1 -Because $testContext
                            $data = $result | ConvertFrom-Json -Depth 10
                            $data.type | Should -Be $resourceManifest.type -Because $testContext
                        }
                        It "allows invoking an executable outside of the specified paths" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            $params = @(
                                'resource', 'get'
                                '--resource', $resourceManifest.type
                                '--input', '{"output": "test"}'
                            )
                            $result = dsc @params 2> $TestDrive/tracing.txt
                            $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                            $testContext.exitCode | Should -Be 0 -Because $testContext
                            $result | Should -Not -BeNullOrEmpty -Because $testContext
                            $resultData = $result | ConvertFrom-Json -Depth 10
                            $resultData.actualState.output | Should -Be 'test' -Because $testContext
                        }
                    }
                }
                Context "when appendEnvPath is false" {
                    BeforeAll {
                        $data.resourcePath.appendEnvPath = $false
                    }
                    Context "when directories is undefined" {
                        BeforeAll {
                            if ($data.resourcePath.Keys -contains 'directories') {
                                $data.resourcePath.Remove('directories')
                            }
                        }

                        It "discovers all builtin resources in DSC exe home" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            $result = dsc resource list 2> $TestDrive/tracing.txt
                            $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt
                            $testContext.exitCode | Should -Be 0 -Because $testContext

                            $builtInInHome = $script:dscBuiltInResources.Where({$_.Path.StartsWith($script:dscHome)})
                            $actualResources = $result | ConvertFrom-Json -Depth 99
                            # Ensure all builtin resources in DSC exe home are returned
                            foreach ($resource in $builtInInHome) {
                                $resource.type | Should -BeIn $actualResources.type -Because (
                                    $resource
                                    | Select-Object -Property type, path, requireAdapter, @{n='condition'; e={$_.manifest.condition}}
                                    | ConvertTo-Json
                                )
                            }
                            # Count must be greater than or equal, depending on resources from discovery extensions
                            $result.Count | Should -BeGreaterOrEqual $builtInInHome.Count -Because $testContext
                        }
                        It "doesn't write a debug message for appending PATH to resourcePath" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            dsc --trace-level debug resource list 2> $TestDrive/tracing.txt
                            $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt
                            "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly 'Appending PATH to resourcePath' -Because $testContext
                        }
                        It "can't discover resources outside of DSC exe home or discovery extensions" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            $result = dsc resource list $resourceManifest.type 2> $TestDrive/tracing.txt
                            $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                            $result | Should -BeNullOrEmpty -Because $testContext
                        }
                    }
                    Context "when directories is defined" {
                        BeforeAll {
                            $data.resourcePath.directories = $directoriesWithoutDscHome
                        }

                        It "discovers all builtin resources in DSC exe home" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            $result = dsc resource list 2> $TestDrive/tracing.txt
                            $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt
                            $testContext.exitCode | Should -Be 0 -Because $testContext

                            $builtInInHome = $script:dscBuiltInResources.Where({$_.Path.StartsWith($script:dscHome)})
                            $actualResources = $result | ConvertFrom-Json -Depth 99
                            # Ensure all builtin resources in DSC exe home are returned
                            foreach ($resource in $builtInInHome) {
                                $resource.type | Should -BeIn $actualResources.type -Because (
                                    $resource
                                    | Select-Object -Property type, path, requireAdapter, @{n='condition'; e={$_.manifest.condition}}
                                    | ConvertTo-Json
                                )
                            }
                            # Count must be greater than or equal, depending on resources from discovery extensions
                            $result.Count | Should -BeGreaterOrEqual $builtInInHome.Count -Because $testContext
                        }
                        It "doesn't write a debug message for appending PATH to resourcePath" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            dsc --trace-level debug resource list 2> $TestDrive/tracing.txt
                            $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt
                            "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly 'Appending PATH to resourcePath' -Because $testContext
                        }
                        It "discovers resources in specified directories" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            $result = dsc resource list $resourceManifest.type 2> $TestDrive/tracing.txt
                            $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                            $result | Should -Not -BeNullOrEmpty -Because $testContext
                            $result.Count | Should -Be 1 -Because $testContext
                        }
                        It "allows invoking an executable outside of the specified paths" {
                            Set-SettingsFile -Scope $scope -Settings $data
                            $params = @(
                                'resource', 'get'
                                '--resource', $resourceManifest.type
                                '--input', '{"output": "test"}'
                            )
                            $result = dsc @params 2> $TestDrive/tracing.txt
                            $testContext = New-TestContext -TraceFilePath $TestDrive/tracing.txt -Result $result

                            $testContext.exitCode | Should -Be 0 -Because $testContext
                            $result | Should -Not -BeNullOrEmpty -Because $testContext
                            $resultData = $result | ConvertFrom-Json -Depth 10
                            $resultData.actualState.output | Should -Be 'test' -Because $testContext
                        }
                    }
                }
            }
        }
    }
    Context "settings cascade" {
        BeforeAll {
            $messages = [pscustomobject]@{
                Trace   = "trace message for test"
                Debug   = "debug message for test"
                Info    = "info message for test"
                Warning = "warning message for test"
                Error   = "error message for test"
            }
            $params = @(
                'resource', 'get'
                '--resource', 'Microsoft.DSC.Transitional/PowerShellScript'
                '--input', ([ordered]@{
                    getScript = @(
                        "Write-Information '$($messages.Trace)'"
                        "Write-Debug       '$($messages.Debug)'"
                        "Write-Verbose     '$($messages.Info)'"
                        "Write-Warning     '$($messages.Warning)'"
                        "Write-Error       '$($messages.Error)'"
                    ) -join "`n"
                } | ConvertTo-Json)
            )
            $machinePreferences = @{
                Scope = 'machine'
                Settings = [ordered]@{
                    tracing = [ordered]@{
                        format = 'json'
                    }
                }
            }
            $userPreferences = @{
                Scope = 'user'
                Settings = [ordered]@{
                    tracing = [ordered]@{
                        level = 'trace'
                        format = 'default'
                    }
                }
            }
            $workspacePreferences = @{
                Scope = 'workspace'
                Settings = [ordered]@{
                    tracing = [ordered]@{
                        level = 'debug'
                    }
                }
            }
            $environmentSettings = @{
                TraceLevel = 'info'
            }
            $cliArgs = @('--trace-level', 'error')
            $policySettings = @{
                Scope = 'policy'
                Settings = [ordered]@{
                    tracing = [ordered]@{
                        format = 'json'
                    }
                }
            }
        }

        Context "machine scope" -Skip:$script:skipSystem {
            It 'overrides default settings' {
                Set-SettingsFile @machinePreferences

                dsc @params 2> $TestDrive/tracing.txt
                $testContext = New-TestContext  -TraceFilePath $TestDrive/tracing.txt

                $traces = Get-Content -Path $TestDrive/tracing.txt | ConvertFrom-Json -Depth 10
                $traces.Count | Should -BeGreaterOrEqual 2 -Because $testContext
                $traces.fields.message -join "`n" | Should -MatchExactly $messages.Error -Because $testContext
                $traces.fields.message -join "`n" | Should -MatchExactly $messages.Warning -Because $testContext
            }
        }
        Context 'user scope' {
            It 'overrides default settings' {
                Set-SettingsFile @userPreferences

                dsc @params 2> $TestDrive/tracing.txt
                $testContext = New-TestContext  -TraceFilePath $TestDrive/tracing.txt

                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Error -Because $testContext
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Warning -Because $testContext
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Info -Because $testContext
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Debug -Because $testContext
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Trace -Because $testContext
            }
            It 'overrides machine settings' -Skip:$script:skipSystem {
                Set-SettingsFile @machinePreferences
                Set-SettingsFile @userPreferences

                dsc @params 2> $TestDrive/tracing.txt
                $testContext = New-TestContext  -TraceFilePath $TestDrive/tracing.txt

                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Error -Because $testContext
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Warning -Because $testContext
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Info -Because $testContext
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Debug -Because $testContext
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Trace -Because $testContext
            }
        }
        Context 'workspace scope' {
            It 'overrides default settings' {
                Set-SettingsFile @workspacePreferences

                dsc @params 2> $TestDrive/tracing.txt
                $testContext = New-TestContext  -TraceFilePath $TestDrive/tracing.txt

                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Error -Because $testContext
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Warning -Because $testContext
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Info -Because $testContext
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Debug -Because $testContext
                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $messages.Trace -Because $testContext
            }
            It 'overrides machine scope' -Skip:$script:skipSystem {
                Set-SettingsFile @machinePreferences
                Set-SettingsFile @workspacePreferences

                dsc @params 2> $TestDrive/tracing.txt
                $testContext = New-TestContext  -TraceFilePath $TestDrive/tracing.txt

                $traces = Get-Content -Path $TestDrive/tracing.txt | ConvertFrom-Json -Depth 10
                $traces.Count | Should -BeGreaterOrEqual 4 -Because $testContext
                $traces.fields.message -join "`n" | Should -MatchExactly $messages.Error -Because $testContext
                $traces.fields.message -join "`n" | Should -MatchExactly $messages.Warning -Because $testContext
                $traces.fields.message -join "`n" | Should -MatchExactly $messages.Info -Because $testContext
                $traces.fields.message -join "`n" | Should -MatchExactly $messages.Debug -Because $testContext
            }
            It 'overrides user scope' {
                Set-SettingsFile @userPreferences
                Set-SettingsFile @workspacePreferences

                dsc @params 2> $TestDrive/tracing.txt
                $testContext = New-TestContext  -TraceFilePath $TestDrive/tracing.txt

                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Error -Because $testContext
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Warning -Because $testContext
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Info -Because $testContext
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Debug -Because $testContext
                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $messages.Trace -Because $testContext
            }
        }
        Context 'environment scope' {
            It 'overrides default settings' {
                Set-SettingsEnvironment @environmentSettings

                dsc @params 2> $TestDrive/tracing.txt
                $testContext = New-TestContext  -TraceFilePath $TestDrive/tracing.txt

                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Error -Because $testContext
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Warning -Because $testContext
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Info -Because $testContext
                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $messages.Debug -Because $testContext
                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $messages.Trace -Because $testContext
            }
            It 'overrides machine scope' -Skip:$script:skipSystem {
                Set-SettingsFile @machinePreferences
                Set-SettingsEnvironment @environmentSettings

                dsc @params 2> $TestDrive/tracing.txt
                $testContext = New-TestContext  -TraceFilePath $TestDrive/tracing.txt

                $traces = Get-Content -Path $TestDrive/tracing.txt | ConvertFrom-Json -Depth 10
                $traces.Count | Should -BeGreaterOrEqual 3 -Because $testContext
                $traces.fields.message -join "`n" | Should -MatchExactly $messages.Error -Because $testContext
                $traces.fields.message -join "`n" | Should -MatchExactly $messages.Warning -Because $testContext
                $traces.fields.message -join "`n" | Should -MatchExactly $messages.Info -Because $testContext
                $traces.fields.message -join "`n" | Should -Not -MatchExactly $messages.Debug -Because $testContext
                $traces.fields.message -join "`n" | Should -Not -MatchExactly $messages.Trace -Because $testContext
            }
            It 'overrides user scope' {
                Set-SettingsFile @userPreferences
                Set-SettingsEnvironment @environmentSettings

                dsc @params 2> $TestDrive/tracing.txt
                $testContext = New-TestContext  -TraceFilePath $TestDrive/tracing.txt

                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Error -Because $testContext
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Warning -Because $testContext
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Info -Because $testContext
                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $messages.Debug -Because $testContext
                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $messages.Trace -Because $testContext
            }
            It 'overrides workspace scope' {
                Set-SettingsFile @workspacePreferences
                Set-SettingsEnvironment @environmentSettings

                dsc @params 2> $TestDrive/tracing.txt
                $testContext = New-TestContext  -TraceFilePath $TestDrive/tracing.txt

                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Error -Because $testContext
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Warning -Because $testContext
                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Info -Because $testContext
                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $messages.Debug -Because $testContext
                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $messages.Trace -Because $testContext
            }
        }
        Context 'cli scope' {
            BeforeAll {
                [string[]]$dscParams = $cliArgs + $params
            }
            It 'overrides default settings' {
                dsc @dscParams 2> $TestDrive/tracing.txt
                $testContext = New-TestContext  -TraceFilePath $TestDrive/tracing.txt

                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Error -Because $testContext
                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $messages.Warning -Because $testContext
                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $messages.Info -Because $testContext
                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $messages.Debug -Because $testContext
                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $messages.Trace -Because $testContext
            }
            It 'overrides machine scope' -Skip:$script:skipSystem {
                Set-SettingsFile @machinePreferences

                dsc @dscParams 2> $TestDrive/tracing.txt
                $testContext = New-TestContext  -TraceFilePath $TestDrive/tracing.txt

                $traces = Get-Content -Path $TestDrive/tracing.txt | ConvertFrom-Json -Depth 10
                $traces.Count | Should -BeGreaterOrEqual 3 -Because $testContext
                $traces.fields.message -join "`n" | Should -MatchExactly $messages.Error -Because $testContext
                $traces.fields.message -join "`n" | Should -Not -MatchExactly $messages.Warning -Because $testContext
                $traces.fields.message -join "`n" | Should -Not -MatchExactly $messages.Info -Because $testContext
                $traces.fields.message -join "`n" | Should -Not -MatchExactly $messages.Debug -Because $testContext
                $traces.fields.message -join "`n" | Should -Not -MatchExactly $messages.Trace -Because $testContext
            }
            It 'overrides user scope' {
                Set-SettingsFile @userPreferences

                dsc @dscParams 2> $TestDrive/tracing.txt
                $testContext = New-TestContext  -TraceFilePath $TestDrive/tracing.txt

                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Error -Because $testContext
                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $messages.Warning -Because $testContext
                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $messages.Info -Because $testContext
                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $messages.Debug -Because $testContext
                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $messages.Trace -Because $testContext
            }
            It 'overrides workspace scope' {
                Set-SettingsFile @workspacePreferences

                dsc @dscParams 2> $TestDrive/tracing.txt
                $testContext = New-TestContext  -TraceFilePath $TestDrive/tracing.txt

                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Error -Because $testContext
                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $messages.Warning -Because $testContext
                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $messages.Info -Because $testContext
                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $messages.Debug -Because $testContext
                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $messages.Trace -Because $testContext
            }
            It 'overrides environment scope' {
                Set-SettingsEnvironment @environmentSettings

                dsc @dscParams 2> $TestDrive/tracing.txt
                $testContext = New-TestContext  -TraceFilePath $TestDrive/tracing.txt

                "$TestDrive/tracing.txt" | Should -FileContentMatchExactly $messages.Error -Because $testContext
                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $messages.Warning -Because $testContext
                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $messages.Info -Because $testContext
                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $messages.Debug -Because $testContext
                "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly $messages.Trace -Because $testContext
            }
        }
        Context 'policy scope' -Skip:$script:skipSystem {
            It 'overrides all other scopes' {
                Set-SettingsFile @machinePreferences
                Set-SettingsFile @userPreferences
                Set-SettingsFile @workspacePreferences
                Set-SettingsEnvironment @environmentSettings
                Set-SettingsFile @policySettings

                [string[]]$dscParams = $cliArgs + $params

                dsc @dscParams 2> $TestDrive/tracing.txt
                $testContext = New-TestContext  -TraceFilePath $TestDrive/tracing.txt

                $traces = Get-Content -Path $TestDrive/tracing.txt | ConvertFrom-Json -Depth 10
                $traces.fields.message -join "`n" | Should -MatchExactly $messages.Error -Because $testContext
                $traces.fields.message -join "`n" | Should -Not -MatchExactly $messages.Warning -Because $testContext
                $traces.fields.message -join "`n" | Should -Not -MatchExactly $messages.Info -Because $testContext
                $traces.fields.message -join "`n" | Should -Not -MatchExactly $messages.Debug -Because $testContext
                $traces.fields.message -join "`n" | Should -Not -MatchExactly $messages.Trace -Because $testContext
            }
        }
    }
}
