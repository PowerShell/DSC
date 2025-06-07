---
description: >-
  This article aims to guide you through methods to output from PowerShell in formats that are
  friendly for screen readers, enhancing the accessibility of your scripts.
ms.custom: experience
ms.date: 03/25/2025
title: Improve the accessibility of DSC output in PowerShell
---

# Improve the accessibility of DSC output in PowerShell

Most terminal environments only display raw text. Users that rely on screen readers are faced with
tedious narration when consuming large amounts of raw text because the raw output doesn't have the
accessibility metadata to characterize the format of the content.

There are two ways to improve the accessibility of the output in PowerShell:

- Output the data in a way that it can be viewed in another tool that supports screen reading
  technologies.
- Reduce the amount of output displayed in the terminal by filtering and selecting the data you
  want and output the text in a more readable format.

## Display the data in a tool outside of the terminal

For large amounts of data, rather than output to the host, consider writing output in a format that
can be viewed in another tool that supports screen reading technologies. You might need to save the
data to a file in a format that can be opened in another application.

### Out-GridView command on Windows

For small to moderate size output, use the `Out-GridView` command. The output is rendered using
Windows Presentation Foundation (WPF) in tabular form, much like a spreadsheet. The GridView
control allows you to sort, filter, and search the data, which reduces the amount of data that
needs to be read. The GridView control is also accessible to screen readers. The **Narrator** tool
built into Windows is able to read the GridView details, including column names and row count.

The following example shows how to display a list of DSC resources in a GridView control.

```powershell
dsc resource list | ConvertFrom-Json | Out-GridView
```

The following example shows how to display a list of DSC adapted resources in a GridView control.

```powershell
dsc resource list -a Microsoft.Windows/WindowsPowerShell |
    ConvertFrom-Json |
    Out-GridView
```

The `Out-GridView` command is only available in PowerShell on Windows.

### Character Separated Value (CSV) format

Spreadsheet applications such as **Microsoft Excel** support CSV files. The following example shows
how to save the output of a command to a CSV file.

```powershell
dsc resource list | ConvertFrom-Json | Export-Csv -Path .\myFile.csv
Invoke-Item .\myFile.csv
```

The `Invoke-Item` command opens the file in the default application for CSV files.

## Reduce the amount of output

One way to improve the accessibility of the output is to reduce the amount of output displayed in
the terminal. PowerShell has several commands that can help you filter and select the data you
want.

### Select and filter data

Rather than returning a large mount of data, use commands such as `Select-Object`, `Sort-Object`,
and `Where-Object` to reduce the amount of output. The following example gets the list of Windows
PowerShell DSC resources that manage processes on the computer.

Each of the following commands improves the output in a different way:

- The `Where-Object` command reduces the number of items returned by filtering the list to only
  show resources that have the word `process` in their type name.
- The `Select-Object` command selects only the resource type name, kind, and version.
- The `Format-List` command displays the output in list format, which provides a better narration
  experience for screen readers.

```powershell
dsc resource list -a Microsoft.Windows/WindowsPowerShell |
    ConvertFrom-Json | 
    Where-Object {$_.type -like "*process*" } |
    Select-Object -Property Type, Kind, Version |
    Format-List
```

## Related content

- [Improve the accessibility of output in PowerShell][01]
- [Out-GridView][02]
- [Export-Csv][03]
- [ConvertTo-Html][04]
- [about_Calculated_Properties][05]

<!-- Link reference definitions -->
[01]: /powershell/scripting/learn/shell/output-for-screen-reader
[02]: xref:Microsoft.PowerShell.Utility.Out-GridView
[03]: xref:Microsoft.PowerShell.Utility.Export-Csv
[04]: xref:Microsoft.PowerShell.Utility.ConvertTo-Html
[05]: /powershell/module/microsoft.powershell.core/about/about_calculated_properties
