function Get-EnvVar 
{
    [CmdletBinding()]
    param 
    (
        [Parameter(Mandatory = $true)]
        [System.String]
        $Name,

        [System.Management.Automation.SwitchParameter]
        [System.Boolean]
        $User,

        [System.Management.Automation.SwitchParameter]
        [System.Boolean]
        $Machine,

        [System.Management.Automation.SwitchParameter]
        [System.Boolean]
        $Current
    )

    $val = @()
    if ($user)
    {
        $val += [System.Environment]::GetEnvironmentVariable($name, [System.EnvironmentVariableTarget]::User);
    }
    if ($machine)
    {
        $val += [System.Environment]::GetEnvironmentVariable($name, [System.EnvironmentVariableTarget]::Machine);
    }
    if (!$user.IsPresent -and !$machine.IsPresent)
    {
        $current = $true
    }
    if ($current)
    {
        $val = invoke-expression "`$env:$name"
    }
    if ($val -ne $null)
    {
        $p = $val.Split(';')
    }
    else
    {
        $p = @()
    }
    
    return $p
}