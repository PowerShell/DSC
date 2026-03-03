// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{borrow::Borrow, collections::HashSet, ops::{Deref, DerefMut}};

use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{schemas::dsc_repo::DscRepoSchema, types::Tag};

/// Wraps a [`HashSet`] of [`Tag`] instances to enable defining a reusable canonical JSON Schema for
/// manifests, resources, and extensions.
///
/// This type is a minimal wrapper around `HashSet<Tag>`. It provides no additional functionality over
/// the underlying type. It implements the [`AsRef`], [`Borrow`], [`Deref`], and [`DerefMut`]
/// traits to enable ergonomically using instances of [`TagList`] as the underlying hash set.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, DscRepoSchema)]
#[serde(into = "Vec<Tag>")]
#[schemars(
    title = t!("schemas.definitions.tags.title"),
    description = t!("schemas.definitions.tags.description"),
    extend(
        "markdownDescription" = t!("schemas.definitions.tags.markdownDescription"),
    )
)]
#[dsc_repo_schema(
    base_name = "tags",
    folder_path = "definitions",
)]
pub struct TagList(HashSet<Tag>);

impl TagList {
    /// Creates an empty [`TagList`] with the default capacity and hasher.
    pub fn new() -> Self {
        Self(HashSet::new())
    }

    /// Creates an empty [`TagList`] with the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(HashSet::with_capacity(capacity))
    }

    /// Indicates whether the list is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

// Treat a reference to `TagList` as a reference to the underlying `HashSet`
impl AsRef<HashSet<Tag>> for TagList {
    fn as_ref(&self) -> &HashSet<Tag> {
        &self.0
    }
}

// Enable using non-mutating `HashSet` methods directly on `TagList`
impl Deref for TagList {
    type Target = HashSet<Tag>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Enable using mutating `HashSet` methods directly on `TagList`
impl DerefMut for TagList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// Enable borrowing a `TagList` as the underlying `HashSet`
impl Borrow<HashSet<Tag>> for TagList {
    fn borrow(&self) -> &HashSet<Tag> {
        &self.0
    }
}

// Enable iterating over the underlying `HashSet` to use in ergonomic `for` loops
impl IntoIterator for TagList {
    type Item = Tag;
    type IntoIter = std::collections::hash_set::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

// Enable creating a TagList from iterators, as with the `collect()` method.
impl FromIterator<Tag> for TagList {
   fn from_iter<T: IntoIterator<Item = Tag>>(iter: T) -> Self {
        let mut set = HashSet::<Tag>::default();
        set.extend(iter);

        Self(set)
   }
}

// Enable converting from a `TagList` to a sorted vector of tags - used for serialization to ensure
// that the serialized data is deterministic.
impl From<TagList> for Vec<Tag> {
    fn from(value: TagList) -> Self {
        let mut list: Vec<Tag> = value.into_iter().collect();
        list.sort();

        list
    }
}
