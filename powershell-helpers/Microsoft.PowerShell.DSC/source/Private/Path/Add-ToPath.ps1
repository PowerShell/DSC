function Add-ToPath
{
    [CmdletBinding()]
    param
    (
        [Parameter(ValueFromPipeline = $true, mandatory = $true)]
        [System.String]
        $Path, 
        
        [Alias("p")]
        [System.Boolean]
        [switch]
        $Persistent, 
        
        [switch]
        [System.Boolean]
        $First,
        
        [System.Management.Automation.SwitchParameter]
        [System.Boolean]
        $User
    ) 
    
    process
    { 
        if ($null -eq $path) { throw [System.ArgumentNullException]"path" }
        if ($User)
        {
            $p = Get-PathEnv -User
        }
        elseif ($persistent)
        {
            $p = Get-PathEnv -Machine
        }
        else
        {
            $p = Get-PathEnv -Current
        }
        $p = $p | ForEach-Object { $_.trimend("\") }
        $p = @($p)
        $paths = @($path) 
        $paths | ForEach-Object { 
            $path = $_.trimend("\")
            Write-Verbose "adding $path to PATH"
            if ($first)
            {
                if ($p.length -eq 0 -or $p[0] -ine $path)
                {
                    $p = @($path) + $p
                }
            }
            else
            {
                if ($path -inotin $p)
                {
                    $p += $path
                }
            }
        }
        
        if ($User)
        {
            Write-Verbose "saving user PATH and adding to current proc"
            [System.Environment]::SetEnvironmentVariable("PATH", [string]::Join(";", $p), [System.EnvironmentVariableTarget]::User);
            #add also to process PATH
            Add-ToPath $path -persistent:$false -first:$first
        }
        elseif ($persistent)
        {            
            write-Verbose "Saving to global machine PATH variable"
            [System.Environment]::SetEnvironmentVariable("PATH", [string]::Join(";", $p), [System.EnvironmentVariableTarget]::Machine);
            #add also to process PATH
            Add-ToPath $path -persistent:$false -first:$first
        }
        else
        {
            $env:path = [string]::Join(";", $p);
            [System.Environment]::SetEnvironmentVariable("PATH", $env:path, [System.EnvironmentVariableTarget]::Process);
        }
    }
}