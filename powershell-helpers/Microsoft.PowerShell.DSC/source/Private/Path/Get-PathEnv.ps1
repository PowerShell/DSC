function Get-PathEnv
{
    [CmdLetBinding(DefaultParameterSetName = "scoped")]
    param
    (
        [Parameter(ParameterSetName = "scoped")]
        [System.Management.Automation.SwitchParameter]
        [System.Boolean]
        $User,

        [Parameter(ParameterSetName = "scoped")] 
        [System.Management.Automation.SwitchParameter]
        [System.Boolean]
        $Machine, 

        [Alias("process")]
        [Parameter(ParameterSetName = "scoped")]
        [System.Management.Automation.SwitchParameter]
        [System.Boolean]
        $Current, 

        [Parameter(ParameterSetName = "all")]
        [System.Management.Automation.SwitchParameter]
        [System.Boolean]
        $All
    )
     
    $scopespecified = $user.IsPresent -or $machine.IsPresent -or $current.IsPresent
    $path = @()
    $userpath = get-envvar "PATH" -user 
    if ($user)
    {
        $path += $userpath
    }
    $machinepath = get-envvar "PATH" -machine
    if ($machine -or !$scopespecified)
    {
        $path += $machinepath
    }
    if (!$user.IsPresent -and !$machine.IsPresent)
    {
        $current = $true
    }
    $currentPath = get-envvar "PATH" -current
    if ($current)
    {
        $path = $currentPath
    }
        
    if ($all)
    {
        $h = @{
            user    = $userpath
            machine = $machinepath
            process = $currentPath
        }
        return @(
            "`r`n USER",
            " -----------",
            $h.user, 
            "`r`n MACHINE",
            " -----------",
            $h.machine, 
            "`r`n PROCESS",
            " -----------",
            $h.process
        )
    }
        
    return $path
}