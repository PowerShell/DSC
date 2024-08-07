function Test-YamlModule 
{
    if (-not (Get-Command -Name 'ConvertTo-Yaml' -ErrorAction SilentlyContinue))
    {
        return $false 
    }

    return $true
}