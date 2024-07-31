[scriptblock]$dscExeSubCommand = {
    param(
        $commandName,
        $parameterName,
        $wordToComplete,
        $commandAst,
        $fakeBoundParameters
    )

    $exeLocation = Resolve-DscExe -ErrorAction SilentlyContinue
    if ($exeLocation)
    {
        # TODO: Filter better
        $files = Get-ChildItem -Path (Split-Path -Path $exeLocation -Parent) -Filter '*.dsc.resource.json'
        $files | ForEach-Object {
            $typeName = (Get-Content $_ | ConvertFrom-Json -ErrorAction SilentlyContinue).type
            if ($typeName) 
            {
                # register new completer
                New-Object -Type System.Management.Automation.CompletionResult -ArgumentList @(
                    $typeName
                    $typeName
                    'ParameterValue'
                    $typeName
                )
            }  
        }
    }
}

Register-ArgumentCompleter -CommandName Invoke-DscResourceConfigurationDocument -ParameterName ResourceName -ScriptBlock $dscExeSubCommand