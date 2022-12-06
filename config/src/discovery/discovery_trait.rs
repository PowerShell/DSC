use dscerror::DscError;
use crate::dscresources::dscresource::DscResource;

use super::*;

pub trait ResourceDiscovery {
    fn discover(&self, filter: Option<String>) -> Result<Box<dyn Iterator<Item = DscResource>>, DscError>;
}
