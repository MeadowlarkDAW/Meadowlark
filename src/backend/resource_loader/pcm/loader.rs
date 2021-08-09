use std::error::Error;
use std::fmt;
use std::fs::File;
use std::path::PathBuf;

use basedrop::{Handle, Shared};

use rusty_daw_time::SampleRate;
use symphonia::core::audio::Signal;
use symphonia::core::audio::AudioBufferRef;
use symphonia::core::codecs::{CodecRegistry, DecoderOptions};
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::{Hint, Probe};

// TODO: Eventually we should use disk streaming to store large files. Using this as a stop-gap
// safety check for now.
pub static MAX_FILE_BYTES: u64 = 1_000_000_000;

use super::{AnyPcm, MonoPcm, StereoPcm};
use crate::util::TwoXHashMap;

pub struct PcmLoader {
    loaded: TwoXHashMap<PathBuf, Shared<AnyPcm>>,

    /// The resource to send when the resource could not be loaded.
    empty_pcm: Shared<AnyPcm>,

    codec_registry: &'static CodecRegistry,
    probe: &'static Probe,

    coll_handle: Handle,
}

impl PcmLoader {
    pub fn new(coll_handle: Handle, sample_rate: SampleRate) -> Self {
        let empty_pcm = Shared::new(
            &coll_handle,
            AnyPcm::Mono(MonoPcm::new(Vec::new(), sample_rate)),
        );

        Self {
            loaded: Default::default(),
            empty_pcm,
            codec_registry: symphonia::default::get_codecs(),
            probe: symphonia::default::get_probe(),
            coll_handle,
        }
    }

    pub fn load(&mut self, path: &PathBuf) -> (Shared<AnyPcm>, Result<(), PcmLoadError>) {
        match self.try_load(path) {
            Ok(pcm) => (pcm, Ok(())),
            Err(e) => {
                log::error!("{}", e);

                // Send an "empty" PCM resource instead.
                (Shared::clone(&self.empty_pcm), Err(e))
            }
        }
    }

    fn try_load(&mut self, path: &PathBuf) -> Result<Shared<AnyPcm>, PcmLoadError> {
        log::info!("Loading PCM file: {:?}", path);

        if let Some(pcm) = self.loaded.get(path) {
            // Resource is already loaded.
            log::debug!("PCM file already loaded");
            return Ok(Shared::clone(pcm));
        }

        // Try to open the file.
        let file = File::open(path).map_err(|e| PcmLoadError::PathNotFound((path.clone(), e)))?;

        // Create a hint to help the format registry guess what format reader is appropriate.
        let mut hint = Hint::new();

        // Provide the file extension as a hint.
        if let Some(extension) = path.extension() {
            if let Some(extension_str) = extension.to_str() {
                hint.with_extension(extension_str);
            }
        }

        // Create the media source stream.
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        // Use the default options for format reader, metadata reader, and decoder.
        let format_opts: FormatOptions = Default::default();
        let metadata_opts: MetadataOptions = Default::default();
        let decode_opts: DecoderOptions = Default::default();

        // Probe the media source stream for metadata and get the format reader.
        let mut probed = self
            .probe
            .format(&hint, mss, &format_opts, &metadata_opts)
            .map_err(|e| PcmLoadError::UnkownFormat((path.clone(), e)))?;

        // Get the default track in the audio stream.
        let track = probed
            .format
            .default_track()
            .ok_or_else(|| PcmLoadError::NoTrackFound(path.clone()))?;
        let track_id = track.id;

        // Get info.
        let n_channels = track
            .codec_params
            .channels
            .ok_or_else(|| PcmLoadError::NoChannelsFound(path.clone()))?
            .count();

        // TODO: Support loading multi-channel audio.
        if n_channels > 2 {
            return Err(PcmLoadError::UnkownChannelFormat((
                path.clone(),
                n_channels,
            )));
        }

        let sample_rate = track.codec_params.sample_rate.unwrap_or_else(|| {
            log::warn!("Could not find sample rate. Assuming a sample rate of 44100");
            44100
        });

        let n_frames = track.codec_params.n_frames;

        // Eventually we should use disk streaming to store large files. Using this as a stop-gap
        // safety check for now.
        if let Some(n_frames) = n_frames {
            let total_bytes = n_channels as u64 * n_frames * 4;
            if total_bytes > MAX_FILE_BYTES {
                return Err(PcmLoadError::FileTooLarge(path.clone()));
            }
        }

        // Create a decoder for the track.
        let mut decoder = self
            .codec_registry
            .make(&track.codec_params, &decode_opts)
            .map_err(|e| PcmLoadError::CouldNotCreateDecoder((path.clone(), e)))?;

        let mut decoded_channels = Vec::<Vec<f32>>::new();
        for _ in 0..n_channels {
            decoded_channels.push(Vec::with_capacity(n_frames.unwrap_or(0) as usize));
        }

        let max_frames = MAX_FILE_BYTES / (4 * n_channels as u64);
        let mut total_frames = 0;

        while let Ok(packet) = probed.format.next_packet() {
            // If the packet does not belong to the selected track, skip over it.
            if packet.track_id() != track_id {
                continue;
            }

            match decoder.decode(&packet) {
                Ok(decoded) => {
                    match decoded {
                        AudioBufferRef::F32(d) => {
                            total_frames += d.chan(0).len() as u64;
                            if total_frames > max_frames {
                                return Err(PcmLoadError::FileTooLarge(path.clone()));
                            }
                            for i in 0..n_channels {
                                decoded_channels[i].extend_from_slice(d.chan(i));
                            }
                        }
                        AudioBufferRef::S32(d) => {
                            total_frames += d.chan(0).len() as u64;
                            if total_frames > max_frames {
                                return Err(PcmLoadError::FileTooLarge(path.clone()));
                            }
                            for i in 0..n_channels {
                                for smp in d.chan(i).iter() {
                                    decoded_channels[i].push(*smp as f32 / i32::MAX as f32);
                                }
                            }
                        } // TODO: Ask creator of symphonia if we are able to get other sample formats from the decoder to save memory.
                    }
                }
                Err(symphonia::core::errors::Error::DecodeError(err)) => {
                    // Decode errors are not fatal. Print the error message and try to decode the next
                    // packet as usual.
                    log::warn!("decode error: {}", err);
                }
                Err(e) => return Err(PcmLoadError::ErrorWhileDecoding((path.clone(), e))),
            }
        }

        let pcm = if n_channels == 1 {
            AnyPcm::Mono(MonoPcm::new(
                decoded_channels.pop().unwrap(),
                SampleRate(sample_rate as f64),
            ))
        } else {
            // Two channels. TODO: Support loading multi-channel audio.

            let right = decoded_channels.pop().unwrap();
            let left = decoded_channels.pop().unwrap();

            AnyPcm::Stereo(StereoPcm::new(left, right, SampleRate(sample_rate as f64)))
        };

        decoder.close();

        let pcm = Shared::new(&self.coll_handle, pcm);

        self.loaded.insert(path.to_owned(), Shared::clone(&pcm));

        log::debug!("Successfully loaded PCM file");

        Ok(pcm)
    }

    /// Drop all PCM resources not being currently used.
    pub fn collect(&mut self) {
        // If no other extant Shared pointers to the resource exists, then
        // remove that entry.
        self.loaded.retain(|_, pcm| Shared::get_mut(pcm).is_none());
    }
}

#[derive(Debug)]
pub enum PcmLoadError {
    PathNotFound((PathBuf, std::io::Error)),
    UnkownFormat((PathBuf, symphonia::core::errors::Error)),
    NoTrackFound(PathBuf),
    NoChannelsFound(PathBuf),
    UnkownChannelFormat((PathBuf, usize)),
    FileTooLarge(PathBuf),
    CouldNotCreateDecoder((PathBuf, symphonia::core::errors::Error)),
    ErrorWhileDecoding((PathBuf, symphonia::core::errors::Error)),
}

impl Error for PcmLoadError {}

impl fmt::Display for PcmLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use PcmLoadError::*;

        match self {
            PathNotFound((path, e)) => write!(f, "Failed to load PCM resource {:?}: file not found | {}", path, e),
            UnkownFormat((path, e)) => write!(
                f,
                "Failed to load PCM resource: format not supported | {} | path: {:?}",
                e,
                path,
            ),
            NoTrackFound(path) => write!(f, "Failed to load PCM resource: no default track found | path: {:?}", path),
            NoChannelsFound(path) => write!(f, "Failed to load PCM resource: no channels found | path: {:?}", path),
            UnkownChannelFormat((path, n_channels)) => write!(
                f,
                "Failed to load PCM resource: unkown channel format | {} channels found | path: {:?}",
                n_channels,
                path
            ),
            FileTooLarge(path) => write!(
                f,
                "Failed to load PCM resource: file is too large | maximum is {} bytes | path: {:?}",
                MAX_FILE_BYTES,
                path
            ),
            CouldNotCreateDecoder((path, e)) => write!(
                f,
                "Failed to load PCM resource: failed to create decoder | {} | path: {:?}",
                e,
                path
            ),
            ErrorWhileDecoding((path, e)) => write!(
                f,
                "Failed to load PCM resource: error while decoding | {} | path: {:?}",
                e,
                path
            ),
        }
    }
}
