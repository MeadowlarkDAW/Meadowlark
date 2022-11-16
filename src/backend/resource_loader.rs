use basedrop::{Collector, Shared};
use meadowlark_core_types::time::SampleRate;
use pcm_loader::{error::PcmLoadError, PcmLoader, PcmRAM, PcmRAMType, ResampleQuality};
use std::path::PathBuf;

use crate::util::TwoXHashMap;

#[derive(Default, Debug, Clone, PartialEq, Hash, Eq)]
pub struct PcmKey {
    pub path: PathBuf,

    pub resample_to_project_sr: bool,
    pub resample_quality: ResampleQuality,
    /* TODO
    /// The amount of doppler stretching to apply.
    ///
    /// By default this is `1.0` (no doppler stretching).
    //pub doppler_stretch_ratio: f64,
     */
}

pub struct ResourceLoader {
    pcm_loader: PcmLoader,

    loaded: TwoXHashMap<PcmKey, Shared<PcmRAM>>,

    /// The resource to send when the resource could not be loaded.
    empty_pcm: Shared<PcmRAM>,

    project_sr: SampleRate,

    collector: Collector,
}

impl ResourceLoader {
    pub fn new(project_sample_rate: SampleRate) -> Self {
        let collector = Collector::new();

        let empty_pcm = Shared::new(
            &collector.handle(),
            PcmRAM::new(PcmRAMType::F32(vec![Vec::new()]), project_sample_rate.as_u32()),
        );

        Self {
            pcm_loader: PcmLoader::new(),
            loaded: Default::default(),
            empty_pcm,
            project_sr: project_sample_rate,
            collector,
        }
    }

    pub fn load_pcm(&mut self, key: &PcmKey) -> (Shared<PcmRAM>, Result<(), PcmLoadError>) {
        match self.try_load(key) {
            Ok(pcm) => (pcm, Ok(())),
            Err(e) => {
                log::error!("{}", e);

                // Send an empty PCM resource instead.
                (Shared::clone(&self.empty_pcm), Err(e))
            }
        }
    }

    pub fn try_load(&mut self, key: &PcmKey) -> Result<Shared<PcmRAM>, PcmLoadError> {
        log::trace!("Loading PCM file: {:?}", &key.path);

        if let Some(pcm) = self.loaded.get(key) {
            // Resource is already loaded.
            log::trace!("PCM file already loaded");
            return Ok(Shared::clone(pcm));
        }

        let target_sample_rate =
            if key.resample_to_project_sr { Some(self.project_sr.as_u32()) } else { None };

        let pcm =
            self.pcm_loader.load(&key.path, target_sample_rate, key.resample_quality, None)?;

        let pcm = Shared::new(&self.collector.handle(), pcm);

        self.loaded.insert(key.to_owned(), Shared::clone(&pcm));

        log::trace!("Successfully loaded PCM file");

        Ok(pcm)
    }

    /// Drop all of the loaded resources that are no longer being used.
    pub fn collect(&mut self) {
        // If no other extant Shared pointers to the resource exists, then
        // remove that entry.
        self.loaded.retain(|_, pcm| Shared::get_mut(pcm).is_none());

        self.collector.collect();
    }
}
