# Desired State Configuration Modernization - Product Requirements

## Overview

Today, DSC resources and configurations can only be written in PowerShell. In order to meet our customers where they are and allow them to gain more value from DSC (and by extension, Azure Machine Configuration) more quickly, we want to allow resources to be written in any language, even shell scripts. Configurations should be able to be authored easily in familiar and pervasive JSON/YAML syntax.

Support for Linux is a must. MOF and WMI/OMI dependencies are relics of the past Windows centric world of DSC, only cause complications and do not bring any benefits. We want to remove reliance on these technologies to make DSC more portable, more powerful, and more useful.

## Goals

### Goals

- Make authoring DSC resources easy using any language.
  - Create native APIs and commands to invoke the Get/Set/Test methods of the DSC resources.
  - Define a Json manifest to describe how to invoke the new DSCv3 resources.
  - Define the contract between DSC and the resources for passing input and output.
  - Continue to provide support for existing PowerShell script and class-based resources.
- Make authoring DSC Configurations quicker and easier.
  - Define a new schema for creating DSC Configurations in JSON/YAML.
  - Eliminate the need for a configuration to be compiled.
  - Make configurations more dynamic with support for variables, conditions, and other functions.
  - Create native APIs and commands to invoke DSC Configurations
- Make it easy to use DSC Configurations in Azure.
  - New, non-PowerShell resources must be able to be retrieved from PowerShell Gallery and private repositories (including folder/share repos).
  - Linux and Windows support is a must.

### Non-Goals

- The PowerShell team will no longer provide an LCM.
  - Continous application of Configurations and storage of results can be provided by other tools, such as Azure Machine Configuration.

## Background

There has been a lot of changes in the Configuration Management/Infrastructure as Code space since DSC first came out. Our competitors are the market leaders and we have not been playing catch-up. Ansible is a notable leader in this space and the ability to author "playbooks" in YAML is appealing to many of its users. It also provides a way to simply run its playbooks without having to compile, pull/push the playbook to the target machine's LCM and have it execute in the background.

The lack of notable improvements in DSC have caused many potential users to feel that it is dead/abandoned technology.

Additionally, recent research completed by the Machine Configuration team shows that there is a longer need to ramp up on DSC/Machine Config before enough value can be garnered for adoption. We need to make it easier for customers to use DSC and Machine Config.

## Requirements

- As a resource author, it is easy for me to write and share resources.
  - I can use any language I'm comfortable with to author my resource.
  - I can easily find and implement the input contract for my resource because input is JSON sent to stdin.
  - I can easily find and implement the output contract for my resource because it is JSON sent to stdout.
  - I can easily implement my resource because the contracts are lightweight.
  - I can easily define my manifest that instructs DSC how to call my resource.
  - I can publish my resource at PowerShell Gallery.
  - I do not need to make any changes to my existing PowerShell script and class based resources for them to be continued to be used.
  - I can easily test my resources by using a native command to invoke it.
  - I can write cross platform resources.
  - I can write resources that are not cross-platform.
- As a configuration author, its easy for me to write and test configurations.
  - I do not need to know a programming language or custom DSL to write a configuration.
  - I can write configurations in JSON or YAML.
  - I do not need to compile or convert my JSON or YAML configuration before it can be invoked.
  - I can discover and use existing resources written by Microsoft and the community on PowerShell Gallery.
  - I can write configurations the same way and utilize the same tools for Windows and Linux.
- As a configuration author, its easy for me to use my configurations in Azure.
  - I can write configuration using knowledge that I already have about Azure because configurations look just like ARM Templates.
  - I can easily start using Azure since ARM Templates look and work like my DSC Configurations.
  
## Assumptions, Constraints, Dependencies

### Assumptions

- The PowerShell team will deliver the native APIs and command that are equivalent to `Invoke-DscResource`. This work requires that the details of the manifest and resource input/output contract are defined.
- The goal "Eliminate the need for a configuration to be compiled" does not preclude implementing the ability later to enable other means of writing Configurations that would then be "compiled" into JSON (such as Bicep, PowerShell, HCL, etc.).

## User Interface and Design

The user interface will consist of command line tools and schema for configurations and resource manifests.

### **dsc.exe**

```text
DSC is a desired state configuration tool.

Usage: dsc subcommand <options>

Subcommands:
config <set|test>               - Invoke DSC configurations
module <find|install>           - Find and install DSC modules
repo <set|remove>               - Add, update or remove module repositories
resource <get|set|test|find?    - Work with DSC resources

Examples:
dsc.exe resource set --name user --module PSDscResources --version 1.1.0 --property <name=value> --property <name=value>
dsc.exe repo set --name Contoso --url https://gallery.contoso.com/api/v2
dsc.exe repo remove --name Contoso
dsc config test --file test.json --parameter-file test.parameters.json --output test.output.json --parameter <name=value>
```

### **Configuration Schema**

The configuration schema is the format of the document used to author configurations. It should be easy enough to author by hand without any special authoring tools.

For the schema, we will use the same schema that is used for Azure (ARM) Templates. Most of the same functions should be available to make configurations dynamic enough to be easily re-usable and adaptable.

JSON
```JSON
{
  "parameters": {
    "timeZone": {
      "type": "string",
      "defaultValue": "Pacific Standard Time"
    }
  },
  "variables": {
    "userName": "TestUser",
    "groupName": "TestGroup",
    "foo": {
      "bar": "baz"
    }
  },
  "resources": [
    {
      "name": "test-user",
      "type": "PSDscResources/User",
      "properties": {
        "UserName": "[variables('userName')]"
      }
    },
    {
      "name": "test-group",
      "type": "PSDscResources/Group",
      "properties": {
        "GroupName": "[variables('groupName')]",
        "MembersToInclude": [
          "[reference('test-user').UserName]"
        ]
      }
    },
    {
      "condition": "[not(equals(variables('foo').bar, 'baz'))]",
      "name": "spooler-service",
      "type": "PSDscResources/Service",
      "properties": {
        "Name": "Spooler"
      }
    },
    {
      "name": "timezone",
      "type": "xTimeZone/xTimeZone",
      "properties": {
        "TimeZone": "[parameters('timeZone')]",
        "IsSingleInstance": "yes"
      }
    },
    {
      "name": "securityoption",
      "type": "SecurityPolicyDsc/SecurityOption",
      "properties": {
        "Name": "SecurityOption",
        "Accounts_Guest_account_status": "Disabled"
      }
    }
  ]
}
```

YAML
```YAML
parameters:
  timeZone:
    type: string
    defaultValue: Pacific Standard Time

variables:
  # Simple variables
  userName: TestUser
  groupName: TestGroup

  foo: # Variable with a value that is a hashtable
    bar: baz

resources:
  - name: test-user # The results of the Get method are stored in a variable named 'test-user'. Get is called after Set.
    type: PSDscResources/User
    properties:
      UserName: "[variables('userName')]" # This is an example of how to use a simple variable
    
  - name: test-group
    type: PSDscResources/Group
    properties:
      GroupName: "[variables('groupName')]"
      MembersToInclude:
        - "[reference('test-user').UserName]"

  - condition: "[not(equals(variables('foo').bar, 'baz'))]" # Conditions must return $true or else this task will be skipped
    name: spooler-service
    type: PSDscResources/Service
    properties:
      Name: Spooler

  - name: timezone
    type: xTimeZone/xTimeZone
    properties:
      TimeZone: "[parameters('timeZone')]"
      IsSingleInstance: yes
  
  - name: securityoption
    type: SecurityPolicyDsc/SecurityOption
    properties:
      Name: SecurityOption
      Accounts_Guest_account_status: Disabled

```

## Risks

- Skills of working in languages that compile to native are in short supply on our dev teams. This may require some initial ramp up in the required languages for existing devs.

## Open Items

- Details about how native DSC resources will be made available in PowerShell Gallery.
- Native commands/APIs for finding and retrieving native resources from PowerShell Gallery.

## References
