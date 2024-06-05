# Completion Item Provider Sample

This is a prototype VS Code DSCv3 completion provider.

### Build
1) Make sure you have Node.js installed.
2) `npm install -g @vscode/vsce`
3) `cd <EnlistmentRoot>\vscode\DscExt`
4) `vsce package`
5) `dscv3*.vsix` generated

### Install

To install a .vsix file in VS Code:

1) Go to the Extensions view.
2) Click Views and More Actions...
3) Select Install from VSIX...

or

in your terminal, run:
`code --install-extension dscv3-completion-provider-0.0.1.vsix`