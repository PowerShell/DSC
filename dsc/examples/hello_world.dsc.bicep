// This is a very simple example Bicep file for testing

targetScope = 'desiredStateConfiguration'

// use workaround where Bicep currently requires version in date format
resource echo 'Microsoft.DSC.Debug/Echo@2025-08-27' = {
  name: 'exampleEcho'
  properties: {
    output: 'Hello, world!'
  }
}

// This is waiting on https://github.com/Azure/bicep/issues/17670 to be resolved
// output exampleOutput string = echo.properties.output
