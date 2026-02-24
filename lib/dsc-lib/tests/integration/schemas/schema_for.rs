// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! These integration tests ensure that we can call [`schemars::schema_for`] against the items
//! defined in [`dsc-lib`] without raising panics when using transform functions to munge the
//! generated schemas.

// use test_case::test_case;

/// Generates a test function that ensures calling [`schemars::schema_for`] on the given type does
/// not panic. Helps ensure that we use the transforms correctly.
macro_rules! test_schema_for {
    ($head:ident $(:: $tail:tt)+) => {
        test_schema_for!( $head :: ; $($tail),* );
    };

    ($($module:ident ::)+ ; $type:ident) => {
            #[test] fn $type() {
                use schemars::schema_for;
                schema_for!($($module ::)+ $type);
            }
    };

    ($($module:ident ::)+ ; $head:ident , $($tail:ident),+) => {
        test_schema_for!( $($module ::)* $head :: ; $($tail),* );
    };
}

#[allow(non_snake_case)]
#[cfg(test)] mod dsc_lib {
    #[cfg(test)] mod configure {
        #[allow(unused_must_use)]
        #[cfg(test)] mod config_doc {
            test_schema_for!(dsc_lib::configure::config_doc::SecurityContextKind);
            test_schema_for!(dsc_lib::configure::config_doc::Operation);
            test_schema_for!(dsc_lib::configure::config_doc::ExecutionKind);
            test_schema_for!(dsc_lib::configure::config_doc::Process);
            test_schema_for!(dsc_lib::configure::config_doc::RestartRequired);
            test_schema_for!(dsc_lib::configure::config_doc::MicrosoftDscMetadata);
            test_schema_for!(dsc_lib::configure::config_doc::Metadata);
            test_schema_for!(dsc_lib::configure::config_doc::UserFunction);
            test_schema_for!(dsc_lib::configure::config_doc::UserFunctionDefinition);
            test_schema_for!(dsc_lib::configure::config_doc::UserFunctionParameter);
            test_schema_for!(dsc_lib::configure::config_doc::UserFunctionOutput);
            test_schema_for!(dsc_lib::configure::config_doc::Configuration);
            test_schema_for!(dsc_lib::configure::config_doc::Parameter);
            test_schema_for!(dsc_lib::configure::config_doc::DataType);
            test_schema_for!(dsc_lib::configure::config_doc::CopyMode);
            test_schema_for!(dsc_lib::configure::config_doc::Copy);
            test_schema_for!(dsc_lib::configure::config_doc::Plan);
            test_schema_for!(dsc_lib::configure::config_doc::Identity);
            test_schema_for!(dsc_lib::configure::config_doc::Sku);
            test_schema_for!(dsc_lib::configure::config_doc::Resource);
        }
        #[allow(unused_must_use)]
        #[cfg(test)] mod config_results {
            test_schema_for!(dsc_lib::configure::config_result::MessageLevel);
            test_schema_for!(dsc_lib::configure::config_result::ResourceMessage);
            test_schema_for!(dsc_lib::configure::config_result::ResourceGetResult);
            test_schema_for!(dsc_lib::configure::config_result::ConfigurationGetResult);
            test_schema_for!(dsc_lib::configure::config_result::ResourceSetResult);
            test_schema_for!(dsc_lib::configure::config_result::GroupResourceSetResult);
            test_schema_for!(dsc_lib::configure::config_result::ConfigurationSetResult);
            test_schema_for!(dsc_lib::configure::config_result::ResourceTestResult);
            test_schema_for!(dsc_lib::configure::config_result::GroupResourceTestResult);
            test_schema_for!(dsc_lib::configure::config_result::ConfigurationTestResult);
            test_schema_for!(dsc_lib::configure::config_result::ConfigurationExportResult);
        }
        #[allow(unused_must_use)]
        #[cfg(test)] mod parameters {
            test_schema_for!(dsc_lib::configure::parameters::SimpleInput);
            test_schema_for!(dsc_lib::configure::parameters::SecureString);
            test_schema_for!(dsc_lib::configure::parameters::SecureObject);
            test_schema_for!(dsc_lib::configure::parameters::SecureKind);
        }
    }
    #[cfg(test)] mod discovery {
        #[allow(unused_must_use)]
        #[cfg(test)] mod command_discovery {
            test_schema_for!(dsc_lib::discovery::command_discovery::ImportedManifest);
        }
    }

    #[cfg(test)] mod dscresources {
        #[allow(unused_must_use)]
        #[cfg(test)] mod dscresource {
            test_schema_for!(dsc_lib::dscresources::dscresource::DscResource);
            test_schema_for!(dsc_lib::dscresources::dscresource::Capability);
            test_schema_for!(dsc_lib::dscresources::dscresource::ImplementedAs);
        }
        #[allow(unused_must_use)]
        #[cfg(test)] mod invoke_result {
            test_schema_for!(dsc_lib::dscresources::invoke_result::GetResult);
            test_schema_for!(dsc_lib::dscresources::invoke_result::ResourceGetResponse);
            test_schema_for!(dsc_lib::dscresources::invoke_result::SetResult);
            test_schema_for!(dsc_lib::dscresources::invoke_result::ResourceSetResponse);
            test_schema_for!(dsc_lib::dscresources::invoke_result::TestResult);
            test_schema_for!(dsc_lib::dscresources::invoke_result::ResourceTestResponse);
            test_schema_for!(dsc_lib::dscresources::invoke_result::ValidateResult);
            test_schema_for!(dsc_lib::dscresources::invoke_result::ExportResult);
            test_schema_for!(dsc_lib::dscresources::invoke_result::ResolveResult);
        }
        #[allow(unused_must_use)]
        #[cfg(test)] mod resource_manifest {
            test_schema_for!(dsc_lib::dscresources::resource_manifest::Kind);
            test_schema_for!(dsc_lib::dscresources::resource_manifest::GetArgKind);
            test_schema_for!(dsc_lib::dscresources::resource_manifest::SetDeleteArgKind);
            test_schema_for!(dsc_lib::dscresources::resource_manifest::InputKind);
            test_schema_for!(dsc_lib::dscresources::resource_manifest::SchemaKind);
            test_schema_for!(dsc_lib::dscresources::resource_manifest::SchemaCommand);
            test_schema_for!(dsc_lib::dscresources::resource_manifest::ReturnKind);
            test_schema_for!(dsc_lib::dscresources::resource_manifest::GetMethod);
            test_schema_for!(dsc_lib::dscresources::resource_manifest::SetMethod);
            test_schema_for!(dsc_lib::dscresources::resource_manifest::TestMethod);
            test_schema_for!(dsc_lib::dscresources::resource_manifest::DeleteMethod);
            test_schema_for!(dsc_lib::dscresources::resource_manifest::ValidateMethod);
            test_schema_for!(dsc_lib::dscresources::resource_manifest::ExportMethod);
            test_schema_for!(dsc_lib::dscresources::resource_manifest::ResolveMethod);
            test_schema_for!(dsc_lib::dscresources::resource_manifest::Adapter);
            test_schema_for!(dsc_lib::dscresources::resource_manifest::AdapterInputKind);
            test_schema_for!(dsc_lib::dscresources::resource_manifest::ListMethod);
            test_schema_for!(dsc_lib::dscresources::resource_manifest::ResourceManifest);
        }
    }

    #[cfg(test)] mod extensions {
        #[allow(unused_must_use)]
        #[cfg(test)] mod discover {
            test_schema_for!(dsc_lib::extensions::discover::DiscoverMethod);
            test_schema_for!(dsc_lib::extensions::discover::DiscoverResult);
        }
        #[allow(unused_must_use)]
        #[cfg(test)] mod dscextension {
            test_schema_for!(dsc_lib::extensions::dscextension::DscExtension);
            test_schema_for!(dsc_lib::extensions::dscextension::Capability);
        }
        #[allow(unused_must_use)]
        #[cfg(test)] mod extension_manifest {
            test_schema_for!(dsc_lib::extensions::extension_manifest::ExtensionManifest);
        }
        #[allow(unused_must_use)]
        #[cfg(test)] mod import {
            test_schema_for!(dsc_lib::extensions::import::ImportMethod);
            test_schema_for!(dsc_lib::extensions::import::ImportArgKind);
        }
        #[allow(unused_must_use)]
        #[cfg(test)] mod secret {
            test_schema_for!(dsc_lib::extensions::secret::SecretArgKind);
            test_schema_for!(dsc_lib::extensions::secret::SecretMethod);
        }
    }

}
