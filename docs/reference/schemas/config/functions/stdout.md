---
description: Reference for the 'stdout' DSC configuration document function
ms.date:     09/01/2026
ms.topic:    reference
title:       stdout
---

# stdout

## Synopsis

Returns the standard output of the command that a DSC extension ran to import a file, for use in
the `output` expression of the extension manifest's `import` method.

## Syntax

```Syntax
stdout()
```

## Description

The `stdout()` function returns, as a string, the standard output that DSC captured from the last
command it ran. DSC only provides this value in one situation: when it processes the `output`
expression of an extension's `import` method.

When you pass a file to DSC with the `--file` option, DSC first checks whether any discovered
extension with the `import` capability lists the file's extension in the `fileExtensions` property
of its `import` method. If one does, DSC runs the command defined by that method's `executable`
and `args` properties and captures the command's standard output. Then:

- If the `import` method doesn't define the `output` property, DSC uses the captured standard
  output as the imported content.
- If the `import` method defines the `output` property, DSC evaluates that property as an
  expression. Inside the expression, `stdout()` returns the captured standard output. DSC converts
  the value the expression returns to JSON and uses it as the imported content.

DSC then processes the imported content as the configuration document.

Use `stdout()` when the command's standard output isn't a configuration document by itself but
contains one or can be converted into one. For example, the expression can parse the output with
[`json()`][00] and use the property access syntax to extract the document from a wrapper object.

The `output` expression must return an object for the imported content to be a valid
configuration document. If the expression returns a string, DSC converts it to a JSON string
literal, which isn't a valid document. To use the standard output as-is, omit the `output`
property instead of defining it as `[stdout()]`.

DSC evaluates the `output` expression in a new, empty context. The expression can't access
configuration parameters, variables, or resource references. It can only use `stdout()` and
functions that don't depend on the configuration document.

Despite the wording of the description in the output of `dsc function list`, DSC doesn't provide
the standard output of resources to configuration documents. Using `stdout()` in a configuration
document always raises an error.

## Examples

### Example 1 - Import a wrapped configuration document

The following extension manifest defines an `import` method for files with the `wrapped` file
extension. The command uses PowerShell to read the file and write its content to standard output.
The `output` expression parses that output with [`json()`][00] and returns the `document`
property, which contains the actual configuration document.

Save the manifest as `wrapped.dsc.extension.json` in a folder that's included in the `PATH`
environment variable so DSC can discover it.

```json
{
  "$schema": "https://aka.ms/dsc/schemas/v3/bundled/extension/manifest.json",
  "type": "Example.Import/Wrapped",
  "version": "0.1.0",
  "description": "Imports configuration documents wrapped in a JSON envelope.",
  "import": {
    "fileExtensions": ["wrapped"],
    "executable": "pwsh",
    "args": ["-NoProfile", "-Command", "Get-Content", "-Raw", { "fileArg": "-Path" }],
    "output": "[json(stdout()).document]"
  }
}
```

When an entry in `args` is an object with the `fileArg` property, DSC replaces it with the value
of `fileArg` followed by the absolute path to the file being imported.

The following file wraps a configuration document in an object with `format` and `document`
properties. Save it as `stdout.example.1.dsc.config.wrapped`.

```json
{
  "format": "wrapped",
  "document": {
    "$schema": "https://aka.ms/dsc/schemas/v3/bundled/config/document.json",
    "resources": [
      {
        "name": "Echo",
        "type": "Microsoft.DSC.Debug/Echo",
        "properties": {
          "output": "Imported through stdout()"
        }
      }
    ]
  }
}
```

```bash
dsc config get --file stdout.example.1.dsc.config.wrapped
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: Imported through stdout()
messages: []
hadErrors: false
```

When DSC processes this command, it:

1. Discovers the `Example.Import/Wrapped` extension and matches the file's `wrapped` extension
   against the `fileExtensions` property.
1. Runs `pwsh -NoProfile -Command Get-Content -Raw -Path <absolute path to the file>` and captures
   the standard output.
1. Evaluates `[json(stdout()).document]`, where `stdout()` returns the captured content.
1. Processes the object returned by the expression as the configuration document.

## Parameters

The function doesn't accept any arguments.

## Output

Returns the standard output that DSC captured from the extension's import command.

```yaml
Type: string
```

## Error conditions

The function raises an error in the following cases:

- **No standard output available**: The function is used anywhere other than in the `output`
  expression of an extension's `import` method, including in a configuration document. DSC raises
  `No standard output is available from the last executed resource`.
- **Arguments passed**: The function is called with one or more arguments. DSC raises
  `Function 'stdout' does not accept arguments`.

If the `output` expression itself raises an error while DSC imports a file, DSC doesn't report
that error. Instead, it tries any other extension with the `import` capability and then reads the
file directly as a configuration document, which usually fails with a parsing error about the
file's content.

## Notes

- The only place DSC sets the value that `stdout()` returns is when it processes the `output`
  expression of an extension's `import` method. Resources don't expose their standard output to
  configuration documents through this function.
- DSC only evaluates the `output` expression when the import command wrote something to standard
  output.
- DSC evaluates the `output` expression in a new context without any configuration parameters or
  variables.
- The `output` expression should return an object, not a string, so that the imported content is
  a valid configuration document.
- For more information about extension manifests, see the
  [DSC extension manifest schema reference][01].

## Related functions

- [`json()`][00] - Parses a JSON string and returns the resulting value

<!-- Link reference definitions -->
[00]: ./json.md
[01]: ../../extension/manifest/root.md
