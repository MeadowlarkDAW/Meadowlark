# Meadowlark DSP Design Document

Here I'll outline the goals and non-goals for built-in DSP effects that will be included in the RustyDAW project (and by extension Meadowlark). I'll also include some guidelines for developers.

### Goals
The highest priority right now is a suite essential plugins for mixing (and essential effects for manipulating audio clips). The goal is to aim for at-least "pretty good" quality. We of course don't have the resources to compete with industry leaders such as FabFilter and iZotope. But the quality of the built-in effects should be good enough to where most producers can acheive a satisfactory mix with only internal plugins.

To ease development of this DSP, I highly recommend porting & modifying already existing open source plugin DSP if one is available for our use case. There is no need to reinvent the wheel when there is already great DSP out there, (especially since we have such a small team at the moment). I'll highlight some of my favorite open source effects and synths in this document I feel would be helpful to port as a starting point. Of course you can still research and develop your own DSP if you wish.

Synthesizers and exotic effects are currently lower on the priority list, but are still welcome for contribution right now!

Also, SIMD optimizations should be used when possible (but of course focus on just getting the DSP to work first before optimizing it).

### Non-Goals
The goal of this plugin project is **NOT** to create a reusable shared DSP library (I believe those to be more hassle than they are worth, especially when it comes to proper SIMD optimizations and "tasteful magic numbers/experimentation" for plugins). These plugins will have their own isolated and optimized SIMD implementations. We are however free to reference/copy-paste portions of DSP across plugins as we see fit (as long as the other plugins are also GPLv3).

Also, like what was mentioned above, we simply don't have the resources to compete with industry leaders such as FabFilter and iZotope. But the quality of the built-in effects should be at-least good enough to where most producers can acheive a satisfactory mix with only internal plugins.

That being said, any kind of effect/synth idea is welcome, it's just that we should focus on the essentials first.

Also the all of these plugins will (initially) be GUI-less. It is important that all our plugin GUIs are designed around the needs of the DSP and not the other way around.

## Developer Guidelines
The link to the kanban-style [`project-board`].

There is a gain plugin example showing the overall format I would like the plugin code to take. That being said, you are free to develop the plugin DSP in whatever language and/or plugin development framework you are most comfortable with (as long as the DSP is self-contained and doesn't rely on a 3rd party library), and I or someone else can port it into our format once you're done.

- The [`example gain dsp`] crate demonstrates how the DSP portion of a plugin more or less should be structured.
- The [`example gain plugin`] crate demonstrates how to use [`baseplug`] to create a gui-less plugin from the DSP crate above.

Plugins will go in the [`rusty-daw-plugins`] repo, and offline audio clip effects will go in the [`rusty-daw-offline-audio-fx`] repo. *Note I decided against putting ported plugins into `rusty-daw-plugin-ports` since most of our plugins will be modifications of existing plugins anyway.*

Please take careful note of what pieces of code are borrowed from ported plugins, and make apparent the appropriate credit and license of those plugins where appropriate. All of our code will be GPLv3 (although we may also consider using AGPL).

Refer to the guide in [`rusty-daw-plugins`] on how to build and install these plugins.

In order to keep the total number of plugins low, I encourage adding a "character" parameter in plugins that switches between different DSP algorithms. For example, the "character" parameter in our compressor plugin could switch between the compressor algorithm found in [`Vitalium`], the algorithm found in [`x42 darc compressor`], the algorithm found in [`pressure4`], the algorithm found in [`ZamCompX2`], and so on. This will also add future-proofing to plugins if we ever decide to add more algorithms in the future without having to introduce new plugins.

Note that the `Audio Clip Effects` are all *offline* effects, so the intended workflow to develop those effects is to just load a WAV file in a standalone terminal app using a crate like [`hound`], modify it, and then export it to another WAV file.

Prefer to use the `SVF` filter in place of all biquad filters. It is simply just a better quality filter. For reference here is an [`implementation of the SVF filter`], and here are the [`Cytomic Technical Papers`] explaining the SVF filter in technical detail.

For resampling algorithms prefer using the "optimal" filters from the [`deip`] paper.

A lot of these can be made by porting the effects in the awesome open-source Vital synth. Although I highly suggest referring to the GPLv3 fork of the plugin called [`Vitalium`] so it's clear what portions of the code we are allowed to use.

I also have a long list of existing open source plugins in my [`Awesome-Audio-DSP`] repo for reference.

## Effect DSP
Note that we should prioritize using the `SVF` filter in place of all biquad filters. It is simply just a better quality filter.
For reference here is an [`implementation of the SVF filter`], and here are the [`Cytomic Technical Papers`] explaining the SVF filter in technical detail.

These are listed roughly in order of priority with the first being the highest priority:

- [ ] A simple "utitlity" plugin for gain, panning, stereo width, and DC offset.
  - Panning should include various pan laws such as "linear", "circular", etc.
- [ ] A simple mute plugin.
  - Including this high on the list because it's very easy to do.
  - One stipulation is that there should be about 5ms of gain smoothing when it toggles on/off. This is necessary to avoid audible clicking.
- [ ] Basic single-knob LP/HP filter plugin like Xfer's DJMFilter.
  - In case you don't know how DJMFilter works, the principle is that the lower half of the knob range controls the cutoff of the LP filter and the HP filter is disabled, the upper half of the knob range controls the cutoff of the HP filter and the LP filter is disabled, and when the knob is roughly in the center both filters are disabled. This makes filter automation very easy.
  - Should use a second-order filter with an adjustable Q control.
  - An optional "drive" parameter for light distortion would be nice.
- [ ] Multiband splitter/merger
  - [ ] Ability to set an arbitrary number of bands (from 2 up to like 16 or something like that).
  - [ ] Include controls to adjust the frequency of each band split.
- [ ] Basic Soft/Hard Clipper
  - Ability to switch between soft and hard mode.
  - Controls for input gain, threshold, and output gain.
  - An additional control that adjusts the input gain and output gain at the same time.
  - I quite like the sound and quality of [`Misstortion`], so porting it could be a great starting point.
  - Ability to split the low end "cutoff" frequency where the wet signal below this cutoff is replaced with the dry signal below this cutoff. This should be easy once we have the "Multiband splitter/merger". This is useful for making distorted bass sounds and kick drums sound "cleaner".
- [ ] Parametric Equalizer
  - We will likely use the `SVF` filter designs described above (at least as a starting point).
  - Typical parametric EQ stuff such as lowpass, highpass, low-shelf, high-shelf, bell, and notch.
  - Lowpass and highpass filters should have multiple intensities (atleast including 6dB/octave, 12db/octave, 24db/octave, and 48db/octave).
  - There needs to be **no** cramping in the high end. This is pretty paramount to the expected sound of parametric EQ.
  - Must sound good when being sharply automated (a technique commonly used in electronic sound design). Using the `SVF` filter will help us a lot here.
  - I quite like the sound and quality of the [`x42 fil4 EQ`], so studying how this works could be a good starting point.
- [ ] Basic Limiter
  - A standard single-band limiter.
  - Controls for input gain, output gain, threshold, and a drop-down for look-ahead time (1ms, 3ms, 10ms, etc).
  - The [`ZamMaximX2`] and [`x42 dpl limiter`] plugins could be a good place to start/port.
  - I'm not sure what goes into making a limiter "sound good". We need to research this some more.
- [ ] Basic Compressor
  - A standard single-band compressor with attack, release, threshold and output gain.
  - In addition to the normal threshold, there should also be an "expander" threshold that boosts signals under a given threshold. Not every "character" of the DSP needs to include this though.
  - Two different detection modes: Peak and RMS. Not every "character" of the DSP needs to include this though.
  - Basic lowpass & highpass filter on sidechain input (including when the input signal is used as the "sidechain").
  - I'm not sure what goes into making a compressor "sound good". We need to research this some more.
  - [`x42 darc compressor`], [`Pressure4`], and the compressor in [`Vitalium`] could be good places to start/port.
  - At some point I would also want to have a "character" with an analogue-modeled bus compressor (i.e. Cytomic's "The Glue").
    - Honestly I'm not sure where to start with this one though, but bus compressors are pretty essential in creating a decent sounding mix, so putting research into this is definitely worth it.
- [ ] Distortion
  - A digital waveshaper with common built-in shapes.
    - The "distortion" effect in [`Vitalium`] is pretty much exactly what I'm looking for here.
  - Analogue modeled distortion "characters".
    - [`ZamTube`], [`Density`], [`ToTape6`], and [`IronOxide5`] be could be a great place to start/port.
  - Should include controls for a pre-filter and a post-filter.
  - Also add the ability to split the low end "cutoff" frequency where the wet signal below this cutoff is replaced with the dry signal below this cutoff. This should be easy once we have the "Multiband splitter/merger". This is useful for making distorted bass sounds sound "cleaner".
- [ ] Delay
  - The "delay" effect in [`Vitalium`] has most of what I'm looking for here.
  - Sync to tempo or freerun
  - Mono, stereo, & ping-pong modes
  - Lowpass & highpass filter controls on taps.
  - Controls for feedback, & mix.
  - Controls for ducking the delay based on the input signal (basically add a compressor with the dry signal as the sidechain input).
  - Possibly extra effects on the taps such as chorusing, pitch shifting, and waveshaping.
- [ ] Gate
  - A standard gating plugin with attack, release, and threshold.
  - I'm not sure what goes into making a gate "sound good". We need to research this some more.
  - The [`ZamGateX2`] plugin could be a good place to start/port.
- [ ] Mid/Side splitter/merger
- [ ] Multiband Compressor
  - The multiband compressor in [`Vitalium`] is pretty much what I'm looking for here, but we can also experiment with adding different compressor algorithms if we wish.
- [ ] Chorus
  - The chorus effect in [`Vitalium`] is pretty much what I'm looking for here.
- [ ] Phaser
  - The phaser effect in [`Vitalium`] is pretty much what I'm looking for here.
- [ ] Tremelo
  - Should have the option to sync the LFO to tempo.
- [ ] Reverb
  - Of course the field of what makes reverbs sound good is a vast and complicated (and a lot of it is hidden away as trade secrets). However, I'll give some resources that could be places to start/port:
  - I happen to quite like the sound of the built-in reverb in [`Vitalium`].
  - Three other decent open source reverbs include [`RoomReverb`], [`Mverb`] and the [`Dragonfly Reverb`].
  - I would also like to add a "charcter" option that uses a port of the awesome [`Aether`] shimmering reverb plugin.
  - The developer of the amazing Valhalla plugins has made a blog on [`developing reverbs`]. 
  - The technical paper on the [`Freeverb algorithm`].
- [ ] Convolver
- [ ] Linear phase EQ
- [ ] Dynamic EQ
  - This should be easy once we have a parametric eq and a compressor.
  - This can also double as a de-esser.
  - We could also consider adding this as an optional feature in the Parametric EQ plugin, but the controls will likely be different so it may be better as a separate plugin.
- [ ] Analogue-modeled/Mastering EQ
  - This will differ from the Parametric EQ plugin in that it will use only knobs to control the parameters.
  - A good starting point could be to port [`Luftikus`], although we should probably add more flexibility in controlling the frequency of bands.
- [ ] Comb filter effect
  - The comb filters in [`Vitalium`] is pretty much what I'm looking for here.
- [ ] Flanger effect
  - The flanger effect in [`Vitalium`] is pretty much what I'm looking for here.
- [ ] Bitcrusher effect
- [ ] Guitar amp simulations
  - [`Guitarix`] could be a great place to start/port.
- [ ] Realtime pitch-shifting effect
- [ ] Vibrato effect
- [ ] Vocoder

## Visualizer Plugins

These are listed roughly in order of priority with the first being the highest priority:

- [ ] Oscilloscope
  - Controls for input gain & time window.
  - Should accept incoming MIDI notes to automatically set the time window based on pitch.
  - Ability to freeze the signal in place.
- [ ] Spectrometer/Spectrogram
  - Should be able to switch between the "classic" style (like Voxengo Span) and the "FL-style" (like the spectrogram in Fruity Wavecandy).
  - [`Wolf Spectrum`] is a good one to reference for the "FL-style" one.
- [ ] Phase correlation meter/Goniometer.
  - [`easySSP`] is a good one to reference here.
- [ ] Loudness meter
  - Detect loudness of a signal with different methods such as RMS and EBU.
  - The [`LUFSMeter`] plugin is a great starting point here.

# Generator Plugins

These are listed roughly in order of priority with the first being the highest priority:

- [ ] Sampler
  - AHDSR gain envelope
  - Should be able to pitch samples in realtime, with an ADSR pitch envelope
  - Basic LP/BP/HP filter with an ADSR filter envelope
- [ ] Noise generator
  - White noise, pink noise, brown noise, and 4.5dB per octave noise (I heard that's better for mixing than pink and brown noise).
- [ ] Simple "3xOSC"-like synth
  - The main purpose of this is to quickly have some kind of basic synth in the project. This will be heavily inspired by FL-studio's 3xOSC plugin.
  - 3 Oscillators with selectable shapes (saw, square, pulse, triangle, noise, etc).
  - ADSR gain envelope
  - Single LP/BP/HP filter, and an ADSR envlope on that filter
  - Portamento
  - Legato mode
  - Simple waveshaper distortion, delay, and chorus effects
- [ ] Multisampler
  - Be able to play soundfont (SF2) files.
    - [`Carla`] could be a good one to reference here.
  - Be able to play SFZ files.
    - [`Sfizz`] could be a good one to reference here.
  - Should be able to pitch samples in realtime
  - AHDSR envelope
  - Keyswitches for SFZ files that support it
- [ ] Flagship classic subtractive synth (low priority)
  - No real guidelines at the moment. I'm mostly just picturing a synth along the lines of u-he's Hive or Synth1.
- [ ] Simple FM synth (low priority)
  - No real guidelines at the moment. I'm mostly just picturing a synth along the lines of [`JuceOPL`] and [`ADLplug`].
- [ ] Flagship wavetable synth (low priority)
  - No real guidelines at the moment. I'm mostly just picturing a synth along the lines of Ableton's Wavetable synth.
- [ ] Flagship additive synth (low priority)
  - No real guidelines at the moment. I'm mostly just picturing a synth along the lines of NI's Reaktor or FL's Harmor.
- [ ] Flagship granular synth (low priority)
  - No real guidelines at the moment. I'm mostly just picturing a synth along the lines of Ableton's Granulator synth.

## Offline Audio Effects
These effects are all *offline* effects, meaning they are pre-computed before being sent to the realtime thread. Actual "realtime" effects on audio samples will be done in a separate "sampler" instrument plugin.

These are listed roughly in order of priority with the first being the highest priority:

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

## MIDI/Note effects
TODO (A lot of this will depend on exactly how the internal control spec will work, so I'm leaving this blank for now.)

[`samplerate`]: https://crates.io/crates/samplerate
[`deip`]: https://github.com/BillyDM/Awesome-Audio-DSP/blob/main/deip.pdf
[`Time-warping`]: https://www.ableton.com/en/manual/audio-clips-tempo-and-warping/
[`Working with audio clips`]: https://www.bitwig.com/userguide/latest/working_with_audio_events/
[`Cytomic Technical Papers`]: https://cytomic.com/index.php?q=technical-papers
[`implementation of the SVF filter`]: https://github.com/wrl/baseplug/blob/trunk/examples/svf/svf_simper.rs
[`x42 fil4 EQ`]: https://github.com/x42/fil4.lv2/tree/master
[`x42 darc compressor`]: https://github.com/x42/darc.lv2/tree/master
[`x42 dpl limiter`]: https://github.com/x42/dpl.lv2/tree/master
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
[`rusty-daw-offline-audio-fx`]: https://github.com/RustyDAW/rusty-daw-offline-audio-fx
[`project-board`]: https://github.com/MeadowlarkDAW/project-board/projects/2
[`example gain dsp`]: https://github.com/RustyDAW/rusty-daw-plugins/tree/main/example-gain/example-gain-dsp
[`example gain plugin`]: https://github.com/RustyDAW/rusty-daw-plugins/tree/main/example-gain/example-gain-baseplug-nogui
[`hound`]: https://crates.io/crates/hound
[`baseplug`]: https://github.com/wrl/baseplug
[`Awesome-Audio-DSP`]: https://github.com/BillyDM/Awesome-Audio-DSP/blob/main/OPEN_SOURCE_PLUGINS_AND_SOFTWARE.md
[`Vitalium`]: https://github.com/DISTRHO/DISTRHO-Ports/tree/master/ports/vitalium
[`ZamCompX2`]: https://github.com/zamaudio/zam-plugins/tree/master/plugins/ZamCompX2
[`ZamMaximX2`]: https://github.com/zamaudio/zam-plugins/tree/master/plugins/ZaMaximX2
[`ZamGateX2`]: https://github.com/zamaudio/zam-plugins/tree/master/plugins/ZamGateX2
[`RoomReverb`]: https://github.com/cvde/RoomReverb
[`Wolf Spectrum`]: https://github.com/wolf-plugins/wolf-spectrum
[`easySSP`]: https://github.com/DISTRHO/DISTRHO-Ports/tree/master/ports-legacy/easySSP
[`Carla`]: https://github.com/falkTX/Carla/
[`sfizz`]: https://github.com/sfztools/sfizz
[`JuceOPL`]: http://www.linuxsynths.com/JuceOPLPatchesDemos/juceopl.html
[`ADLplug`]: http://www.linuxsynths.com/ADLplugPatchesDemos/adlplug.html
[`Guitarix`]: https://github.com/brummer10/guitarix
[`Aether`]: https://github.com/Dougal-s/Aether
