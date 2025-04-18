# Design Document for the DSP of Meadowlark's Plugin Suite
Meadowlark aims to be a FREE and open-source DAW (Digital Audio Workstation) for Linux, Mac and Windows. Its goals are to be a powerful recording, composing, editing, sound designing, mixing, and mastering tool for artists around the world, while also being intuitive and customizable.

# Objective
Our main focus for Meadowlark's suite of internal plugins will be on a suite of essential mixing and mastering FX plugins. (Contribution on synths and other exotic effects are welcome, but they are not a priority right now).

We obviously don't have the resources to compete with the DSP quality of companies like Waves, iZotope, or Fabfilter. Our aim is have "good enough" quality to where a producer can create a "pretty good" mix using Meadowlark's internal plugins alone. Essentially I want beginning producers to not feel like they have to hunt down external plugins for each part of the music production process. In addition this will make it much easier to create music production tutorials using Meadowlark's internal plugins alone.

Because we have a small team at the moment, we will focus more on porting DSP from other existing open source plugins to Rust rather than doing all of the R&D from scratch ourselves. People can still do their own R&D if they wish (and there are cases where we have to because an open source plugin doesn't exist for that case), but there already exists some great DSP in the open source world.

# Special Notes

### Licensing
Before porting any existing open source plugin DSP, make sure that the license of the plugin is compatible with the `GPLv3` license. For example, if the license says `GPLv2 or later`, then we're good, but if the license says `GPLv2` only, then we will need to contact the original author to make sure it's okay to relicense the port to `GPLv3`.

### Programming Languages
While we definitely prefer writing in Rust, if you wish to contribute DSP and you aren't comfortable with Rust, you are free to develop the DSP in C, C++, Zig, or Faust. The one condition being that the DSP does not include any heavy dependencies, including JUCE's built-in DSP classes). We can then port and/or create bindings to the DSP later.

### GUI
If you wish to contribute, please do not worry about the UI until the DSP is completed. DSP should always be the main focus when developing a plugin. The UI comes second. Plugin frameworks like [nih-plug], [DPF], and `JUCE` make it easy to create gui-less plugins that can be loaded into any DAW or host for testing.

### Biquad vs SVF
We should always consider replacing any biquad filters with the [SVF](https://cytomic.com/files/dsp/SvfLinearTrapOptimised2.pdf) filter model since it is better than the biquad model in practically every way. An implemention of this model can be found [here](https://github.com/MeadowlarkDAW/meadow-dsp/blob/main/meadow-dsp-mit/src/filter/svf/f32.rs).

# Non-Goals

### Complex Synthesizers
We will not be focusing on creating "feature rich" synthesizer plugins. In my opinion, open source synthesizers like [Vital]/[Vitalium] and [Surge XT] already do this well. Users can just install those.

That being said, I do think there is room to have some simple synthesizer plugins built-in. I explain this in more detail below.

### Modularity
While we are definitely taking a lot of inspiration from Bitwig Studio, we won't be aiming for the same "the DAW is a modular instrument" concept of Bitwig (at least not to the degree that Bitwig takes it). Here is my reasoning:

* Bitwig's modulators are a very complex concept to newcomers and non-sound-designers, and I feel having this as a core focus/feature in Meadowlark will only serve to deter those users away.
* Bitwig's modulator system adds a whole slew of complexity and clutter to the UI.
* IMO the sound quality, workflow, and performance is much better when using dedicated modular synths anyway. The developers of modular synths spend time making sure their specific modulators, oscillators, filters, and effects all sound and fit nicely together.
* Complex modular synth setups can be very cumbersome to use when the UI is constrained to an inline horizontal FX rack.
* Dedicated synths are far more likely to have thriving community of preset makers for them.
* I personally rarely find myself using Bitwig's modulators or synths anyway. I just use the built-in modulators that come with the Vital, SurgeXT, and Zebra2 synths.

Instead we will simply have macros, LFOs, envelopes, and a modulation matrix built into Meadowlark's "chain" plugin.

# The Plugin Suite
> Note, this list of plugins is not set in stone. We may decide to add or remove any of these from the list in the future.

# Containers
"Containers" are modules inside the horizontal FX rack that contain other plugins. They are analogous to containers in Bitwig Studio.

## Macro
> priority: High | difficulty: MVP is easy, full features will be somewhat hard

This is the main container type. It is analogous to the "chain" container in Ableton and Bitwig. In addition, it is the method in which macros will work in Meadowlark.

For MVP, this is all that the chain plugin will do. In later releases, the user can expand this container to reveal a view where they can assign macros to any parameter in any plugin contained within the macro container. Here, the user can also add "macro effects" like LFOs and envelopes.

Doing it this way makes it easier to save the whole container, complete with macros, as a "preset" in the preset browser.

## Multiband
> priority: High | difficulty: Somewhat easy

This plugin will behave similarly to the multiband container device in Bitwig Studio with two additions:

* The user can dynamically select between 2 to 5 bands, unlike Bitwig which supports only 2 or 3 bands.
* An additional drop-down control is used to select the algorithm using for band splitting.

#### References

The DSP for this has already been done for us in Rust: https://github.com/robbert-vdh/nih-plug/tree/master/plugins/crossover. It's just a matter of implementing it as a container plugin inside of Meadowlark.

## Mid/Side
> priority: High | difficulty: Very easy

A simple container plugin that splits a stereo signal into its mid/side components. The DSP for this should be pretty straightforward.

## L/R
> priority: High | difficulty: Very easy

A simple container plugin that splits a stereo signal into separate left and right mono signals. The DSP for this should be very straightforward.

## Layer
> priority: High | difficulty: Very easy

A container plugin that lets you layer multiple plugins in parallel.

## Drum Machine
> priority: Medium | difficulty: Somewhat hard

A container plugin that behaves like the Drum Machine container plugin in Bitwig.

# Audio FX

## Utility
>  priority: High | difficulty: Easy

This is a simple plugin with boring but essential utilities. It will behave mostly like the Utility plugin from Ableton Live, and have the following features:

* Toggle buttons to invert the phase of the left and/or right channel
* A drop-down to select the "channel mode" (Stereo, Swap L/R, Mono, L only, R only)
* Stereo width control
* Gain control
* Pan control
* DC blocker toggle button

Unlike the Ableton plugin this plugin won't have a "bass mono" toggle button and frequency control. Instead this can easily be done using the multi-band splitter plugin.

#### References

* https://github.com/m1m0zzz/utility-clone
* https://github.com/butchwarns/utilities

## Time Offset
> priority: High | difficulty: Easy

A plugin that simply delays the signal by a given amount (either by samples or by ms). The DSP for this should be pretty straightforward. Sub-sample interpolation is not necessary for what this plugin aims to do.

## Parametric EQ
> priority: Very High | difficulty: Hard

> The DSP for this plugin is already being worked on at the time of this writing.

The parametric equalizer is the single most important tool when it comes to mixing, so it is important that we get this right.

This plugin should have the essentials:

* A lowshelf/lowpass band
  * The lowpass band should include various slopes including 6dB/oct, 12dB/oct, 24dB/oct, 36dB/oct, and 48dB/oct.
* A high-shelf/highpass band
  * The highpass band should include various slopes including 6dB/oct, 12dB/oct, 24dB/oct, 36dB/oct, and 48dB/oct.
* A number of bell/notch filter bands
* A spectrometer that can show either pre eq, post eq, or be off
  * Note that I personally prefer to have the "FL-Studio" style of spectrometer of using color to represent frequency intensity instead of the traditional "line graph" approach. I feel the latter gives a false impression of how humans actually hear sound, and it can lead to bad habits of "mixing with your eyes". I really like FL's approach here because the "wash of color" more accurately represents how our brains actually perceive frequencies as a "wash of frequencies".

In addition I would prefer this eq to have two toggles:

* A toggle that turns on "adpative Q" mode. When enabled, it automatically narrows the Q factor as the gain of a band increases. This has the effect of sounding more "musical" to some people.
* If we go with a DSP algorithm that introduces latency, then we should add a "high quality" toggle (should be enabled by default) that switches between using latency for better quality, or using zero latency at the cost of worse quality.

Two more notes on quality:

* Preferably, frequencies should not "cramp" near the high end. For more info check out [this article](https://www.pro-tools-expert.com/production-expert-1/what-is-eq-cramping-and-should-you-care).
* The filters must behave well when being automated (EQs are commonly used as filter effects in electronic music production). If we use SVF models, this should already be taken care for us.

#### Non-goals
This plugin will have no mid/side mode. This is because the user can easily construct a mid/side EQ by placing two EQs into the mid/side splitter plugin.

This plugin will have no "dynamic EQ" mode, since this would introduce a lot of complexity to our already cramped UI. The dynamic EQ will be its own dedicated plugin with its own dedicated DSP.

## Dynamic EQ
> priority: Medium | difficulty: Hard

This should be a simple but effective dynamic equalizer plugin. It should have the following features:

* Highpass, lowpass, high shelf, low shelf, and bell filter types
* Standard compressor controls: gain/attack/release
* Sidechain input
* A simple lowpass/bandpass filter applied to the sidechain input
* A toggle for peak/rms modes

Note, this will also double as our "de-esser" plugin. De-essing can simply be a preset for this plugin.

The trickiest part of this plugin would probably be the optimizations, since dynamically calculating parametric filter coefficients is prohibitively expensive. I imagine the best solution might be to dynamically generate a look-up table based on the "modulation range" of the automated band. Either that or a compile-time LUT for 44100 and 48000 Hz.

## Linear Phase EQ
> priority: Medium | difficulty: Hard

This should be a high quality linear phase parameter equalizer, useful for mastering. I don't know how linear phase equalizers work, so I could use some help on this one.

## A "General Purpose" Compressor
> priority: High | difficulty: Hard

Compressors and limiters come in a wide variety of forms and functions. As such, there should be multiple compressor/limiter plugins in Meadowlark, not just one.

That being said, I do think we should focus on having a good "general purpose" compressor. At the bare minimum it should have the following features:

* The standard attack/release/ratio/input gain/makeup gain controls
* A "knee" control
* A toggle to use "peak" or "rms" detection (or some blend of those)
* Sidechain support, along with a simple highpass and lowpass filter applied to the sidechain signal

Some research needs to be done here on the best DSP to use for this. There are two plugins that stand out, [Squeezer](https://github.com/mzuther/Squeezer) and [ZLEComp](https://github.com/ZL-Audio/ZLEComp). To me, ZLEComp sounds a bit "cleaner", while Squeezer sounds a bit more "musical". Still, to me they aren't stellar, just passable. Though I would like to get more opinions on this.

The compressor algorithm in [Vital]/[Vitalium] is interesting as well, as it allows upwards compression in addition to downwards compression, although it is missing some of the other required features. It could still be a good starting point for some custom DSP.

The DSP in [faustCompressors](https://github.com/magnetophon/faustCompressors) also looks really promising.

Right now I'm leaning towards using Squeezer as the `1.0` version of the compressor, and maybe later having a `2.0` version that uses custom DSP.

## Compressor (Pressure5)
> priority: High | difficulty: Easy

This will be a port/bindings to Airwindow's [Pressure5](https://github.com/airwindows/airwindows/tree/master/plugins/LinuxVST/src/Pressure5) plugin. It is a popular compressor with a few but unique controls.

## Compressor (Molot Lite)
> priority: Low | difficulty: Medium

This will be a port/bindings to the DSP of the open source plugin [Molot Lite](https://github.com/magnetophon/molot-lite). It is an "aggressive" sounding compressor with a lot of color and character.

## OTT-Like Compressor
> priority: Medium | difficulty: Medium

This will be a port/bindings to the multiband compressor module from the [Vital]/[Vitalium] synthesizer.

## Limiter (ZaMaximX2)
> priority: High | difficulty: Medium

This will be a port/bindings to the DSP of the open source plugin [ZaMaximX2](https://github.com/zamaudio/zam-plugins). It is a decent sounding low-latency brickwall limiter.

## Look-ahead Limiter (lamb)
> priority: Low | difficulty: Somewhat Easy

This will use the DSP from the open source plugin [lamb](https://github.com/magnetophon/lamb-rs). A high quality look-ahead limiter for mixing and mastering.

## Safety Limiter
> priority: High | difficulty: Easy

A simple plugin that plays a warning tone when the signal goes above a certain threshold. Useful for development purposes.

This will use the DSP from the nih-plug plugin [Safety Limiter](https://github.com/robbert-vdh/nih-plug/tree/master/plugins/safety_limiter).

## Spectral Compressor
> priority: Low | difficulty: Somewhat Easy

This will use the DSP from the awesome nih-plug plugin [Spectral Compressor](https://github.com/robbert-vdh/nih-plug/tree/master/plugins/spectral_compressor). It has a lot of uses for sound design and mixing. It can also double as a "de-esser" plugin with a preset.

## Distortion (Waveshaper)
> priority: Medium | difficulty: Medium

This plugin will consist of the following:

* Input & output gain controls
* Wet/dry mix control
* A drop-down list of waveshaper algorithms (nonlinear functions) to choose from
* A drop-down list for the amount of oversampling to use (None, 2x, 4x, or 8x).

There are a plethora of nonlinear functions to choose from in the open source world. Most notably are the [distortion module from Vital](https://github.com/DISTRHO/DISTRHO-Ports/blob/5c55f9445ee6ff75d53c7f8601fc341d200aa4a0/ports-juce6.0/vitalium/source/synthesis/effects/distortion.cpp) and the the [waveshapers in SurgeXT](https://github.com/surge-synthesizer/sst-waveshapers).

## Distortion (ToTape6)
> priority: High | difficulty: Easy

This will be a port/bindings to the DSP of Airwindow's [ToTape6](https://github.com/airwindows/airwindows/tree/master/plugins/LinuxVST/src/ToTape6) plugin. A favorite distortion plugin of mine.

## Distortion (Density)
> priority: High | difficulty: Easy

This will be a port/bindings to Airwindow's [Density](https://github.com/airwindows/airwindows/tree/master/plugins/LinuxVST/src/ToTape6) plugin. Another favorite distortion plugin of mine.

## Distortion (Soft Vacuum)
> priority: High | difficulty: Easy

This will use the DSP from the nih-plug plugin [Soft Vacuum](https://github.com/robbert-vdh/nih-plug/tree/master/plugins/soft_vacuum), a nice sounding distortion plugin inspired by the `Hard Vacuum` plugin by Airwindows.

## Distortion (Bitcrusher)
> priority: Medium | difficulty: Medium

This will be a port/bindings to the bitcrushing algorithm found inside the distortion module from the [Vital]/[Vitalium] synthesizer.

## Guitar Amp
> priority: Low | difficulty: Hard 

This is something I want, but more research needs to be done. There are several new "neural-network" based guitar amp and cabinet simulation plugins out that seem promising.

## Spline-based Waveshaper (Wolf Shaper)
> priority: Low | difficulty: Medium

This will be a port/bindings to the DSP of the open source plugin [Wolf Shaper](https://github.com/wolf-plugins/wolf-shaper). It's a fantastic spline-based waveshaper.

## Mastering Clipper (ClipOnly2)
> priority: High | difficulty: Easy

This will be a port/bindings to Airwindow's [ClipOnly2](https://github.com/airwindows/airwindows/tree/master/plugins/LinuxVST/src/ClipOnly2) plugin. This is a subtle clipping effect that works great for mastering.

## Performance Filter
> priority: Medium | difficulty: Somewhat easy

This is a simple lowpass/highpass filter plugin that will function exactly like the [DJM](https://splice.com/plugins/4442-djmfilter-vst-au-by-xfer-records) plugin by Xfer Records.

We should select a filter model that sounds both "musical" and which behaves well when being modulated.

## Chorus
> priority: Medium | difficulty: Medium

This will be a port/bindings to the chorus module from the [Vital]/[Vitalium] synthesizer.

## Chorus (YK)
> priority: Low | difficulty: Medium

 I'm a fan of the [YK Chorus](https://github.com/SpotlightKid/ykchorus) plugin that emulates the chorus module from a well-known vintage analog synthesizer.

However, this plugin is licensed under GPL-2.0, so I'm not sure if there's a way we can include it in Meadowlark. I'll see what I an do, but it's not a big deal if we can't include it.

## Phaser
> priority: Medium | difficulty: Medium

This will be a port/bindings to the phaser module from the [Vital]/[Vitalium] synthesizer.

## Flanger
> priority: Medium | difficulty: Medium

This will be a port/bindings to the flanger module from the [Vital]/[Vitalium] synthesizer.

## Delay
> priority: Medium | difficulty: Medium

This will be a port/bindings to the delay module from the [Vital]/[Vitalium] synthesizer.

## Filter
> priority: Medium | difficulty: Medium

This will be a port/bindings to the filter module from the [Vital]/[Vitalium] synthesizer.

## Reverb (Vitalium)
> priority: High | difficulty: Already mostly done

This will be a port/bindings to the reverb module from the [Vital]/[Vitalium] synthesizer.

I have actually already done this in my plugin [Vitalium Verb](https://github.com/BillyDM/vitalium-verb). Though I do want to make a few more tweaks such as balancing the volume of the "size" parameter to be more consistent.

> Note on the reverb plugins: Instead of having multiple separate reverb plugins which could be overwhelming to new users, it might be better to consolidate all or most of the reverb plugins into a single plugin, and have the different algorithms be different "characters" that the user selects in a drop-down menu. However, we would need to make careful considerations on how to cleanly handle the different list of parameters of each reverb algorithm.

## Reverb (Airwindows)
> priority: High | difficulty: Easy

This will be a port/bindings to Airwindow's [Reverb](https://github.com/airwindows/airwindows/tree/master/plugins/LinuxVST/src/Reverb) plugin. It doesn't have much in the way of controls, but it sounds good.

## Reverb (Surge XT)
> priority: Medium | difficulty: Somewhat easy

This will be a port/bindings to the [reverb modules from the Surge XT synthesizer](https://github.com/surge-synthesizer/sst-effects). They are decent sounding reverbs.

## Reverb (Dragonfly)
> priority: Low | difficulty: Somewhat hard

This will be a port/bindings to the popular [Dragonfly](https://github.com/michaelwillis/dragonfly-reverb) reverb plugins.

## Shimmering Reverb (Cloudseed)
> priority: Low | difficulty: Somewhat Hard

This will be a port/bindings to the awesome shimmering reverb plugin [Cloudseed](https://github.com/ValdemarOrn/CloudSeed). There are also derivatives of Cloudseed we could reference such as [CloudReverb](https://github.com/xunil-cloud/CloudReverb) and [Aether](https://github.com/Dougal-s/Aether).

## Convolution
> priority: Low | difficulty: Hard

This will be a convolution plugin where the user can import any impulse response in WAV format. This can be used to create reverbs and other effects.

There are several options for convolution, some in native Rust. [This library](https://github.com/holoplot/fft-convolution) looks promising.
## Airwindows Consolidated

> priority: Low | difficulty: Easy but tedious

While I have selected a few key plugins I want from the Airwindows project to be their own dedicated internal plugin in Meadowlark, Airwindows has a *ton* of other plugins. While I would like to have more of them since the [Airwindows Consolidated](https://github.com/baconpaul/airwin2rack) project has already done the work creating a static library for us, I want to avoid polluting the plugin list with too many individual plugins.

So instead, I think it would be best to have a plugin that behaves like the `Airwindows Consolidated` plugin.

# Generators (Instruments)

## Noise Generator
> priority: High | difficulty: Easy

A simple plugin that can generate white, pink, and brown noise.

## Test Tone
> priority: High | difficulty: Easy

A simple plugin that generates a sine wave.

## Sampler (One-Shot)
> priority: High | difficulty: Hard

A plugin that plays a sample when triggered by MIDI. It will function similarly to the sampler plugin in Bitwig with adjustable start/end times, looping, ADSR controls (with adjustable curves), a pitch control, and an envelope for the pitch control.

## Multisampler (SFZ / DecentSampler player)
> priority: Medium | difficulty: Very Hard

This is a plugin that will play SFZ and DecentSampler files. We could also consider adding soundfont support.

## 3xOSC Synth
> priority: Medium | difficulty: Hard

A simple synthesizer like the 3xOSC synth from LMMS. Note we can't create bindings to it directly since it is licensed under GPL2.

## Chiptune Synth
> priority: Low | difficulty: Hard

It would be nice to have a simple chiptune-style synthesizer, especially since producing game soundtracks is one of the target applications.

# Visualizers

## Waveform
> priority: Low | difficulty: Medium

A plugin which displays a scrolling waveform.

## Oscilloscope
> priority: Medium | difficulty: Medium

An oscilloscope plugin.

## Spectrometer
> priority: Medium | difficulty: Somewhat Hard

A spectrometer plugin. This plugin should have two modes: a standard "line graph" mode, and a "spectrogram" mode like what is found in FL Studio's "Wave Candy" plugin.

Also see (TODO) on how to properly handle low frequencies.

## Goniometer
> priority: Medium | difficulty: Medium

A plugin for visualizing the stereo phase correlation of a signal.

## Loudness Meter
> priority: Low | difficulty: Medium

A plugin that can measure the loudness of a signal in RMS or LUFS.

## Pitch Detector
> priority: Low | difficulty: Medium

A plugin that simply detects the pitch of a tuned audio signal. More research needs to be done here, but there are plenty of open source plugins that do this. 

# The Plugin Suite - MIDI FX

## Note Transpose
> priority: High | difficulty: Easy

A plugin that simply transposes notes up or down.

## Note Scale
> priority: High | difficulty: Easy

A plugin that snaps notes to the nearest note in the scale.

## Note Repeater
> priority: High | difficulty: Easy

A plugin that plays multiple notes when a single note is played. Especially useful when paired with the "Note Scale" plugin.

## Arpeggiator
> priority: Medium | difficulty: Somewhat Hard

An arpeggiator effect. I currently don't have plans on how it should work exactly, but Bitwig's arpeggiator should be a good reference.

## Note Randomizer
> priority: Low | difficulty: Medium

A plugin that randomly generates MIDI notes. The user can independently adjust the ranges and weighting curves for the pitch, velocity, and note length. The seed is locked to the transport, and the user can cycle between randomly generated seeds and enter seeds manually.

This effect will be especially interesting when paired with the other note effects. :)

# Routing

## Audio Send/Receive
> priority: High | difficulty: Easy

An "audio receive" plugin can receive audio from any "audio send" plugin as long as it doesn't create a cycle in the audio graph.

## Note Send/Receive
> priority: High | difficulty: Easy

A "note receive" plugin can receive audio from any "note send" plugin as long as it doesn't create a cycle in the audio graph.

## Feedback Send/Receive
> priority: Low | difficulty: Somewhat easy

Similar to the Audio Send/Receive plugins, except that cycles in the audio graph are allowed. This is accomplished using a internal buffer shared by the two plugins.

NOTE: Be sure to include a "panic" button to stop any runaway feedback!

# Bundled Synths?
It might be worth looking into bundling complex open source synthesizers like [Surge XT] and [Vitalium] with Meadowlark (as in having it as an option when installing Meadowlark on Windows and Mac).

However, I'm not decided on this yet. For one, it would increase the complexity of packaging. And two, while it is technically in our right to redistribute plugin binaries like this because of the GPL-3.0 license, I would still like to get explicit permission from the authors first, especially with Vitalium. I don't want to undermine the Vital author's business model by providing an easy-to access free binary complete with our own set of factory presets and wavetables.

Alternatively we could just point users to the Surge and/or Vital websites (which their permission of course).

# "Pipe Dream" Plugins
These are a list of plugins I really wish existed in the open source world, but which I do not currently have the DSP expertise to do.

## Buss Compressor
I really wish there was an open source plugin that models famous analog buss compressors, like [The Glue](https://cytomic.com/product/glue/) by Cytomic or [Presswerk](https://u-he.com/products/presswerk/) by u-he. I love the sound of these compressors.

## The Kotelnikov and SlickEQ plugins by Tokyo Dawn Labs
If there were open source plugins that sounded similar to these, my dreams of having a complete open source ecosystem of high quality mixing and mastering tools would be complete. :)

It would be awesome if Tokyo Dawn Labs released an open source "Lite" version of these plugins like they have with Molot, but that probably won't happen.

[nih-plug]: https://github.com/robbert-vdh/nih-plug
[DPF]: https://github.com/DISTRHO/DPF
[Vital]: https://github.com/mtytel/vital
[Vitalium]: https://github.com/DISTRHO/DISTRHO-Ports/tree/5c55f9445ee6ff75d53c7f8601fc341d200aa4a0/ports-juce6.0/vitalium
[Surge XT]: https://surge-synthesizer.github.io/