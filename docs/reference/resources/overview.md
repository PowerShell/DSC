# Built-in DSC resource reference

Each release of DSC includes built-in resources that you can use immediately after you install DSC.
This document lists the available resources and links to the reference documentation for each.

## All built-in resources

- [Microsoft/OSInfo](./microsoft/osinfo/resource.md)
- [Microsoft.DSC/Assertion](./microsoft/dsc/assertion/resource.md)
- [Microsoft.DSC/Group](./microsoft/dsc/group/resource.md)
- [Microsoft.DSC/Include](./microsoft/dsc/include/resource.md)
- [Microsoft.DSC/PowerShell](./microsoft/dsc/powershell/resource.md)
- [Microsoft.DSC.Debug/Echo](./microsoft/dsc/debug/echo/resource.md)
- [Microsoft.DSC.Transitional/RunCommandOnSet](./microsoft/dsc/transitional/runcomandonset/resource.md)
- [Microsoft.Windows/RebootPending](./microsoft/windows/rebootpending/resource.md)
- [Microsoft.Windows/Registry](./microsoft/windows/registry/resource.md)
- [Microsoft.WIndows/WindowsPowerShell](./microsoft/windows/windowspowershell/resource.md)
- [Microsoft.Windows/WMI](./microsoft/windows/wmi/resource.md)

## Built-in assertion resources

You can use the following built-in resources to query the current state of a machine but not to
change the state of the machine directly:

- [Microsoft/OSInfo](./microsoft/osinfo/resource.md)
- [Microsoft.DSC/Assertion](./microsoft/dsc/assertion/resource.md)
- [Microsoft.Windows/RebootPending](./microsoft/windows/rebootpending/resource.md)

## Built-in adapter resources

You can use the following built-in resources to leverage resources that don't define a DSC Resource
Manifest:

- [Microsoft.DSC/PowerShell](./microsoft/dsc/powershell/resource.md)
- [Microsoft.Windows/WindowsPowerSHell](./microsoft/windows/windowspowershell/resource.md)
- [Microsoft.Windows/WMI](./microsoft/windows/wmi/resource.md)

## Built-in configurable resources

The following built-in resources to change the state of a machine directly:

- [Microsoft.DSC.Transitional/RunCommandOnSet](./microsoft/dsc/transitional/runcomandonset/resource.md)
- [Microsoft.Windows/Registry](./microsoft/windows/registry/resource.md)

## Built-in debugging resources

You can use the following built-in resources when debugging or exploring DSC. They don't affect
the state of the machine.

- [Microsoft.DSC.Debug/Echo](./microsoft/dsc/debug/echo/resource.md)

## Built-in group resources

You can use the following built-in resources to change how DSC processes a group of nested resource
instances:

- [Microsoft.DSC/Assertion](./microsoft/dsc/assertion/resource.md)
- [Microsoft.DSC/Group](./microsoft/dsc/group/resource.md)
- [Microsoft.DSC/Include](./microsoft/dsc/include/resource.md)
