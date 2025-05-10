// This example demonstrates how to create a file using the Windows PowerShell DSC extension.
// The file is created in the C:\DSC directory on the target machine.
// You should at least have the Bicep CLI v0.34.34 installed to run this example with experimental feature desiredStateConfiguration turned on.

targetScope = 'desiredStateConfiguration'

resource powerShellAdapter 'Microsoft.Windows/WindowsPowerShell@2025-01-07' = {
  name: 'Use Bicep to create file'
  properties: {
      resources: [
          {
              name: 'File'
              type: 'PSDesiredStateConfiguration/File'
              properties: {
                  Ensure: 'Present'
                  Type: 'Directory'
                  DestinationPath: 'C:\\DSC\\'
              }
          }
      ]
  }
}

// bicep build file.bicep
