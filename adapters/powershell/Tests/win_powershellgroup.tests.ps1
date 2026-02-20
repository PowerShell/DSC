# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

BeforeDiscovery {
    if ($IsWindows) {
        $identity = [System.Security.Principal.WindowsIdentity]::GetCurrent()
        $principal = [System.Security.Principal.WindowsPrincipal]::new($identity)
        $isElevated = $principal.IsInRole([System.Security.Principal.WindowsBuiltInRole]::Administrator)
    }
}

Describe 'WindowsPowerShell adapter resource tests - requires elevated permissions' -Skip:(!$IsWindows -or !$isElevated) {

  BeforeAll {
    $OldPSModulePath = $env:PSModulePath
    $dscHome = Split-Path (Get-Command dsc -ErrorAction Stop).Source -Parent
    $psexeHome = Split-Path (Get-Command powershell -ErrorAction Stop).Source -Parent
    $ps7exeHome = Split-Path (Get-Command pwsh -ErrorAction Stop).Source -Parent
    $env:DSC_RESOURCE_PATH = $dscHome + [System.IO.Path]::PathSeparator + $psexeHome + [System.IO.Path]::PathSeparator + $ps7exeHome
    $windowsPowerShellPath = Join-Path $testDrive 'WindowsPowerShell' 'Modules'
    $moduleFile = @"
@{
    RootModule           = 'PSClassResource.psm1'
    ModuleVersion        = '0.1.0'
    GUID                 = '1b2e177b-1819-4f51-8bc9-795dd8fae984'
    Author               = 'Microsoft Corporation'
    CompanyName          = 'Microsoft Corporation'
    Copyright            = '(c) Microsoft Corporation. All rights reserved.'
    Description          = 'DSC Resource for Windows PowerShell Class'
    PowerShellVersion    = '5.1'
    DscResourcesToExport = @(
        'PSClassResource'
    )
    PrivateData          = @{
        PSData = @{
            Tags       = @(
                'PSDscResource_PSClassResource'
            )
            DscCapabilities = @(
            'get'
            'test'
            'set'
            'export'
            )
        }
    }
}
"@
    $moduleFilePath = Join-Path $windowsPowerShellPath 'PSClassResource' '0.1.0' 'PSClassResource.psd1'
    if (-not (Test-Path -Path $moduleFilePath)) {
      New-Item -Path $moduleFilePath -ItemType File -Value $moduleFile -Force | Out-Null
    }

    $module = @'
enum Ensure {
    Present
    Absent
}

[DSCResource()]
class PSClassResource {
    [DscProperty(Key)]
    [string] $Name

    [string] $NonDscProperty

    hidden
    [string] $HiddenNonDscProperty

    [DscProperty()]
    [Ensure] $Ensure = [Ensure]::Present

    [DscProperty()]
    [PSCredential] $Credential

    PSClassResource() {
    }

    [PSClassResource] Get() {
        return $this
    }

    [bool] Test() {
        return $true
    }

    [void] Set() {
        if ($null -eq $this.Credential) {
          throw 'Credential property is required'
        }

        if ($this.Credential.UserName -ne 'MyUser') {
            throw 'Invalid user name'
        }
    }

    static [PSClassResource[]] Export()
    {
        $resultList = [System.Collections.Generic.List[PSClassResource]]::new()
        $resultCount = 5
        if ($env:PSClassResourceResultCount) {
            $resultCount = $env:PSClassResourceResultCount
        }
        1..$resultCount | %{
            $obj = New-Object PSClassResource
            $obj.Name = "Object$_"
            $obj.Ensure = [Ensure]::Present
            $resultList.Add($obj)
        }

        return $resultList.ToArray()
    }
}
'@

    $modulePath = Join-Path $windowsPowerShellPath 'PSClassResource' '0.1.0' 'PSClassResource.psm1'
    if (-not (Test-Path -Path $modulePath)) {
      New-Item -Path $modulePath -ItemType File -Value $module -Force | Out-Null
    }

    ## Add script base Classs for testing and credential object

    $moduleFileScriptRootPSD1 = @"
@{
        # Script module or binary module file associated with this manifest.
        RootModule            = 'TestScriptBaseDSC.psm1'

        # Version number of this module.
        moduleVersion        = '0.0.1'

        # ID used to uniquely identify this module
        GUID                 = 'c3775be8-84a1-43f5-a99c-1b9f2d6bc178'

        # Author of this module
        Author               = ''

        # Company or vendor of this module
        CompanyName          = ''

        # Copyright statement for this module
        Copyright            = ''

        # Description of the functionality provided by this module
        Description          = ''

        # Minimum version of the Windows PowerShell engine required by this module
        PowerShellVersion    = '5.0'

        # Cmdlets to export from this module
        CmdletsToExport      = @()

        # Variables to export from this module
        VariablesToExport    = @()

        # Aliases to export from this module
        AliasesToExport      = @()

        # Dsc Resources to export from this module
        DscResourcesToExport = @('CredentialValidation')

        # Private data to pass to the module specified in RootModule/ModuleToProcess. This may also contain a PSData hashtable with additional module metadata used by PowerShell.
        PrivateData          = @{

            PSData = @{

                # Tags applied to this module. These help with module discovery in online galleries.
                Tags         = @('DesiredStateConfiguration', 'DSC', 'DSCResourceKit', 'DSCResource')

                # A URL to the license for this module.
                LicenseUri   = ''

                # A URL to the main website for this project.
                ProjectUri   = ''

                # A URL to an icon representing this module.
                IconUri      = ''

                # ReleaseNotes of this module
                ReleaseNotes = ''

                # Set to a prerelease string value if the release should be a prerelease.
                Prerelease   = ''
            } # End of PSData hashtable
        } # End of PrivateData hashtable
    }
"@

    $moduleScriptRootPSM1 = @"
    # Root module for CredentialValidationDsc
    # No code required
"@

    $moduleScriptCredentialValidationPSM1 = @'


    $VerbosePreference = 'SilentlyContinue'
    $InformationPreference = 'SilentlyContinue'
    $ProgressPreference = 'Continue'
    $ErrorActionPreference = 'SilentlyContinue'

    function Get-TargetResource {
        [System.Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSAvoidGlobalVars', '')]
        [OutputType([Hashtable])]
        param (
            [Parameter(Mandatory)]
            [string] $Name,

            [Parameter(Mandatory = $true)]
            [System.Management.Automation.PSCredential]
            $Credential
        )
        Write-Verbose "[GET] Get Function running"
        return @{
                Name = $Name
                Credential = $Credential
        }
      
    }

    function Test-TargetResource {
        [System.Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSAvoidGlobalVars', '')]
        [OutputType([System.Boolean])]
        param (
            [Parameter(Mandatory)]
            [string] $Name,

            [Parameter(Mandatory = $true)]
            [System.Management.Automation.PSCredential]
            $Credential

        )
        Write-Verbose "[TEST]Checking credentials"
        Write-Verbose "[TEST]Checking credentials UserName:  $($Credential.UserName)"
        Write-Verbose "[TEST]Checking credentials Password:  $($Credential.Password)"

      if ($null -eq $Credential) {
              throw 'Credential property is required'
              $inDesiredState = $false
              return $false
            }

        if ($Credential.UserName -ne 'MyUser') {
                throw 'Invalid user name'
                $inDesiredState = $false
        } else {
                $inDesiredState = $true
        }


        return $inDesiredState
        
    }

    function Set-TargetResource {
        [System.Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSAvoidGlobalVars', '')]
        [CmdletBinding()]
        param (
            [Parameter(Mandatory)]
            [string] $Name,

            [Parameter(Mandatory = $true)]
            [System.Management.Automation.PSCredential]
            $Credential

        )

          if ($null -eq $Credential) {
              throw 'Credential property is required'
              $inDesiredState = $false
              return $false
            }

            if ($Credential.UserName -ne 'MyUser') {
                    throw 'Invalid user name'
                    $inDesiredState = $false
            } else {
                    $inDesiredState = $true
            }


            return $inDesiredState
        Write-Verbose "[SET]Credential cannot be remediated by DSC."
    }

'@

    $moduleScriptCredentialValidationSchemaMof = @"
    [ClassVersion("1.0.0.0"), FriendlyName("CredentialValidation")]
    class CredentialValidation : OMI_BaseResource
    {
        [Key] string Name;
        [Required, Description("Test Credentials for Script Base"), EmbeddedInstance("MSFT_Credential")] String Credential;
    };

"@



    $modulePathRootPSM1 = Join-Path $windowsPowerShellPath 'TestScriptBaseDSC' '0.0.1' 'TestScriptBaseDSC.psm1'
        if (-not (Test-Path -Path $modulePathRootPSM1)) {
        New-Item -Path $modulePathRootPSM1 -ItemType File -Value $moduleScriptRootPSM1 -Force | Out-Null
    }


    $modulePathRootPSD1 = Join-Path $windowsPowerShellPath 'TestScriptBaseDSC' '0.0.1' 'TestScriptBaseDSC.psd1'
        if (-not (Test-Path -Path $modulePathRootPSD1)) {
        New-Item -Path $modulePathRootPSD1 -ItemType File -Value $moduleFileScriptRootPSD1 -Force | Out-Null
    }


    $modulePathScriptCredentialValidationPSM1 = Join-Path $windowsPowerShellPath 'TestScriptBaseDSC' '0.0.1' 'DSCResources' 'CredentialValidation' 'CredentialValidation.psm1'
    if (-not (Test-Path -Path $modulePathScriptCredentialValidationPSM1)) {
        Write-Host "File will be created: $modulePathScriptCredentialValidationPSM1"
        New-Item -Path $modulePathScriptCredentialValidationPSM1 -ItemType File -Value $moduleScriptCredentialValidationPSM1 -Force | Out-Null
    }

    $modulePathScriptCredentialValidationSchemaMof = Join-Path $windowsPowerShellPath 'TestScriptBaseDSC' '0.0.1' 'DSCResources' 'CredentialValidation' 'CredentialValidation.schema.mof'
    if (-not (Test-Path -Path $modulePathScriptCredentialValidationSchemaMof)) {
        Write-Host "File will be created: $modulePathScriptCredentialValidationSchemaMof"
        New-Item -Path $modulePathScriptCredentialValidationSchemaMof -ItemType File -Value $moduleScriptCredentialValidationSchemaMof -Force | Out-Null
    }

    $env:PSModulePath = $windowsPowerShellPath + [System.IO.Path]::PathSeparator + $env:PSModulePath + [System.IO.Path]::PathSeparator
  }

  AfterAll {
    $env:PSModulePath = $OldPSModulePath
    $env:DSC_RESOURCE_PATH = $null
  }

  It '_inDesiredState is returned correction: <Context>' -TestCases @(
    @{ Context = 'Both running'; FirstState = 'Running'; SecondState = 'Running' }
    @{ Context = 'Both stopped'; FirstState = 'Stopped'; SecondState = 'Stopped' }
    @{ Context = 'First Stopped'; FirstState = 'Stopped'; SecondState = 'Running' }
    @{ Context = 'First Running'; FirstState = 'Running'; SecondState = 'Stopped' }
  ) {
    param($Context, $FirstState, $SecondState)
    $yaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Use Windows PowerShell resources
    type: Microsoft.Windows/WindowsPowerShell
    properties:
      resources:
      - name: Check Spooler service 1
        type: PsDesiredStateConfiguration/Service
        properties:
          Name: Spooler
          State: $FirstState
      - name: Check Spooler service 2
        type: PsDesiredStateConfiguration/Service
        properties:
          Name: Spooler
          State: $SecondState
"@

    $inDesiredState = if ($FirstState -eq $SecondState) {
      $FirstState -eq (Get-Service Spooler).Status
    }
    else {
      $false
    }

    $out = dsc -l trace config test -i $yaml 2>"$testdrive/error.log" | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path "$testdrive/error.log" -Raw | Out-String)
    $out.results[0].result.inDesiredState | Should -Be $inDesiredState
  }

  It 'Config works with credential object' {
  $yaml = @'
   $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
   resources:
     - name: Cred test
       type: PSClassResource/PSClassResource
       properties:
         Name: Test
         Credential:
           UserName: 'MyUser'
           Password: 'MyPassword'
'@

    $out = dsc -l debug config set -i $yaml 2> "$testdrive/error.log" | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path "$testdrive/error.log" -Raw | Out-String)
  }

  It 'Config does not work when credential properties are missing required fields' {
    $yaml = @'
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
        - name: Service info
          type: PsDesiredStateConfiguration/Service
          properties:
            Name: Spooler
            Credential:
              UserName: 'User'
              OtherProperty: 'Password'
'@
    # Compared to PowerShell we use test here as it filters out the properties
    $out = dsc -l debug config test -i $yaml 2> "$testdrive/error.log" | Out-String
    $LASTEXITCODE | Should -Be 2
    $out | Should -BeNullOrEmpty
    (Get-Content -Path "$testdrive/error.log" -Raw) | Should -BeLike "*ERROR*Credential object 'Credential' requires both 'username' and 'password' properties*" -Because (Get-Content -Path "$testdrive/error.log" -Raw | Out-String)
  }

  ## Scipt base resources test running

It 'Config works with credential object Script base resources' {

$inDesiredState = $true

$yaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Working with classic DSC resources
  type: Microsoft.Windows/WindowsPowerShell
  properties:
    resources:
    - name: Script-resource Info
      type: TestScriptBaseDSC/CredentialValidation
      properties:
        Name: TestScriptResource1
        Credential:       
           UserName: MyUser
           Password: Password
'@

$out = dsc -l trace config test -i $yaml 2>"$testdrive/error.log" | ConvertFrom-Json
$LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path "$testdrive/error.log" -Raw | Out-String)
$out.results[0].result.inDesiredState | Should -Be $inDesiredState
}


# This works
It 'Not Valide credentials with Script base resources - wrong properties' {


$yaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Working with classic DSC resources
  type: Microsoft.Windows/WindowsPowerShell
  properties:
    resources:
    - name: Script-resource Info
      type: TestScriptBaseDSC/CredentialValidation
      properties:
        Name: TestScriptResource1
        Credential:       
           username: MyUser
           Notpassword: Password
'@

$out = dsc -l trace config test -i $yaml 2>"$testdrive/error.log" | Out-String
$LASTEXITCODE | Should -Be 2
$out | Should -BeNullOrEmpty
(Get-Content -Path "$testdrive/error.log" -Raw) | Should -BeLike "*ERROR*Credential object 'Credential' requires both 'username' and 'password' properties*" -Because (Get-Content -Path "$testdrive/error.log" -Raw | Out-String)

}



  It 'List works with class-based PS DSC resources' {
    $out = dsc resource list --adapter Microsoft.Windows/WindowsPowerShell 2> "$testdrive/error.log" | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path "$testdrive/error.log" -Raw | Out-String)
    $out.type | Should -Contain 'PSClassResource/PSClassResource' -Because ($out.type | Out-String)
    $out | Where-Object -Property type -EQ PSClassResource/PSClassResource | Select-Object -ExpandProperty implementedAs | Should -Be 1
    ($out | Where-Object -Property type -EQ 'PSClassResource/PSClassResource').capabilities | Should -BeIn @('get', 'test', 'set', 'export')
  }

  It 'Get works with class-based PS DSC resources' {
    $out = dsc resource get -r PSClassResource/PSClassResource --input (@{Name = 'TestName' } | ConvertTo-Json) 2> "$testdrive/error.log" | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path "$testdrive/error.log" -Raw | Out-String)
    $out.actualState.Name | Should -Be 'TestName'
    $out.actualState.Ensure | Should -Be 'Present'
    $propCount = $out.actualState | Get-Member -MemberType NoteProperty
    $propCount.Count | Should -Be 3 -Because ($out | Out-String)
  }

  It 'Set works with class-based PS DSC resources' {
    $out = dsc resource set -r PSClassResource/PSClassResource --input (@{Name = 'TestName'; Credential = @{"UserName" = "MyUser"; "Password" = "MyPassword"} } | ConvertTo-Json) 2> "$testdrive/error.log" | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path "$testdrive/error.log" -Raw | Out-String)
    $out.changedProperties.Count | Should -Be 0 -Because ($out | ConvertTo-Json -Depth 10 | Out-String)
  }

  It 'Export works with class-based PS DSC resources' {
    $out = dsc -l trace resource export -r PSClassResource/PSClassResource 2> "$testdrive/error.log" | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path "$testdrive/error.log" -Raw | Out-String)
    $out | Should -Not -BeNullOrEmpty
    $out.resources.count | Should -Be 5
    $out.resources[0].properties.Ensure | Should -Be 'Present' # Check for enum property
  }

  It 'Config calling PS Resource directly works for <operation> with metadata <metadata> and adapter <adapter>' -TestCases @(
    @{ Operation = 'get'; metadata = 'Microsoft.DSC'; adapter = 'Microsoft.Windows/WindowsPowerShell' }
    @{ Operation = 'set'; metadata = 'Microsoft.DSC'; adapter = 'Microsoft.Windows/WindowsPowerShell' }
    @{ Operation = 'test'; metadata = 'Microsoft.DSC'; adapter = 'Microsoft.Windows/WindowsPowerShell' }
    @{ Operation = 'get'; metadata = 'Microsoft.DSC'; adapter = 'Microsoft.Adapter/WindowsPowerShell' }
    @{ Operation = 'set'; metadata = 'Microsoft.DSC'; adapter = 'Microsoft.Adapter/WindowsPowerShell' }
    @{ Operation = 'test'; metadata = 'Microsoft.DSC'; adapter = 'Microsoft.Adapter/WindowsPowerShell' }
    @{ Operation = 'get'; metadata = 'Ignored' }
    @{ Operation = 'set'; metadata = 'Ignored' }
    @{ Operation = 'test'; metadata = 'Ignored' }
  ) {
    param($Operation, $metadata, $adapter)

    $yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Class-resource Info
              type: PSClassResource/PSClassResource
              metadata:
                ${metadata}:
                  requireAdapter: $adapter
              properties:
                Name: TestInstance
                Credential:
                  UserName: 'MyUser'
                  Password: 'MyPassword'
"@
    $out = dsc -l trace config $operation -i $yaml 2> $TestDrive/tracing.txt
    $text = $out | Out-String
    $out = $out | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw -Path $TestDrive/tracing.txt)
    switch ($Operation) {
      'get' {
        $out.results[0].result.actualState.Name | Should -BeExactly 'TestInstance' -Because ("$text`n" + (Get-Content -Raw -Path $TestDrive/tracing.txt))
      }
      'set' {
        $out.results[0].result.beforeState.Name | Should -BeExactly 'TestInstance' -Because $text
        if ($adapter -eq 'Microsoft.Adapter/WindowsPowerShell') {
          # the `single` mode of the adapter performs a `get` after `set` and returns that result so we can validate it
          $out.results[0].result.afterState.Name | Should -BeExactly 'TestInstance' -Because $text
        }
      }
      'test' {
        $out.results[0].result.inDesiredState | Should -BeTrue -Because $text
      }
    }
    if ($metadata -eq 'Microsoft.DSC') {
      "$TestDrive/tracing.txt" | Should -FileContentMatch "Invoking $Operation for '$adapter'" -Because (Get-Content -Raw -Path $TestDrive/tracing.txt)
    }
    if ($adapter -eq 'Microsoft.Windows/WindowsPowerShell') {
      (Get-Content -Raw -Path $TestDrive/tracing.txt) | Should -Match "Resource 'Microsoft.Windows/WindowsPowerShell' is deprecated" -Because (Get-Content -Raw -Path $TestDrive/tracing.txt)
    }
  }
}

