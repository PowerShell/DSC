configuration MyConfiguration {
        Import-DscResource -ModuleName PSDesiredStateConfiguration
        Node localhost
        {
            Environment CreatePathEnvironmentVariable
            {
                Name = 'TestPathEnvironmentVariable'
                Value = 'TestValue'
                Ensure = 'Present'
                Path = $true
                Target = @('Process')
            }
        }
    }