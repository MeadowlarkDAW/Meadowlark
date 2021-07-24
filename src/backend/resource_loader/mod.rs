use basedrop::Handle;
use std::error::Error;
use std::fmt;

pub mod pcm;
pub use pcm::{AnyPcm, MonoPcm, PcmLoadError, PcmLoader, StereoPcm};

pub struct ResourceLoader {
    pub pcm_loader: PcmLoader,
}

impl ResourceLoader {
    pub fn new(coll_handle: Handle) -> Self {
        Self {
            pcm_loader: PcmLoader::new(coll_handle),
        }
    }

    pub fn collect(&mut self) {
        self.pcm_loader.collect();
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub enum ResourceLoadError {
    PCM(PcmLoadError),
}

impl Error for ResourceLoadError {}

impl fmt::Display for ResourceLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceLoadError::PCM(e) => write!(f, "Load error: {}", e),
        }
    }
}

impl From<PcmLoadError> for ResourceLoadError {
    fn from(e: PcmLoadError) -> Self {
        ResourceLoadError::PCM(e)
    }
}
