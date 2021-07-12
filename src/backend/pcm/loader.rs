use std::error::Error;
use std::fmt;
use std::fs::File;
use std::path::PathBuf;

use fnv::FnvHashSet;

use symphonia::core::audio::Signal;
use symphonia::core::audio::{AudioBuffer, AudioBufferRef};
use symphonia::core::codecs::{CodecRegistry, DecoderOptions};
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::{Hint, Probe};
use symphonia::core::units::Duration;

// Eventually we should use disk streaming to store large files. Using this as a stop-gap
// safety check for now.
pub static MAX_FILE_BYTES: u64 = 2_000_000_000;

use super::{AnyPcm, MonoPcm, StereoPcm};

pub struct PcmLoader {
    loaded_paths: FnvHashSet<PathBuf>,

    codec_registry: &'static CodecRegistry,
    probe: &'static Probe,
}

impl PcmLoader {
    pub fn new() -> Self {
        Self {
            loaded_paths: FnvHashSet::default(),
            codec_registry: symphonia::default::get_codecs(),
            probe: symphonia::default::get_probe(),
        }
    }

    pub fn load(&mut self, path: &PathBuf) -> Result<AnyPcm, PcmLoadError> {
        log::info!("Loading PCM file: {:?}", path);

        if self.loaded_paths.contains(path) {
            return Err(PcmLoadError::AlreadyLoaded);
        }

        // Try to open the file.
        let file = File::open(path).map_err(|e| PcmLoadError::PathNotFound(e))?;

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
            .map_err(|e| PcmLoadError::UnkownFormat(e))?;

        // Get the default track in the audio stream.
        let track = probed
            .format
            .default_track()
            .ok_or_else(|| PcmLoadError::NoTrackFound)?;
        let track_id = track.id;

        // Get info.
        let n_channels = track
            .codec_params
            .channels
            .ok_or_else(|| PcmLoadError::NoChannelsFound)?
            .count();

        // TODO: Support loading multi-channel audio.
        if n_channels > 2 {
            return Err(PcmLoadError::UnkownChannelFormat(n_channels));
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
                return Err(PcmLoadError::FileTooLarge);
            }
        }

        // Create a decoder for the track.
        let mut decoder = self
            .codec_registry
            .make(&track.codec_params, &decode_opts)
            .map_err(|e| PcmLoadError::CouldNotCreateDecoder(e))?;

        let mut decoded_channels = Vec::<Vec<f32>>::new();
        for _ in 0..n_channels {
            decoded_channels.push(Vec::with_capacity(n_frames.unwrap_or(0) as usize));
        }

        let mut audio_buf: Option<AudioBuffer<f32>> = None;

        let max_frames = MAX_FILE_BYTES / (4 * n_channels as u64);
        let mut total_frames = 0;

        while let Ok(packet) = probed.format.next_packet() {
            // If the packet does not belong to the selected track, skip over it.
            if packet.track_id() != track_id {
                continue;
            }

            match decoder.decode(&packet) {
                Ok(decoded) => {
                    // If this is the first packet, use it to create the intermediate buffer.
                    if audio_buf.is_none() {
                        // Get the buffer specification. This is a description of the decoded audio
                        // buffer's sample format.
                        let spec = decoded.spec().clone();

                        // Get the maximum duration of a packet.
                        let duration = Duration::from(decoded.capacity() as u64);

                        audio_buf = Some(AudioBuffer::new(duration, spec));
                    }

                    if let Some(audio_buf) = audio_buf.as_mut() {
                        audio_buf.clear();

                        match decoded {
                            AudioBufferRef::F32(d) => d.convert(audio_buf),
                            AudioBufferRef::S32(d) => d.convert(audio_buf),
                            // TODO: Ask creator of symphonia if we are able to get other sample formats from the decoder to save memory.
                        }

                        total_frames += audio_buf.chan(0).len() as u64;
                        if total_frames > max_frames {
                            return Err(PcmLoadError::FileTooLarge);
                        }

                        for i in 0..n_channels {
                            decoded_channels[i].extend_from_slice(audio_buf.chan(i));
                        }
                    }
                }
                Err(symphonia::core::errors::Error::DecodeError(err)) => {
                    // Decode errors are not fatal. Print the error message and try to decode the next
                    // packet as usual.
                    log::warn!("decode error: {}", err);
                }
                Err(e) => return Err(PcmLoadError::ErrorWhileDecoding(e)),
            }
        }

        let resource = if n_channels == 1 {
            AnyPcm::Mono(MonoPcm::new(
                decoded_channels.pop().unwrap(),
                sample_rate as f32,
            ))
        } else {
            // Two channels. TODO: Support loading multi-channel audio.

            let right = decoded_channels.pop().unwrap();
            let left = decoded_channels.pop().unwrap();

            AnyPcm::Stereo(StereoPcm::new(left, right, sample_rate as f32))
        };

        decoder.close();

        self.loaded_paths.insert(path.to_owned());

        log::info!("Successfully loaded PCM file");

        Ok(resource)
    }
}

#[derive(Debug)]
pub enum PcmLoadError {
    PathNotFound(std::io::Error),
    UnkownFormat(symphonia::core::errors::Error),
    NoTrackFound,
    NoChannelsFound,
    UnkownSampleFormat,
    UnkownChannelFormat(usize),
    FileTooLarge,
    CouldNotCreateDecoder(symphonia::core::errors::Error),
    ErrorWhileDecoding(symphonia::core::errors::Error),
    AlreadyLoaded,
}

impl Error for PcmLoadError {}

impl fmt::Display for PcmLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use PcmLoadError::*;

        match self {
            PathNotFound(e) => write!(f, "Could not load PCM resource: file not found | {}", e),
            UnkownFormat(e) => write!(
                f,
                "Could not load PCM resource: format not supported | {}",
                e
            ),
            NoTrackFound => write!(f, "Could not load PCM resource: no default track found"),
            NoChannelsFound => write!(f, "Could not load PCM resource: no channels found"),
            UnkownSampleFormat => write!(f, "Could not load PCM resource: unkown sample format"),
            UnkownChannelFormat(n_channels) => write!(
                f,
                "Could not load PCM resource: unkown channel format | {} channels found",
                n_channels
            ),
            FileTooLarge => write!(
                f,
                "Could not load PCM resource: file is too large | maximum is {} bytes",
                MAX_FILE_BYTES
            ),
            CouldNotCreateDecoder(e) => write!(
                f,
                "Could not load PCM resource: failed to create decoder | {}",
                e
            ),
            ErrorWhileDecoding(e) => write!(
                f,
                "Could not load PCM resource: error while decoding | {}",
                e
            ),
            AlreadyLoaded => write!(f, "Could not load PCM resource: resource is already loaded"),
        }
    }
}
