// This is a very simple example Bicep file for testing

targetScope = 'desiredStateConfiguration'

// use workaround where Bicep currently requires version in date format
resource echo 'Microsoft.DSC.Debug/Echo@2025-01-01' = {
  name: 'exampleEcho'
  properties: {
    output: 'Hello, world!'
  }
}
