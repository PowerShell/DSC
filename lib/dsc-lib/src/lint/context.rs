// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::collections::{HashMap, HashSet};

use serde_json::Value;

use crate::{
    configure::config_doc::{Configuration, Parameter, Resource},
    dscerror::DscError,
    dscresources::dscresource::DscResource,
    functions::FunctionDispatcher,
};

use super::{expression_index::ExpressionIndex, options::LintOptions, pointer::JsonPointer};

/// A resource declaration and where it sits in the document.
pub struct ResourceSite<'a> {
    /// The pointer to the resource declaration.
    pub pointer: JsonPointer,
    /// The resource declaration.
    pub resource: &'a Resource,
    /// The index of the resource that declares this one in its nested `resources`, if any.
    pub parent: Option<usize>,
}

/// The outcome of discovering the resource type a declaration uses.
pub struct ResolvedResource {
    /// The installed resource that satisfies the declaration, if one was found.
    pub found: Option<DscResource>,
    /// Whether any version of the resource type is installed, regardless of `requireVersion`.
    pub type_exists: bool,
}

/// Everything a rule can inspect.
pub struct LintContext<'a> {
    /// The document as parsed, for checks that depend on how it was written.
    pub raw: &'a Value,
    /// The document as a typed configuration.
    pub config: &'a Configuration,
    /// Every resource declaration, depth first, with nested resources after their parent.
    pub resources: Vec<ResourceSite<'a>>,
    /// Every expression in the document, parsed once.
    pub expressions: ExpressionIndex,
    /// The options of the lint run.
    pub options: &'a LintOptions,
    /// The built-in functions, for checking calls without evaluating them.
    pub functions: &'a FunctionDispatcher,
    /// The parameter values supplied for the run, when any were.
    pub supplied_parameters: Option<HashMap<String, Value>>,
    /// The discovered resource for each entry of `resources`, when discovery ran.
    pub discovered: Option<Vec<ResolvedResource>>,
}

impl LintContext<'_> {
    /// Whether a string is an expression, as opposed to a plain or escaped (`[[`) string.
    ///
    /// # Arguments
    ///
    /// * `text` - The string to classify.
    #[must_use]
    pub fn is_expression(text: &str) -> bool {
        text.starts_with('[') && !text.starts_with("[[")
    }

    /// Group the resources that share a parent, in document order.
    ///
    /// # Returns
    ///
    /// The indexes into `resources` for each group, top-level resources first.
    #[must_use]
    pub fn sibling_groups(&self) -> Vec<Vec<usize>> {
        let mut groups: Vec<(Option<usize>, Vec<usize>)> = Vec::new();
        for (index, site) in self.resources.iter().enumerate() {
            match groups.iter_mut().find(|(parent, _)| *parent == site.parent) {
                Some((_, members)) => members.push(index),
                None => groups.push((site.parent, vec![index])),
            }
        }
        groups.into_iter().map(|(_, members)| members).collect()
    }

    /// The raw value of a resource declaration.
    ///
    /// # Arguments
    ///
    /// * `index` - The index into `resources`.
    #[must_use]
    pub fn raw_resource(&self, index: usize) -> Option<&Value> {
        self.resources.get(index).and_then(|site| site.pointer.resolve(self.raw))
    }

    /// The declared parameter with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The parameter name.
    #[must_use]
    pub fn declared_parameter(&self, name: &str) -> Option<&Parameter> {
        self.config.parameters.as_ref().and_then(|parameters| parameters.get(name))
    }

    /// The fully qualified names (`namespace.member`) of the user-defined functions.
    #[must_use]
    pub fn user_function_names(&self) -> HashSet<String> {
        let mut names = HashSet::new();
        if let Some(functions) = &self.config.functions {
            for function in functions {
                for member in function.members.keys() {
                    names.insert(format!("{}.{member}", function.namespace));
                }
            }
        }
        names
    }

    /// The text of an error without the error-kind prefix, for use in a diagnostic message.
    ///
    /// # Arguments
    ///
    /// * `error` - The error to describe.
    #[must_use]
    pub fn error_message(error: &DscError) -> String {
        match error {
            DscError::Parser(message) | DscError::Validation(message) => message.clone(),
            other => other.to_string(),
        }
    }
}

/// Flatten the resources of a configuration into sites, depth first.
///
/// # Arguments
///
/// * `config` - The configuration.
#[must_use]
pub fn collect_resource_sites(config: &Configuration) -> Vec<ResourceSite<'_>> {
    let mut sites = Vec::new();
    collect_sites(&config.resources, &JsonPointer::root().push("resources"), None, &mut sites);
    sites
}

/// Append the sites for a list of resources and their nested resources.
///
/// # Arguments
///
/// * `resources` - The resources to add.
/// * `pointer` - The pointer to the `resources` array.
/// * `parent` - The index of the resource that declares this list, if any.
/// * `sites` - Where sites are appended.
fn collect_sites<'a>(resources: &'a [Resource], pointer: &JsonPointer, parent: Option<usize>, sites: &mut Vec<ResourceSite<'a>>) {
    for (index, resource) in resources.iter().enumerate() {
        let resource_pointer = pointer.push_index(index);
        let site_index = sites.len();
        sites.push(ResourceSite {
            pointer: resource_pointer.clone(),
            resource,
            parent,
        });
        if let Some(children) = &resource.resources {
            collect_sites(children, &resource_pointer.push("resources"), Some(site_index), sites);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_nested_resources_depth_first() {
        let config: Configuration = serde_yaml::from_str(r"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: group
  type: Microsoft.DSC/Group
  properties:
    resources:
    - name: inner
      type: Microsoft.DSC.Debug/Echo
      properties:
        output: hi
  resources:
  - name: nested
    type: Microsoft.DSC.Debug/Echo
- name: last
  type: Microsoft.DSC.Debug/Echo
").unwrap();
        let sites = collect_resource_sites(&config);
        let pointers: Vec<&str> = sites.iter().map(|site| site.pointer.as_str()).collect();
        assert_eq!(pointers, vec!["/resources/0", "/resources/0/resources/0", "/resources/1"]);
        assert_eq!(sites[1].parent, Some(0));
        assert_eq!(sites[2].parent, None);
    }

    #[test]
    fn classifies_expressions() {
        assert!(LintContext::is_expression("[parameters('x')]"));
        assert!(!LintContext::is_expression("[[literal]"));
        assert!(!LintContext::is_expression("plain"));
    }
}
