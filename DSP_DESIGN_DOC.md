# Meadowlark DSP Design Document

Here I'll outline the goals and non-goals for built-in DSP effects that will be included in the RustyDAW project (and by extension Meadowlark). I'll also include some guidelines for developers.

### Goals
The highest priority right now is a suite essential plugins for mixing (and essential effects for manipulating audio clips). The goal is to aim for at-least "pretty good" quality. We of course don't have the resources to compete with industry leaders such as FabFilter and iZotope. But the quality of the built-in effects should be good enough to where most producers can acheive a decent/statisfactory mix with only internal plugins.

To ease development of this DSP, I highly recommend porting & modifying already-existing open source plugin DSP if one is available for our use case. There is no need to reinvent the wheel when there is already great DSP out there, (especially since we have such a small team at the moment). I'll highlight some of my favorite open source effects and synths in this document I feel would be helpful to port as a starting point.

Synthesizers and exotic effects are currently lower on the priority list, but are still welcome for contribution right now!

Also, SIMD optimizations should be used when possible (but of course focus on just getting the DSP to work first before optimizing it).

### Non-Goals
The goal of this plugin project is **NOT** to create a reusable shared DSP library (I believe those to be more hassle than they are worth, especially when it comes to proper SIMD optimizations and "tasteful magic numbers/experimentation" for plugins). The goal of this plugin project is to simply provide standalone "plugins", each with their own separate and optimized DSP implementation. We are however free to reference/copy-paste portions of DSP across plugins as we see fit (as long as the other plugins are also GPLv3).

Also, like what was mentioned above, we simply don't have the resources to compete with industry leaders such as FabFilter and iZotope. But the quality of the built-in effects should be at-least good enough to where most producers can acheive a decent/statisfactory mix with only internal plugins.

That being said, any kind of effect/synth idea is welcome, it's just that we should focus on the essentials first.

Also the all of these plugins will (initially) be GUI-less. It is important that all our plugin GUIs are designed around the needs of the DSP and not the other way around.

## Developer Guidelines
The link to the kanban-style [`project-board`].

Any ported plugin DSP should be added to the [`rusty-daw-plugin-ports`] repo, and any original/modified plugin DSP should be added to the [`rusty-daw-plugins`] repo. Please take careful note of what pieces of code are borrowed from ported plugins, and make apparent the appropriate credit and license of those plugins where appropriate. All of our code will be GPLv3 (although we may also consider using AGPL).

- An [`example gain dsp`] crate is provided to demonstrate how to use the RustyDAW types as well as portable SIMD.
- An [`example gain plugin`] is provided in the `rusty-daw-plugins` repo. It demonstrates how to develop and test RustyDAW DSP as a (GUI-less) VST plugin.
- Note that the `Audio Clip Effects` are all *offline* effects, so the intended workflow to develop those effects is to just load a WAV file in a standalone terminal app using a crate like [`hound`], modify it, and then export it to another WAV file.

When possible, prefer to use types from the [`rusty-daw-core`] crate (Which includes types such as `SampleRate`, `MusicalTime`, `SampleTime`, `Seconds`, `MonoBlockBuffer`, and `StereoBlockBuffer`). Also please use the `ParamF32`/`ParamF32Handle` types which conveniently and automatically smooths parameter inputs for you.

Prefer to use the `SVF` filter in place of all biquad filters. It is simply just a better quality filter. For reference here is an [`implementation of the SVF filter`], and here are the [`Cytomic Technical Papers`] explaining the SVF filter in technical detail.

For resampling algorithms prefer using the "optimal" filters from the [`deip`] paper.

When possible, DSP should be designed around a configurable `MAX_BLOCKSIZE` constant that defines the maximum block size the DAW will send to any effect. This will make allocating buffers on the stack easier, as well as making it easier to optimize by letting the compiler easily elid bounds checking when it knows that the current number of frames is less than or equal to `MAX_BLOCKSIZE`. You can assume that `MAX_BLOCKSIZE` is always a power of 2, and you can expect the block size to be relatively small (in the range of 64 to 256). However, note that the actual number of frames given in a particular process cycle may *not* be a power of 2.

## Audio Clip Effects
Audio Clip Effects are all *offline* effects, meaning they are pre-computed before being sent to the realtime thread. Actual "realtime" effects on audio samples will be done in a separate "sampler" instrument plugin.

These are listed in order of priority with the first being the highest priority:

- [ ] Doppler stretching (simple stretching of audio clips by speeding up or slowing down the sample rate).
  - This can likely be easily accomplished using [`samplerate`] crate. However, an eventual goal is to support automating this over time, so some research needs to be done on how to make this possible. For reference on what I mean take a look at Bitwig's [`Working with audio clips`] section in its manual.
  - The "optimal" designs from the [`deip`] paper could be a great starting point.
- [ ] Reverse effect
  - This one should be really simple
- [ ] Normalize
- [ ] DC Offset
- [ ] Gain / Pan automation
  - Ability to automate gain & pan in an audio clip. For reference look at Bitwig's [`Working with audio clips`] section in its manual.
- [ ] Time-warping (stretch the audio clip without altering pitch / pitch shift the audio clip without altering the length)
  - DAWs like Ableton Live have the ability to select different algorithms that are more optimal for different use cases (i.e. preserve transients, preserve tone, preserve formants, etc.)
  - For reference take a look Live's documentation on its [`Time-warping`] feature and also Bitwig's [`Working with audio clips`] section in its manual. Of course there is a lot here, but I feel this gives a good reference to what users could expect from a high-quality DAW. 
  - The "optimal" designs from the [`deip`] paper could be a great starting point.
- [ ] Split audio clips by transients
  - In theory this shouldn't be *too* tricky to do.
- [ ] Apply convolution from an impulse response from file
  - This should be simple once we have a convolver in Rust
- [ ] Creative effects like formant shifting
- [ ] Xtreme stretch effect (like the PaulStretch algorithm)
  - A Rust implementation of this effect is already being developed in the [`TimeStretch`] crate, so this shouldn't be too hard to include.

## Effect DSP
Note that we should prioritize using the `SVF` filter in place of all biquad filters. It is simply just a better quality filter.
For reference here is an [`implementation of the SVF filter`], and here are the [`Cytomic Technical Papers`] explaining the SVF filter in technical detail.

These are listed in order of priority with the first being the highest priority:

- [ ] Panning
  - Should include various pan laws such as "linear", "circular", etc.
- [ ] Parametric Equalizer
  - We will likely use the `SVF` filter designs described above (at least as a starting point).
  - Typical parametric EQ stuff such as lowpass, highpass, low-shelf, high-shelf, bell, and notch.
  - Lowpass and highpass filters should have multiple intensities (atleast including 6dB/octave, 12db/octave, 24db/octave, and 48db/octave).
  - There needs to be **no** cramping in the high end. This is pretty paramount to the perceived "quality" of a digital parametric EQ.
  - Must sound good when being sharply automated (a technique commonly used in electronic sound design).
  - I quite like the sound and quality of the [`x42 fil4 EQ`], so porting this could be a good starting point.
- [ ] Multiband splitter/merger
- [ ] Basic Soft/Hard Clipper
  - Ability to switch between soft and hard mode.
  - Controls for input gain, threshold, and output gain.
  - I quite like the sound and quality of [`Misstortion`], so porting it could be a great starting point.
  - Ability to split the low end "cutoff" frequency where the wet signal below this cutoff is replaced with the dry signal below this cutoff. This should be easy once we have the "Multiband splitter/merger". This is useful for making distorted bass sounds sound "cleaner".
- [ ] Basic Limiter
  - A standard single-band limiter.
  - Controls for input gain, output gain, threshold, and a drop-down for look-ahead time (1ms, 3ms, 10ms, etc).
  - I'm not sure what goes into making a limiter "sound good". We need to research this some more.
- [ ] Basic Compressor
  - A standard single-band compressor with attack, release, threshold and output gain.
  - In addition to the normal threshold, there should also be an "expander" threshold that boosts signals under a given threshold.
  - Two different detection modes: Peak and RMS
  - Basic lowpass & highpass filter on sidechain input (including when the input signal is used as the "sidechain").
  - I'm not sure what goes into making a compressor "sound good". We need to research this some more.
  - I quite like the sound and quality of the [`x42 darc compressor`], so porting this could be a good starting point.
  - The [`Pressure4`] compressor by Airwindows is also a nice sounding open-source compressor.
- [ ] Waveshaper
  - A waveshaper with common built-in shapes, along with the ability to draw in custom shapes.
  - I quite like the sound of the [`Wolf Shaper`] plugin, so porting that could be a great starting point.
  - Should include controls for a pre-filter and a post-filter.
  - Ability to split the low end "cutoff" frequency where the wet signal below this cutoff is replaced with the dry signal below this cutoff. This should be easy once we have the "Multiband splitter/merger". This is useful for making distorted bass sounds sound "cleaner".
- [ ] Delay effect
  - Sync to tempo or freerun
  - Mono, stereo, & ping-pong modes
  - Lowpass & highpass filter controls on taps.
  - Controls for feedback, & mix.
  - Controls for ducking the delay based on the input signal.
  - Possibly extra effects on the taps such as chorusing, pitch shifting, and waveshaping.
- [ ] Gate
  - A standard gating plugin with attack, release, and threshold.
  - I'm not sure what goes into making a gate "sound good". We need to research this some more.
- [ ] Mid/Side splitter/merger
- [ ] Stereo width adjustment
- [ ] DC Offset
- [ ] Multiband Compressor
  - This should be trivial once we complete the "Basic Compressor" and "Multiband splitter/merger".
- [ ] Chorus effect
- [ ] Phaser effect
- [ ] General-purpose reverb
  - Of course the field of what makes reverbs sound good is a vast and complicated (and a lot of it is hidden away as trade secrets). However, I'll give some resources that could be good starting points:
  - I happen to quite like the sound of the [`built-in reverb in the Vital synth`]. Porting this could be a great starting point.
  - The developer of the amazing Valhalla plugins has made a blog on [`developing reverbs`]. 
  - The technical paper on the [`Freeverb algorithm`].
  - Two other decent open source reverbs include [`Mverb`] and the [`Dragonfly Reverb`].
- [ ] Convolver
- [ ] Bus compressor
  - An analogue-modeled bus compressor like Cytomic's "The Glue".
  - Honestly I'm not sure where to start with this one though, but bus compressor are pretty essential in creating a decent sounding mix, so putting research into this is definitely worth it.
- [ ] Analogue-modeled distortion effects.
  - Porting [`ZamTube`] as well as [`Density`], [`ToTape6`], and [`IronOxide5`] by Airwindows could be a great starting point.
  - Should include controls for a pre-filter and a post-filter.
  - Ability to split the low end "cutoff" frequency where the wet signal below this cutoff is replaced with the dry signal below this cutoff. This should be easy once we have the "Multiband splitter/merger". This is useful for making distorted bass sounds sound "cleaner".
- [ ] Linear phase EQ
- [ ] Dynamic EQ
  - This should be easy once we have a parametric eq and a compressor.
  - This can also double as a de-esser.
- [ ] Analogue-modeled EQ
  - A good starting point could be to port [`Luftikus`], although we should probably add more flexibility in controlling the frequency of bands.
- [ ] Comb filter effect
- [ ] Flanger effect
- [ ] Realtime pitch-shifting effect (grain based)
- [ ] Vocoder
- [ ] Shimmering Reverb
  - We can pretty much just port the amazing [`CloudSeed`] reverb. It sounds fantastic the way it is.
- [ ] Analogue-modeled filters (if we ever want to do an analouge-modeled synth).

## Visualizer DSP

These are listed in order of priority with the first being the highest priority:

- [ ] Oscilloscope DSP
  - Controls for input gain & time window.
  - Should accept incoming MIDI notes to automatically set the time window based on pitch.
  - Ability to freeze the signal in place.
- [ ] Spectrogram DSP
  - Controls for attack/fall rate.
  - Ability to freeze the signal.
- [ ] Phase correlation detection (for phase correlation meters).
- [ ] Loudness detection
  - Detect loudness of a signal with different methods such as RMS and EBU.
  - The [`LUFSMeter`] plugin is a great starting point here.

## Generator DSP

These are listed in order of priority with the first being the highest priority:

- [ ] HADSR envelope generator
  - Preferably with configurable levels of quality.
- [ ] LFO generators
  - Include common LFO shapes such as sine, triangle, square, saw up, saw down, random hold, and random glide.
  - Automatable controls for amplitude, speed, & phase.
  - Support for skewing shapes would be nice too.
  - Preferably with configurable levels of quality.
- [ ] Simple noise generator.
  - Include white noise, pink noise, and brown noise.
- [ ] MIDI-triggered Sampler
  - Automatable controls for hadsr
  - Automatable control for speed/pitch (doppler effect)
    - The "optimal" designs from the [`deip`] paper could be a great starting point.
  - Automatable "start time"
  - Looping controls with fades
- [ ] High-quality oscillators
  - Sine, triangle, saw, square, & pulse.
  - We will likely use Polyblep oscillators.
- [ ] Unison
  - Configurable voice spread.
- [ ] Voice allocator logic
  - Include support for mono, legato, poly, and polyglide.
- [ ] Analouge-modeled oscillators/analogue-modeled adsr envelopes (if we ever want to do an analouge-modeled synth).
- [ ] FM/Ring Oscillators (if we ever want to do an FM synth).
- [ ] Wavetable oscillators (if we ever want to do a wavetable synth).
- [ ] Additive synthesis engine (if we ever want to do an additive synth).
- [ ] Granular synthesis engine (if we ever want to do a granular synth).

## MIDI/Note effects
TODO (A lot of this will depend on exactly how the internal control spec will work, so I'm leaving this blank for now.)


[`samplerate`]: https://crates.io/crates/samplerate
[`deip`]: https://github.com/BillyDM/Awesome-Audio-DSP/blob/main/deip.pdf
[`Time-warping`]: https://www.ableton.com/en/manual/audio-clips-tempo-and-warping/
[`Working with audio clips`]: https://www.bitwig.com/userguide/latest/working_with_audio_events/
[`Cytomic Technical Papers`]: https://cytomic.com/index.php?q=technical-papers
[`implementation of the SVF filter`]: https://github.com/wrl/baseplug/blob/trunk/examples/svf/svf_simper.rs
[`x42 fil4 EQ`]: https://github.com/x42/fil4.lv2/tree/d9fa3861575ac06229ea97e352e887b24c23d975
[`x42 darc compressor`]: https://github.com/x42/darc.lv2/tree/7f1f42b879777e570c83fd566ac28cbfdd51e6fc
[`developing reverbs`]: https://valhalladsp.com/2021/09/20/getting-started-with-reverb-design-part-1-dev-environments/
[`Freeverb algorithm`]: https://ccrma.stanford.edu/~jos/pasp/Freeverb.html
[`built-in reverb in the Vital synth`]: https://github.com/mtytel/vital/blob/main/src/synthesis/effects/reverb.cpp
[`MVerb`]: https://github.com/DISTRHO/MVerb
[`Dragonfly Reverb`]: https://github.com/michaelwillis/dragonfly-reverb
[`CloudSeed`]: https://github.com/xunil-cloud/CloudReverb
[`TimeStretch`]: https://github.com/spluta/TimeStretch
[`Luftikus`]: https://github.com/lkjbdsp/lkjb-plugins/tree/master/Luftikus
[`Wolf Shaper`]: https://github.com/wolf-plugins/wolf-shaper
[`ZamTube`]: https://github.com/zamaudio/zam-plugins/tree/master/plugins/ZamTube
[`Density`]: https://github.com/airwindows/airwindows/tree/master/plugins/LinuxVST/src/Density
[`ToTape6`]: https://github.com/airwindows/airwindows/tree/master/plugins/LinuxVST/src/ToTape6
[`IronOxide5`]: https://github.com/airwindows/airwindows/tree/master/plugins/LinuxVST/src/IronOxide5
[`Pressure4`]: https://github.com/airwindows/airwindows/tree/master/plugins/LinuxVST/src/Pressure4
[`Misstortion`]: https://github.com/nimbletools/misstortion1
[`LUFSMeter`]: https://github.com/klangfreund/LUFSMeter
[`rusty-daw-core`]: https://crates.io/crates/rusty-daw-core
[`packed_simd_2`]: https://crates.io/crates/packed_simd_2
[`rusty-daw-plugin-ports`]: https://github.com/RustyDAW/rusty-daw-plugin-ports
[`rusty-daw-plugins`]: https://github.com/RustyDAW/rusty-daw-plugins
[`project-board`]: https://github.com/MeadowlarkDAW/project-board/projects/2
[`example gain dsp`]: https://github.com/RustyDAW/rusty-daw-plugins/tree/main/example-gain-dsp
[`example gain plugin`]: https://github.com/RustyDAW/rusty-daw-plugins/tree/main/example-gain-plugin
[`hound`]: https://crates.io/crates/hound
