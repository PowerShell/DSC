---
description: Reference for the rules the 'dsc lint' command checks configuration documents against
ms.date:     10/20/2026
ms.topic:    reference
title:       dsc lint rules
---

# dsc lint rules

The `dsc lint` command checks a configuration document against the rules on this page. Each rule
has an identifier you can use with the `--rule` option or in the `lint` section of the settings
file, a category, a default level, and a tier:

- `static` rules only need the document.
- `resource` rules need the installed resources and only run when discovery is enabled.

Rules with a fixed level can't be changed or turned off.

## Rules at a glance

| Rule                             | Category    | Default level | Tier     |
| :------------------------------- | :---------- | :------------ | :------- |
| `parse-error`                    | document    | error (fixed) | static   |
| `schema-invalid`                 | document    | error (fixed) | static   |
| `unsupported-document-type`      | document    | error (fixed) | static   |
| `duplicate-resource`             | document    | error         | static   |
| `duplicate-resource-name`        | document    | warning       | static   |
| `empty-resources`                | document    | warning       | static   |
| `unknown-microsoft-dsc-metadata` | document    | warning       | static   |
| `api-version-alias`              | document    | info          | static   |
| `date-version`                   | document    | warning       | static   |
| `missing-require-version`        | document    | off           | static   |
| `invalid-expression`             | expression  | error         | static   |
| `unknown-function`               | expression  | error         | static   |
| `invalid-argument-count`         | expression  | error         | static   |
| `invalid-argument-type`          | expression  | error         | static   |
| `undefined-parameter`            | expression  | error         | static   |
| `undefined-variable`             | expression  | error         | static   |
| `condition-not-expression`       | expression  | warning       | static   |
| `no-unused-parameters`           | parameter   | warning       | static   |
| `no-unused-variables`            | parameter   | warning       | static   |
| `parameter-default-type`         | parameter   | error         | static   |
| `invalid-parameter-constraints`  | parameter   | error         | static   |
| `secure-parameter-default`       | parameter   | warning       | static   |
| `prefer-secure-type`             | parameter   | warning       | static   |
| `secure-value-in-output`         | parameter   | warning       | static   |
| `use-parameter-description`      | parameter   | off           | static   |
| `undeclared-parameter-value`     | parameter   | error         | static   |
| `missing-parameter-value`        | parameter   | error         | static   |
| `parameter-value-type`           | parameter   | error         | static   |
| `unresolved-dependency`          | dependency  | error         | static   |
| `circular-dependency`            | dependency  | error         | static   |
| `reference-without-depends-on`   | dependency  | warning       | static   |
| `duplicate-depends-on`           | dependency  | info          | static   |
| `copy-loop-deprecated`           | deprecation | warning       | static   |
| `copy-options-unsupported`       | deprecation | error         | static   |
| `security-context-in-metadata`   | deprecation | warning       | static   |
| `security-context-conflict`      | deprecation | error         | static   |
| `dsc-version-unsatisfied`        | deprecation | warning       | static   |
| `system-root-unused`             | environment | warning       | static   |
| `hardcoded-system-path`          | environment | off           | static   |
| `discovery-failed`               | resource    | warning (fixed) | resource |
| `unknown-resource-type`          | resource    | warning       | resource |
| `unsatisfied-require-version`    | resource    | warning       | resource |
| `deprecated-resource`            | resource    | warning       | resource |
| `property-schema-mismatch`       | resource    | error         | resource |
| `nested-resources-unsupported`   | resource    | warning       | resource |
| `exist-false-unsupported`        | resource    | error         | resource |
| `canonical-type-name`            | resource    | info          | resource |

## Document rules

### parse-error

The document must be valid JSON or YAML. The diagnostic includes the parser's message and, when
known, the line and column of the problem. This rule has a fixed level.

### schema-invalid

The document must validate against the configuration document JSON schema. The command reports
every violation with the JSON pointer to the problem, such as an unknown property, a missing
`$schema`, or a `dependsOn` entry that isn't a `resourceId()` expression. This rule has a fixed
level.

### unsupported-document-type

Only configuration documents can be linted. A document whose `$schema` identifies a resource or
extension manifest is rejected. This rule has a fixed level.

### duplicate-resource

Resource names must be unique among resources of the same type at the same level. DSC refuses to
run a document that declares the same resource twice. Types are compared without regard to case.

```yaml
resources:
- name: Same
  type: Microsoft.DSC.Debug/Echo
- name: Same          # duplicate-resource
  type: microsoft.dsc.debug/echo
```

### duplicate-resource-name

Resources at the same level should have distinct names even when their types differ, because DSC
resolves dependencies by name first and then checks the type.

### empty-resources

The configuration should declare at least one resource.

### unknown-microsoft-dsc-metadata

Keys under `Microsoft.DSC` metadata must be ones DSC recognizes. DSC silently ignores unknown
keys, so a typo such as `securityContxt` has no effect.

### api-version-alias

Resources should use `requireVersion` rather than its alias `apiVersion`.

### date-version

Version requirements should be semantic. Date-based versions such as `2025-01-01` are deprecated
and only supported for compatibility.

### missing-require-version

Resources should pin the version they require with `requireVersion`. This rule is off by default;
enable it to require reproducible documents.

## Expression rules

The expression rules check strings in the places DSC evaluates them: parameter default values,
variables, output values and conditions, user-defined function bodies, and the name, condition,
`dependsOn` entries, copy settings, and properties of each resource. A parameter default value or
variable is only an expression when the whole value is a string; DSC uses object and array values
as written, so strings nested in them aren't checked. The properties of a group resource, such as
`Microsoft.DSC/Group`, hold an embedded configuration document that DSC evaluates separately, so
they aren't checked against the outer document.

### invalid-expression

Every string that starts with `[` is evaluated as an expression and must parse. To use a literal
string that starts with a bracket, escape it with `[[`. When the expression is the default value of
a `secureString` or `secureObject` parameter, the diagnostic omits the expression and the parser
error, because they can contain a secret.

```yaml
properties:
  output: "[concat('unterminated)]"   # invalid-expression
  literal: "[[not an expression]"     # fine
```

### unknown-function

Expressions may only call built-in functions or user-defined functions declared in `functions`.
Function names are case-sensitive; the diagnostic suggests the correct casing when the name
matches a built-in function except for case.

### invalid-argument-count

Function calls must pass the number of arguments the function accepts. The message is the same one
DSC raises at runtime.

### invalid-argument-type

Literal arguments must be of a kind the function accepts at that position. Arguments that are
nested expressions aren't checked, because their value is only known at runtime.

### undefined-parameter

`parameters()` must reference a parameter declared in `parameters`. Calls inside user-defined
function bodies refer to the function's own parameters and aren't checked.

### undefined-variable

`variables()` must reference a variable declared in `variables`.

### condition-not-expression

A `condition` on a resource or output must be an expression that evaluates to `true`. A plain
string such as `"true"` never does, so the item is always skipped.

## Parameter rules

### no-unused-parameters

Declared parameters should be referenced by at least one expression.

### no-unused-variables

Declared variables should be referenced by at least one expression.

### parameter-default-type

A literal default value must satisfy the parameter's type, `allowedValues`, value limits, and
length limits. Default values that are expressions aren't checked. An escaped string such as
`"[[x]"` is checked as the value DSC uses, `"[x]"`.

### invalid-parameter-constraints

Parameter constraints must be consistent with each other and with the parameter type: `minValue`
can't exceed `maxValue`, `minLength` can't exceed `maxLength`, value limits only apply to `int`
parameters, length limits only apply to `string`, `secureString`, and `array` parameters,
`allowedValues` can't be empty, and every allowed value must be of the parameter's type.

### secure-parameter-default

A `secureString` or `secureObject` parameter shouldn't have a hard-coded default value, because
anyone with the document can read it. An empty string or an expression is allowed.

### prefer-secure-type

A `string` or `object` parameter whose name suggests a secret, such as `password`, `token`, or
`apiKey`, should use the `secureString` or `secureObject` type.

### secure-value-in-output

Outputs shouldn't use a secure parameter or a `secret()` value. DSC only withholds outputs whose
declared type is `secureString` or `secureObject`; it doesn't track where a value came from. An
output of any other type that extracts a plain value, such as
`[parameters('password').secureString]`, returns that value in the result. Outputs declared as
`secureString` or `secureObject` aren't reported, because DSC never returns them.

### use-parameter-description

Parameters should have a description. This rule is off by default.

### undeclared-parameter-value

When you pass parameters with `--parameters` or `--parameters-file`, every supplied value must
match a declared parameter.

### missing-parameter-value

When you pass parameters, every declared parameter without a default value must be supplied a
value. Without parameters this rule doesn't run, because a document can declare required
parameters on purpose.

### parameter-value-type

When you pass parameters, every supplied value must satisfy the parameter's type and constraints.
A `secureString` value must be supplied as a plain string, not in the wrapped
`{"secureString": "..."}` form that a default value can use.

## Dependency rules

### unresolved-dependency

Every `dependsOn` entry and every `reference(resourceId(...))` call must point at a resource
declared at the same level. The diagnostic distinguishes a missing resource from a resource whose
type doesn't match. Resources in copy loops aren't checked, because their names are only known at
runtime. `dependsOn` entries match resource names as written, the way DSC resolves them. A
`reference()` call matches the name DSC evaluates, so it isn't reported when a resource at the same
level has a name that is an expression.

```yaml
resources:
- name: first
  type: Microsoft.DSC.Debug/Echo
- name: second
  type: Microsoft.DSC.Debug/Echo
  dependsOn:
  - "[resourceId('Microsoft.DSC.Debug/Echo', 'missing')]"   # unresolved-dependency
```

### circular-dependency

Resource dependencies must not form a cycle, including a resource that depends on itself. The
message shows the cycle, like `a -> b -> a`.

### reference-without-depends-on

A resource that references another resource with `reference()` must list it in `dependsOn`. DSC
only makes the output of resources that have already run available to `reference()`, so the
reference fails when the referenced resource is declared later.

### duplicate-depends-on

A dependency should be listed once.

## Deprecation rules

### copy-loop-deprecated

Copy loops are deprecated and will be removed in a future release. See
[issue #1429][01] for details.

### copy-options-unsupported

Copy loops don't support the `mode` and `batchSize` options; DSC rejects a document that sets
them.

### security-context-in-metadata

The security context should be set with `directives.securityContext` rather than the deprecated
`Microsoft.DSC` metadata.

### security-context-conflict

When both the metadata and the directive set a security context, they must agree; DSC refuses to
run the document otherwise.

### dsc-version-unsatisfied

The `directives.version` requirement should be satisfied by the version of DSC in use. The rule
checks the version of the `dsc` command that runs the linter.

## Environment rules

### system-root-unused

When you pass `--system-root`, at least one expression in the document should call `systemRoot()`.
Otherwise the document doesn't target the specified root.

### hardcoded-system-path

Absolute operating system paths such as `C:\Windows` or `/etc` should be built with `systemRoot()`
so the document can target another root. This rule is off by default because many documents
intentionally target the running system.

## Resource rules

These rules run after resource discovery and are skipped with `--no-discovery`.

### discovery-failed

Resource discovery must succeed for the resource rules to run. When it fails, the command reports
this diagnostic and continues with the static results only. This rule has a fixed level.

### unknown-resource-type

Each resource type must be installed. This rule reports a warning rather than an error, because a
document can be valid on a machine that has the resource.

### unsatisfied-require-version

An installed version of the resource type must satisfy `requireVersion`.

### deprecated-resource

Resources shouldn't use resource types whose manifest marks them as deprecated. The diagnostic
includes the deprecation message from the manifest.

### property-schema-mismatch

Literal property values must validate against the resource's embedded schema. Escaped strings such
as `"[[x]"` are validated as the value DSC passes to the resource, `"[x]"`. Properties that hold
expressions aren't checked, because their value is only known at runtime. When an expression sits
inside an `anyOf` or `oneOf`, the rule only reports the failure if every alternative fails for a
reason the expression can't change, such as a literal sibling with the wrong type. The rule never
invokes the resource to retrieve its schema.

### nested-resources-unsupported

Only group and adapter resources can declare nested `resources`.

### exist-false-unsupported

A resource type must have the `delete` or `setHandlesExist` capability for `_exist: false` to
work. Without either, `dsc config set` fails for the resource.

### canonical-type-name

Resource types should be written with the casing their manifest uses.

<!-- Link reference definitions -->
[01]: https://github.com/PowerShell/DSC/issues/1429
