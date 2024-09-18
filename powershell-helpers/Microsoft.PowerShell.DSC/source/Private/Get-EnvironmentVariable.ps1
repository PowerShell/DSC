function Get-EnvironmentVariable
{
    [CmdletBinding()]
    param 
    (
        [Parameter(Mandatory = $true)]
        [String]
        $Name,

        [Parameter()]
        [System.EnvironmentVariableTarget]
        $Scope = [System.EnvironmentVariableTarget]::Process,

        [Parameter()]
        [Switch]
        $Expanded,

        [Parameter()]
        [Switch]
        $ShowProperties
    )
    
    $Getter = [System.Environment]::GetEnvironmentVariable($Name, $Scope)
    if ($null -eq $Getter)
    {
        $RawValue = $null
        $GetterType = $null
    }
    else
    {
        if ($Scope -ne "Process")
        {
            if (!$Expanded)
            {
                $AllEnvironmentVariables = Get-Item -Path (Get-EnvironmentPath -Scope $Scope)
                $GetterType = $AllEnvironmentVariables.GetValueKind($Name)
            }
            else
            {
                $AllEnvironmentVariables = [System.Environment]::GetEnvironmentVariables($Scope)
                $GetterType = $Getter.GetTypeCode()
            }
            if ($GetterType -eq "ExpandString")
            {
                $RawValue = $AllEnvironmentVariables.GetValue(
                    $Name, $null, 'DoNotExpandEnvironmentNames'
                )
            }
            elseif ($GetterType -eq "String")
            {
                $RawValue = $Getter
                if ($Expanded)
                {
                    $Getter = [System.Environment]::ExpandEnvironmentVariables($Getter)
                }
            }
            else
            {
                # inappropriate kind (dword, bytes, ...)
                $RawValue = $null
                $GetterType = $null
            }
        }
        else
        {
            # $Scope -eq "Process"
            $RawValue = $null
            $GetterType = "String"
        }
    }
    $params = @{
        Name            = $Name
        Value           = $Getter
        Scope           = $Scope
        ValueType       = $GetterType
        BeforeExpansion = $RawValue
    }
    $null = New-EnvironmentVariableObject @params | Set-Variable -Name NewEnvVar

    if ($ShowProperties)
    {
        $NewEnvVar | Add-Member ScriptMethod ToString { $this.Value } -Force -PassThru
    }
    else
    {
        if (!$Expanded)
        {
            $NewEnvVar | Add-Member ScriptMethod ToString { $this.Value } -Force -PassThru | Select-Object -ExpandProperty BeforeExpansion
        }
        else
        {
            $NewEnvVar | Add-Member ScriptMethod ToString { $this.Value } -Force -PassThru | Select-Object -ExpandProperty Value
        }
    }
}