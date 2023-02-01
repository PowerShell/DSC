use crate::dscresources::dscresource::DscResource;

pub trait ResourceDiscovery {
    fn discover(&self) -> Box<dyn Iterator<Item = DscResource>>;
}
