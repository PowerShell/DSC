// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Defines command plug-ins for DSC.
//!
//! A command plug-in adds a top-level command to a DSC host, like the `dsc` CLI, without being
//! compiled into the host. A plug-in is described by a [`PluginManifest`], which names the command
//! and the executable that implements it. The host discovers the manifests with
//! [`discover_plugins()`], finds the plug-in that provides a command with [`find_plugin()`], and
//! runs the executable that [`DscPlugin::resolve_invocation()`] returns.
//!
//! Plug-ins differ from extensions. DSC calls an extension with a fixed contract while it
//! processes an operation. A person or a script calls a plug-in command, and the plug-in defines
//! its own arguments and output.
//!
//! [`PluginManifest`]: plugin_manifest::PluginManifest
//! [`discover_plugins()`]: discovery::discover_plugins
//! [`find_plugin()`]: discovery::find_plugin
//! [`DscPlugin::resolve_invocation()`]: dscplugin::DscPlugin::resolve_invocation

pub mod command_name;
pub mod discovery;
pub mod dscplugin;
pub mod plugin_manifest;
