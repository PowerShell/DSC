# DSCTest Resource

## Sleep

Example config:

```yaml
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/config/document.json
resources:
- name: Sleep1
  type: Test/Sleep
  properties:
    seconds: 30
```
