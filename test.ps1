Configuration MyConfiguration {
    Import-DscResource -ModuleName PSDesiredStateConfiguration 
    node localhost {
        Environment CreatePathEnvironmentVariable {
            Name = 'TestPathEnvironmentVariable'
            Value = 'TestValue'
            Ensure = 'Present'
            Path = $true
            Target = @('Process')
        }
    }
}