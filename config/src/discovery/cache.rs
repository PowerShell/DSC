// this would be a local file system cache of last discovery
// can use last modified time to determine if we need to re-discover

use crate::dscresources::dscresource::DscResource;
use std::collections::HashMap;

pub struct DscResourceCache {
    pub cache: HashMap<String, DscResource>,
}

impl Default for DscResourceCache {
    fn default() -> Self {
        Self::new()
    }
}

impl DscResourceCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn add(&mut self, resource: DscResource) {
        self.cache.insert(resource.name.clone(), resource);
    }

    pub fn get(&self, name: &str) -> Option<&DscResource> {
        self.cache.get(name)
    }

    pub fn remove(&mut self, name: &str) {
        self.cache.remove(name);
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }

    pub fn read_from_disk(&mut self) {
        // TODO: read from file
    }

    pub fn write_to_disk(&self) {
        // TODO: write to file
    }
}
