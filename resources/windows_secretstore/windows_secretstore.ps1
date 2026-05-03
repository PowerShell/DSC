# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

<#
.SYNOPSIS
    DSC v3 resource script for managing Microsoft.PowerShell.SecretStore configuration.

.DESCRIPTION
    Implements Get, Set, and Test operations for the SecretStore vault configuration.
    Requires the Microsoft.PowerShell.SecretStore module to be installed.

.PARAMETER Operation
    The DSC operation to perform: Get, Set, or Test.

.PARAMETER jsonInput
    JSON string received via pipeline containing the desired state properties.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0)]
    [ValidateSet('Get', 'Set', 'Test')]
    [string]$Operation,

    [Parameter(Mandatory = $true, Position = 1, ValueFromPipeline = $true)]
    [string]$jsonInput
)

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

function Write-DscTrace {
    param(
        [Parameter(Mandatory = $true)]
        [ValidateSet('Error', 'Warn', 'Info', 'Debug', 'Trace')]
        [string]$Level,

        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        [string]$Message
    )

    $trace = @{ $Level.ToLower() = $Message } | ConvertTo-Json -Compress
    $host.ui.WriteErrorLine($trace)
}

function Assert-ModuleAvailable {
    param([string]$ModuleName)

    if (-not (Get-Module -ListAvailable -Name $ModuleName -ErrorAction SilentlyContinue |
              Select-Object -First 1)) {
        Write-DscTrace -Level Error -Message (
            "Required module '$ModuleName' is not installed. " +
            "Install it with: Install-Module -Name $ModuleName -Repository PSGallery -Force"
        )
        exit 1
    }
}

function Test-IsNonInteractiveSession {
    <#
    .SYNOPSIS
        Detects if the current PowerShell process was started with -NonInteractive.
    #>
    try {
        $commandLineArgs = [Environment]::GetCommandLineArgs()
        foreach ($arg in $commandLineArgs) {
            if ($arg -ieq '-NonInteractive') {
                return $true
            }
        }
    }
    catch {
        # If detection fails, default to interactive assumptions.
    }

    return $false
}

function ConvertTo-SecretStoreSecureString {
    param(
        [AllowNull()]
        [AllowEmptyString()]
        [object]$Value
    )

    if ($null -eq $Value) {
        return $null
    }

    if ($Value -is [System.Security.SecureString]) {
        return $Value
    }

    if ($Value -is [System.Collections.IDictionary] -and $Value.Contains('secureString')) {
        $Value = $Value['secureString']
    }
    elseif ($Value.PSObject.Properties['secureString']) {
        $Value = $Value.secureString
    }

    $plaintext = [string]$Value
    if ([string]::IsNullOrEmpty($plaintext)) {
        return $null
    }

    return (ConvertTo-SecureString -String $plaintext -AsPlainText -Force)
}

function Get-CurrentState {
    <#
    .SYNOPSIS
        Returns a hashtable representing the current SecretStore configuration.
    #>
    param(
        [switch]$SuppressNonInteractiveError,

        [AllowNull()]
        [System.Security.SecureString]$Password
    )

    if ($null -ne $Password) {
        try {
            Unlock-SecretStore -Password $Password -ErrorAction Stop | Out-Null
        }
        catch {
            Write-DscTrace -Level Error -Message "Failed to unlock SecretStore with the provided password: $_"
            exit 1
        }
    }

    try {
        $config = Get-SecretStoreConfiguration -ErrorAction Stop
        return [ordered]@{
            authentication  = $config.Authentication.ToString()
            passwordTimeout = [int]$config.PasswordTimeout
            interaction     = $config.Interaction.ToString()
            scope           = $config.Scope.ToString()
        }
    }
    catch {
        if ($_.ToString() -match 'NonInteractive mode|require interactive input') {
            if ($SuppressNonInteractiveError) {
                return [ordered]@{
                    authentication             = 'None'
                    passwordTimeout            = 900
                    interaction                = 'None'
                    scope                      = 'CurrentUser'
                    requiresInteractiveInput   = $true
                }
            }

            Write-DscTrace -Level Error -Message (
                "SecretStore is configured to require interactive input. " +
                "This DSC resource runs PowerShell with -NonInteractive, so prompts are not allowed. " +
                "Reconfigure SecretStore in an interactive session first, for example: " +
                "Set-SecretStoreConfiguration -Authentication None -Interaction None -PasswordTimeout -1 -Confirm:`$false"
            )
            exit 1
        }

        Write-DscTrace -Level Error -Message "Failed to retrieve SecretStore configuration: $_"
        exit 1
    }
}

function Ensure-SecretStoreVaultRegistered {
    <#
    .SYNOPSIS
        Ensures the SecretStore vault is registered before configuration changes.
    #>
    try {
        $vault = Get-SecretVault -Name 'SecretStore' -ErrorAction SilentlyContinue
        if ($null -eq $vault) {
            Register-SecretVault -Name 'SecretStore' -ModuleName 'Microsoft.PowerShell.SecretStore' -DefaultVault -ErrorAction Stop
            Write-DscTrace -Level Info -Message 'Registered SecretStore vault.'
        }
    }
    catch {
        Write-DscTrace -Level Error -Message "Failed to register SecretStore vault: $_"
        exit 1
    }
}

# ---------------------------------------------------------------------------
# Prerequisites
# ---------------------------------------------------------------------------

Assert-ModuleAvailable -ModuleName 'Microsoft.PowerShell.SecretStore'

try {
    Import-Module Microsoft.PowerShell.SecretStore -ErrorAction Stop
}
catch {
    Write-DscTrace -Level Error -Message "Failed to import Microsoft.PowerShell.SecretStore: $_"
    exit 1
}

# ---------------------------------------------------------------------------
# Parse input
# ---------------------------------------------------------------------------

$desired = $null
try {
    $desired = $jsonInput | ConvertFrom-Json -AsHashtable -ErrorAction Stop
}
catch {
    Write-DscTrace -Level Error -Message "Failed to parse JSON input: $_"
    exit 1
}

if ($null -eq $desired) {
    $desired = @{}
}

# ---------------------------------------------------------------------------
# Operations
# ---------------------------------------------------------------------------

switch ($Operation) {
    'Get' {
        try {
            $suppressNonInteractiveError = Test-IsNonInteractiveSession
            $password = $null
            if ($desired.ContainsKey('password')) {
                $password = ConvertTo-SecretStoreSecureString -Value $desired['password']
            }

            Get-CurrentState -SuppressNonInteractiveError:$suppressNonInteractiveError -Password $password | ConvertTo-Json -Compress
        }
        catch {
            Write-DscTrace -Level Error -Message "Get operation failed: $_"
            exit 1
        }
    }

    'Set' {
        try {
            $setParams = @{ Confirm = $false }
            $password = $null

            if ($desired.ContainsKey('password')) {
                $password = ConvertTo-SecretStoreSecureString -Value $desired['password']
                if ($null -eq $password) {
                    Write-DscTrace -Level Error -Message 'The password property was provided but is empty. Provide a non-empty SecureString value.'
                    exit 1
                }

                $setParams['Password'] = $password
                $setParams['Authentication'] = 'Password'
            }

            if ($desired.ContainsKey('authentication') -and -not $setParams.ContainsKey('Authentication')) {
                $setParams['Authentication'] = $desired['authentication']
            }
            if ($desired.ContainsKey('passwordTimeout')) { $setParams['PasswordTimeout'] = [int]$desired['passwordTimeout'] }
            if ($desired.ContainsKey('interaction'))     { $setParams['Interaction']     = $desired['interaction'] }
            if ($desired.ContainsKey('scope'))           { $setParams['Scope']           = $desired['scope'] }

            if ($setParams.ContainsKey('Authentication') -and $setParams['Authentication'] -eq 'Password' -and -not $setParams.ContainsKey('Password')) {
                Write-DscTrace -Level Error -Message (
                    'Authentication was set to Password but no password property was provided. Supply password as a DSC SecureString parameter.'
                )
                exit 1
            }

            if ($setParams.Count -eq 1) {
                # Only Confirm was in params - nothing to change
                Write-DscTrace -Level Info -Message 'No configurable properties specified; nothing to set.'
            }
            else {
                Ensure-SecretStoreVaultRegistered
                try {
                    Set-SecretStoreConfiguration @setParams -ErrorAction Stop
                }
                catch {
                    if ($_.ToString() -match 'NonInteractive mode|require interactive input') {
                        # If SecretStore requires prompts, reset it with the desired settings so DSC can proceed unattended.
                        $resetParams = @{
                            Force   = $true
                            Confirm = $false
                        }

                        if ($setParams.ContainsKey('Authentication'))  { $resetParams['Authentication']  = $setParams['Authentication'] }
                        if ($setParams.ContainsKey('Password'))        { $resetParams['Password']        = $setParams['Password'] }
                        if ($setParams.ContainsKey('PasswordTimeout')) { $resetParams['PasswordTimeout'] = $setParams['PasswordTimeout'] }
                        if ($setParams.ContainsKey('Interaction'))     { $resetParams['Interaction']     = $setParams['Interaction'] }
                        if ($setParams.ContainsKey('Scope'))           { $resetParams['Scope']           = $setParams['Scope'] }

                        Write-DscTrace -Level Warn -Message (
                            'SecretStore requires interactive input; attempting Reset-SecretStore with desired settings to enable unattended DSC execution.'
                        )
                        Reset-SecretStore @resetParams -ErrorAction Stop
                    }
                    else {
                        throw
                    }
                }
                Write-DscTrace -Level Info -Message 'SecretStore configuration updated successfully.'
            }

            # Return the resulting state without surfacing interactive prompts in DSC's noninteractive host.
            $suppressNonInteractiveError = Test-IsNonInteractiveSession
            Get-CurrentState -SuppressNonInteractiveError:$suppressNonInteractiveError -Password $password | ConvertTo-Json -Compress
        }
        catch {
            if ($_.ToString() -match 'NonInteractive mode|require interactive input') {
                Write-DscTrace -Level Error -Message (
                    "Set operation requires interactive input with the current SecretStore settings. " +
                    "Run this once in an interactive PowerShell session to allow unattended DSC runs: " +
                    "Set-SecretStoreConfiguration -Authentication None -Interaction None -PasswordTimeout -1 -Confirm:`$false"
                )
                exit 1
            }

            Write-DscTrace -Level Error -Message "Set operation failed: $_"
            exit 1
        }
    }

    'Test' {
        try {
            $password = $null
            if ($desired.ContainsKey('password')) {
                $password = ConvertTo-SecretStoreSecureString -Value $desired['password']
            }

            $current        = Get-CurrentState -SuppressNonInteractiveError -Password $password
            $inDesiredState = $true
            $normalizedDesiredAuthentication = $null

            if ($desired.ContainsKey('password')) {
                $normalizedDesiredAuthentication = 'Password'
            }
            elseif ($desired.ContainsKey('authentication')) {
                $normalizedDesiredAuthentication = $desired['authentication']
            }

            if ($current['requiresInteractiveInput']) {
                Write-DscTrace -Level Info -Message (
                    'SecretStore currently requires interactive input, so it is not in the desired state for unattended DSC execution.'
                )
                $inDesiredState = $false
            }

            $propertyMap = @{
                authentication  = 'authentication'
                passwordTimeout = 'passwordTimeout'
                interaction     = 'interaction'
                scope           = 'scope'
            }

            foreach ($key in $propertyMap.Keys) {
                $hasDesiredValue = $desired.ContainsKey($key)
                $desiredValue = $null

                if ($key -eq 'authentication' -and $null -ne $normalizedDesiredAuthentication) {
                    $hasDesiredValue = $true
                    $desiredValue = $normalizedDesiredAuthentication
                }
                elseif ($hasDesiredValue) {
                    $desiredValue = $desired[$key]
                }

                if ($hasDesiredValue) {
                    $currentValue = $current[$key]

                    if ($current['requiresInteractiveInput']) {
                        continue
                    }

                    # Normalize integer comparison
                    if ($key -eq 'passwordTimeout') {
                        $desiredValue = [int]$desiredValue
                        $currentValue = [int]$currentValue
                    }

                    if ($currentValue -ne $desiredValue) {
                        Write-DscTrace -Level Info -Message (
                            "Property '$key' is not in desired state. " +
                            "Current: '$currentValue', Desired: '$desiredValue'."
                        )
                        $inDesiredState = $false
                    }
                }
            }

            $current['_inDesiredState'] = $inDesiredState
            $current | ConvertTo-Json -Compress
        }
        catch {
            Write-DscTrace -Level Error -Message "Test operation failed: $_"
            exit 1
        }
    }
}
