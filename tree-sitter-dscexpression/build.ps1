#requires -version 7.4

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

# check if tools are installed

$PSNativeCommandUseErrorActionPreference = $true
$ErrorActionPreference = 'Stop'

npx tree-sitter generate
node-gyp configure
node-gyp build
npx tree-sitter test
