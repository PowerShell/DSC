# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Group resource tests' {
    It 'Nested groups should work for get' {
        $out = (dsc config get -p $PSScriptRoot/../examples/groups.dsc.yaml -f yaml | Out-String).Trim()
        $LASTEXITCODE | Should -Be 0
        $out | Should -BeExactly @'
results:
- name: First Group
  type: DSC/Group
  result:
  - name: First
    type: Test/Echo
    result:
      actualState:
        text: First
  - name: Nested Group
    type: DSC/Group
    result:
    - name: Nested First
      type: Test/Echo
      result:
        actualState:
          text: Nested First
    - name: Nested Second
      type: Test/Echo
      result:
        actualState:
          text: Nested Second
- name: Last Group
  type: DSC/Group
  result:
  - name: Last
    type: Test/Echo
    result:
      actualState:
        text: Last
messages: []
hadErrors: false
'@
    }

}
