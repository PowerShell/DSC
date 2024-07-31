function Export-DscConfigurationDocument
{
    [CmdletBinding()]
    Param 
    (
        [Parameter(Mandatory = $true)]
        [System.String]
        $Path
    )

    if (-not (Test-PsPathExtension $Path))
    {
        return @{}
    }

    # Parse the abstract syntax tree to get all hash table values representing the configuration resources
    [System.Management.Automation.Language.Token[]] $tokens = $null
    [System.Management.Automation.Language.ParseError[]] $errors = $null
    $ast = [System.Management.Automation.Language.Parser]::ParseFile($Path, [ref]$tokens, [ref]$errors)
    $configurations = $ast.FindAll({$args[0].GetType().Name -like 'HashtableAst'}, $true)

    # Create configuration document resource class (can be re-used)
    $configurationDocument = [DscConfigurationResource]::new()
    
    # Build simple regex
    $regex = [regex]::new('Configuration\s+(\w+)')
    $configValue = $regex.Matches($ast.Extent.Text).Value

    if (-not $configValue) 
    {
        return
    }

    $documentConfigurationName  = $configValue.TrimStart('Configuration').Trim(" ")

    # Start to build the outer basic format
    $configurationDocument.name = $documentConfigurationName
    # Hardcoded PowerShell 7 adapter type info
    $configurationDocument.type = 'Microsoft.DSC/PowerShell' 

    # Bag to hold resources
    $resourceProps = [System.Collections.Generic.List[object]]::new()

    foreach ($configuration in $configurations) 
    {
        # Get parent configuration details
        $resourceName = ($configuration.Parent.CommandElements.Value | Select-Object -Last 1 )
        $resourceConfigurationName = ($configuration.Parent.CommandElements.Value | Select-Object -First 1)

        # Get module details 
        $module = Get-DscResource -Name $resourceConfigurationName -ErrorAction SilentlyContinue

        # Build the module
        $resource = [DscConfigurationResource]::new()
        $resource.properties = $configuration.SafeGetValue()
        $resource.name = $resourceName
        $resource.type = ("{0}/{1}" -f $module.ModuleName, $resourceConfigurationName)

        Write-Verbose ("Adding document with data")
        Write-Verbose ($resource | ConvertTo-Json | Out-String)
        $resourceProps.Add($resource)
    }

    # Add all the resources
    $configurationDocument.properties = @{
        resources = $resourceProps
    }

    return $configurationDocument
}