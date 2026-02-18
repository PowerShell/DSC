# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Group resource tests' {
    It 'Nested groups should work for get' {
        $out = (dsc config get -f $PSScriptRoot/../examples/groups.dsc.yaml -o yaml | Out-String).Trim()
        $LASTEXITCODE | Should -Be 0
        $out | Should -BeLike @'
executionInformation:
  duration: PT*S
  endDatetime: *
  executionType: actual
  operation: get
  startDatetime: *
  version: 3*
metadata:
  Microsoft.DSC:
    duration: PT*S
    endDatetime: *
    executionType: Actual
    operation: Get
    securityContext: *
    startDatetime: *
    version: 3*
results:
- executionInformation:
    duration: *
  metadata:
    Microsoft.DSC:
      duration: *
  name: First Group
  type: Microsoft.DSC/Group
  result:
  - executionInformation:
      duration: *
    metadata:
      Microsoft.DSC:
        duration: *
    name: First
    type: Microsoft.DSC.Debug/Echo
    result:
      actualState:
        output: First
  - executionInformation:
      duration: *
    metadata:
      Microsoft.DSC:
        duration: *
    name: Nested Group
    type: Microsoft.DSC/Group
    result:
    - executionInformation:
        duration: *
      metadata:
        Microsoft.DSC:
          duration: *
      name: Nested First
      type: Microsoft.DSC.Debug/Echo
      result:
        actualState:
          output: Nested First
    - executionInformation:
        duration: *
      metadata:
        Microsoft.DSC:
          duration: *
      name: Nested Second
      type: Microsoft.DSC.Debug/Echo
      result:
        actualState:
          output: Nested Second
- executionInformation:
    duration: *
  metadata:
    Microsoft.DSC:
      duration: *
  name: Last Group
  type: Microsoft.DSC/Group
  result:
  - executionInformation:
      duration: *
    metadata:
      Microsoft.DSC:
        duration: *
    name: Last
    type: Microsoft.DSC.Debug/Echo
    result:
      actualState:
        output: Last
messages: `[`]
hadErrors: false
'@ -Because $out
    }

}
