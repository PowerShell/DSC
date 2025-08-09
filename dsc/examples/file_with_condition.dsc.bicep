// This example demonstrates how to create a file using the Windows PowerShell DSC extension.
// The file is created in the C:\DSC directory on the target machine.
// You should at least have the Bicep CLI v0.34.34 installed to run this example with experimental feature desiredStateConfiguration turned on.
// To run the second resource, you can add the --parameters '{"parameters":{"restartService":true}}' flag to the command line.
// The configuration document requires to be run elevated.
// From dsc version 3.2.0-preview.4 and above onwards, you can directly run the example using `dsc config get --file file_with_condition.dsc.bicep`.

targetScope = 'desiredStateConfiguration'

@description('Set to true to ensure the service is running after the file creation.')
param restartService bool = false

resource powerShellAdapter 'PSDesiredStateConfiguration/File@2025-01-07' = {
  name: 'Use Bicep to create file'
  properties: {
        Ensure: 'Present'
        Type: 'File'
        DestinationPath: '${systemroot()}\\DSC\\config.txt'
        Contents: 'This file was created using Bicep extension from DSC.'
    }
}

// Optionally ensure the service is running after the file creation
resource ensureServiceRunning 'PSDesiredStateConfiguration/Service@2025-01-07' = if (restartService) {
  name: 'Ensure DSC service is running'
  properties: {
      Name: 'Spooler'
      StartupType: 'Automatic'
      State: 'Running'
  }
}

