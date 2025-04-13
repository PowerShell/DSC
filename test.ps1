$yaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: File
    type: Microsoft.Windows/WindowsPowerShell
    properties:
      resources:
        - name: File
          type: PSDesiredStateConfiguration/File
          properties:
            DestinationPath: $testdrive\test.txt
  - name: File present
    type: Microsoft.DSC/Assertion
    properties:
      $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
      resources:
        - name: Use powershell adapter
          type: Microsoft.Windows/WindowsPowerShell
          properties:
            resources:
              - name: File present
                type: PSDesiredStateConfiguration/File
                properties:
                  DestinationPath: $testDrive\test.txt
    dependsOn:
      - "[resourceId('Microsoft.Windows/WindowsPowerShell', 'File')]"
'@