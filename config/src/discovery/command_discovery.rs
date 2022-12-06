// find resources via PATH
// reuse code from https://github.com/PowerShell/MSH/blob/main/config/src/main.rs

use crate::dscerror::DscError;
use crate::discovery::discovery_trait::{ResourceDiscovery};
use crate::dscresources::dscresource::DscResource;

pub struct CommandDiscovery {
    pub command: String,
}

pub struct CommandIterator {
    pub command: String,
}

impl Iterator for CommandIterator {
    type Item = DscResource;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl ResourceDiscovery for CommandDiscovery {
    fn discover(&self, _filter: Option<String>) -> Result<Box<dyn Iterator<Item = DscResource>>, DscError> {
        Ok(Box::new(CommandIterator {
            command: self.command.clone(),
        }))
    }
}
