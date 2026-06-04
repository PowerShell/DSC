# Built-in DSC resource reference

Each release of DSC includes built-in resources that you can use immediately after you install DSC.
This document lists the available resources and links to the reference documentation for each.

## All built-in resources

- [Microsoft/OSInfo](./Microsoft/OSInfo/index.md)
- [Microsoft.DSC/Assertion](./Microsoft/DSC/Assertion/index.md)
- [Microsoft.DSC/Group](./Microsoft/DSC/Group/index.md)
- [Microsoft.DSC/Include](./Microsoft/DSC/Include/index.md)
- [Microsoft.Adapter/PowerShell](./Microsoft/Adapter/PowerShell/index.md)
- [Microsoft.Adapter/WindowsPowerShell](./Microsoft/Adapter/WindowsPowerShell/index.md)
- [Microsoft.DSC/PowerShell](./Microsoft/DSC/PowerShell/index.md)
- [Microsoft.DSC.Debug/Echo](./Microsoft/DSC/Debug/echo/index.md)
- [Microsoft.DSC.Transitional/RunCommandOnSet](./Microsoft/DSC/Transitional/RunCommandOnSet/index.md)
- [Microsoft.Windows/RebootPending](./Microsoft/Windows/RebootPending/index.md)
- [Microsoft.Windows/Registry](./Microsoft/Windows/Registry/index.md)
- [Microsoft.Windows/WindowsPowerShell](./Microsoft/Windows/WindowsPowerShell/index.md)
- [Microsoft.Windows/WMI](./Microsoft/Windows/WMI/index.md)

## Built-in assertion resources

You can use the following built-in resources to query the current state of a machine but not to
change the state of the machine directly:

- [Microsoft/OSInfo](./Microsoft/OSInfo/index.md)
- [Microsoft.DSC/Assertion](./Microsoft/DSC/Assertion/index.md)
- [Microsoft.Windows/RebootPending](./Microsoft/Windows/RebootPending/index.md)

## Built-in adapter resources

You can use the following built-in resources to leverage resources that don't define a DSC Resource
Manifest:

- [Microsoft.Adapter/PowerShell](./Microsoft/Adapter/PowerShell/index.md)
- [Microsoft.Adapter/WindowsPowerShell](./Microsoft/Adapter/WindowsPowerShell/index.md)
- [Microsoft.DSC/PowerShell](./Microsoft/DSC/PowerShell/index.md)
- [Microsoft.Windows/WindowsPowerShell](./Microsoft/windows/windowspowershell/index.md)
- [Microsoft.Windows/WMI](./Microsoft/windows/wmi/index.md)

> [!WARNING]
> `Microsoft.DSC/PowerShell` and `Microsoft.Windows/WindowsPowerShell` will be deprecated in a
> future release. Use `Microsoft.Adapter/PowerShell` and `Microsoft.Adapter/WindowsPowerShell`
> instead.

## Built-in configurable resources

The following built-in resources to change the state of a machine directly:

- [Microsoft.DSC.Transitional/RunCommandOnSet](./Microsoft/DSC/Transitional/RunCommandOnSet/index.md)
- [Microsoft.Windows/Registry](./Microsoft/Windows/Registry/index.md)

## Built-in debugging resources

You can use the following built-in resources when debugging or exploring DSC. They don't affect
the state of the machine.

- [Microsoft.DSC.Debug/Echo](./Microsoft/DSC/Debug/echo/index.md)

## Built-in group resources

You can use the following built-in resources to change how DSC processes a group of nested resource
instances:

- [Microsoft.DSC/Assertion](./Microsoft/DSC/Assertion/index.md)
- [Microsoft.DSC/Group](./Microsoft/DSC/Group/index.md)
- [Microsoft.DSC/Include](./Microsoft/DSC/Include/index.md)
