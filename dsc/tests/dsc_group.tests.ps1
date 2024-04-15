# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Group resource tests' {
    It 'Nested groups should work for get' {
        $out = (dsc config get -p $PSScriptRoot/../examples/groups.dsc.yaml -f yaml | Out-String).Trim()
        $LASTEXITCODE | Should -Be 0
        $out | Should -BeLike @'
metadata:
  Microsoft.DSC:
    version: 3*
    operation: Get
    executionType: Actual
    startDatetime: *
    endDatetime: *
    duration: PT*S
    securityContext: *
results:
- metadata:
    Microsoft.DSC:
      duration: *
  name: First Group
  type: Microsoft.DSC/Group
  result:
  - metadata:
      Microsoft.DSC:
        duration: *
    name: First
    type: Test/Echo
    result:
      actualState:
        output: First
  - metadata:
      Microsoft.DSC:
        duration: *
    name: Nested Group
    type: Microsoft.DSC/Group
    result:
    - metadata:
        Microsoft.DSC:
          duration: *
      name: Nested First
      type: Test/Echo
      result:
        actualState:
          output: Nested First
    - metadata:
        Microsoft.DSC:
          duration: *
      name: Nested Second
      type: Test/Echo
      result:
        actualState:
          output: Nested Second
- metadata:
    Microsoft.DSC:
      duration: *
  name: Last Group
  type: Microsoft.DSC/Group
  result:
  - metadata:
      Microsoft.DSC:
        duration: *
    name: Last
    type: Test/Echo
    result:
      actualState:
        output: Last
messages: `[`]
hadErrors: false
'@
    }

}
