# Meadowlark Design Document

Meadowlark aims to be a FREE and open-source DAW (Digital Audio Workstation) for Linux, Mac and Windows. Its goals are to be a powerful recording, composing, editing, sound designing, mixing, and mastering tool for artists around the world, while also being intuitive and customizable.

# Objective

*TL;DR: we want a solid, stable, free & open-source, and flexible digital audio workstation (DAW).*

A DAW is a unique project: it's a large and complex one, but if successful, it can drive change for many different technologies and define new standards. It's important for this work to be done **in the open** to avoid the mistakes of other technologies/standards, and to accelerate the pace of innovation.

Why create a new DAW from scratch? Why not contribute to an open-source DAW that already exists?

* We want a more powerful and flexible audio graph engine for advanced routing capabilities. This would require quite an overhaul to do on existing open-source DAWs.
* Most existing open-source DAWs use older and less-flexible UI libraries. We have a vision for a modern, clean, and intuitive UI with a novel workflow for Meadowlark. We also want our UI to be fully user themeable where users can freely share UI themes they have created.
* We want a DAW that is truly *FREE* and open source. No "you have to pay for a pre-compiled binary" models.
* We want to embrace the new open-source and developer-friendly [`CLAP`] audio plugin standard with first-class support.
* We want an integrated ecosystem of good quality stock plugins, with a special focus on mixing plugins.
* We prefer writing code in [`Rust`]. While C++ is fine for the task of writing audio software, it makes it trickier to create a stable and maintainable large codebase (cross-platform support, ease of writing modular code, etc). Stability is a high priority for us.
    * Rust has no garbage collection, so it is still (more easily) audio-safe.
    * Rust is cross-platform by default: works on Windows, Mac OS, and Linux, across multiple different CPU architectures, without much compilation fuss.
    * The modules and crates system makes it easy to split your code into distinct modular components, and `cargo` handles all of the compilation for you.
    * Rust's safety guarantees can significantly reduce the occurrence of crashes and reduces the time needed for debugging.
* We want to help build a new independent and open source audio development ecosystem from the ground up, so no dependencies on dominating libraries like JUCE.

# Goals/Non-Goals

## Goals
> Because this is a large and ambitious project, please keep in mind that goals marked with `(Not MVP)` are not considered goals for the first MVP (minimum viable product) release. Once MVP is done we will have a more detailed roadmap for the development cycle.

* A highly flexible and robust audio graph engine that lets you route anything anywhere, while also automatically adding delay compensation where needed.
* First-class support for the open source [`CLAP`] audio plugin standard.
   * `(Not MVP)` Additional support for LV2 plugins via an LV2 to CLAP bridge.
   * `(Not MVP)` Additional support for VST3 plugins via a VST3 to CLAP bridge. (Although if and when CLAP becomes widely adopted enough, we may decide to drop support for VST3 altogether because of all its issues with licensing and complexity.)
   * `(Not MVP)` Additional support for VST2 plugins via a VST2 to CLAP bridge. (Well, maybe. I'm unsure about the licensing issues here too.)
* `(Not MVP)` The entire audio engine including plugin hosting will run in an isolated process, serving as crash protection from buggy plugins.
* An easy-to-use settings panel for connecting to system audio and MIDI devices.
   * `(Not MVP)` Support for MIDI2 & OSC devices
* A "hybrid" approach to arranging clips on the timeline, combining ideas from the "free-flow" style of arranging in FL Studio with the more traditional track-based approach found in DAWs such as Ableton Live and Bitwig. See the full [`UX design doc`](https://github.com/MeadowlarkDAW/Meadowlark/blob/main/UX_DESIGN_DOC.md) for more details on how this hybrid system works.
   * Audio clips with quick controls for crossfades
      * Audio clips can be reversed in-place
      * Non-destructive doppler stretching (pitch shifting by speeding up or slowing down the audio clip) can be applied to audio clips in-place
         * `(Not MVP)` Support for additional high quality pitch shifting and time stretching effects that can be applied to audio clips in-place
      * `(Not MVP)` Support for really long audio clips using disk streaming. It should be possible to use Meadowlark to edit recordings that are multiple hours or even multiple days long.
   * Piano roll clips
   * Automation clips
   * Any clip can be sliced using the slice tool
   * Merge mutliple clips together into a single clip
   * Standard looping controls on the timeline with adjustable loop points
   * `(Not MVP)` Ability to switch the timeline to the more traditional track-based approach for users who prefer that workflow
   * `(Not MVP)` Ability to automate the tempo of the project.
   * `(Not MVP)` Ability to create multiple regions in the timeline with different time signatures.
* Record audio and MIDI into clips on the timeline
   * Built-in metronome
   * Recording is done in a separate thread to ensure that nothing in the raw recording is lost even if the engine overruns.
   * Multiple sources can be recorded at once into multiple clips
   * `(Not MVP)` Versitile recording features such as non-destructive loop recording and comping
   * `(Not MVP)` Easily bounce tracks & piano roll clips to an audio clip
* `(Not MVP)` Global "groove" (swing tempo) controls (like in Bitwig)
   * `(Not MVP)` Ability to assign different groove settings to different regions on the timeline
* A versitile piano roll for editing piano roll clips.
   * Per-note modulation editing
   * Quick-quantizaton features
   * Ability to highlight which notes are in your chosen scale
   * `(Not MVP)` Advanced quantization features including humanization
   * `(Not MVP)` MPE editing
   * `(Not MVP)` Ability to edit multiple piano roll clips at once (like in Reaper).
   * `(Not MVP)` Support for non-western scales (including microtonal scales).
   * `(Not MVP)` Composition-aiding tools such as chord-suggestion tools
   * `(Not MVP)` Easily create various rythms and strum patterns from presets (like in FL Studio).
   * `(Not MVP)` A "tracker mode" that transforms the piano roll into a tracker-like interface.
* Easily group mixer tracks into folders where the "parent" tracks are mix busses (like in Bitwig).
* Easily create and route to mixer send tracks.
* A versitile horizontal audio plugin FX rack with inline plugin UIs (like Ableton Live/Bitwig Studio/Renoise)
   * A versitile "Chain" plugin with assignable macros, LFOs, and MIDI/Audio triggered envelopes. A "chain" can also be saved as a preset.
   * "Send" and "Recieve" plugins that can send and recieve audio and note data to/from anywhere in the project (as long as it doesn't create a cycle in the audio graph).
      * `(Not MVP)` "Feedback Send" and "Feedback Recieve" plugins for creating feedback cycles in the audio graph.
   * `(Not MVP)` A custom open source CLAP extension that allows third party plugins to have custom inline UIs in Meadowlark's horizontal FX rack
* `(Not MVP)` A full suite of internal plugins. This internal plugin suite is mostly focused on creating a complete set of high quality FX plugins for mixing/mastering, but some synths and more exotic effects are planned as well. Refer to the [`plugin suite design doc`](https://github.com/MeadowlarkDAW/meadowlark-plugins/blob/main/DESIGN_DOC.md) for more details. But in short I'll list the plugins that are currently planned in this suite:
   * Container plugins
      * "Chain" plugin (includes assignable macros, LFOs, and MIDI/Audio triggered envelopes)
      * Layer plugin
      * `(Not MVP)` Multiband, Mid-Side, & L/R splitter container plugins (like in Bitwig)
      * `(Not MVP)` An "A/B" container plugin that lets you easily A/B different FX chains
      * `(Not MVP)` "Drum machine" container plugin (like the Drum Machine plugin in Bitwig)
   * Audio FX
      * Gain/pan/stereo width utility plugin
      * Time shift plugin (delays the signal by a given amount of time). This will even support negative time shifts thanks to our flexible audio graph engine!
      * Audio Send/Recieve plugins (like in Bitwig)
         * `(Not MVP)` "Feedback Send" and "Feedback Recieve" plugins for creating feedback cycles in the audio graph.
      * `(Not MVP)` Parametric EQ
      * `(Not MVP)` Basic single-band compressor
      * `(Not MVP)` Multiband dynamics processor (like the Multiband Dynamics plugin in Ableton Live)
      * `(Not MVP)` Limiter
      * `(Not MVP)` Gate
      * `(Not MVP)` Analogue-modeled bus compressor (like Cytomic's The Glue)
      * `(Not MVP)` Distortion/saturation plugin (includes waveshaping, bitcrushing, various tube amps)
      * `(Not MVP)` Delay plugin
      * `(Not MVP)` Reverb plugin (containing various algorithms the user can switch between)
      * `(Not MVP)` DJM-Filter-like performance filter plugin (like Xfer's DJM-Filter)
      * `(Not MVP)` Chorus
      * `(Not MVP)` Flanger
      * `(Not MVP)` Phaser
      * `(Not MVP)` Filter with many different modes (including comb filters)
      * `(Not MVP)` Tremolo/Auto-Pan plugin
      * `(Not MVP)` Convolver (for convolution reverbs and other effects that use an impulse response as input)
      * `(Not MVP)` Dynamic EQ (will also double as a de-esser)
      * `(Not MVP)` Analog-modeled EQ
      * `(Not MVP)` Vibrato effect plugin
      * `(Not MVP)` Vocal Compressor (an analogue model of something like the famous LA2A compressor)
      * `(Not MVP)` Guitar Amp/Cabinet plugin
      * `(Not MVP)` Linear phase EQ
      * `(Not MVP)` Vocoder
   * Visualizers
      * `(Not MVP)` Oscilloscope
      * `(Not MVP)` Spectrometer/Spectrogram
      * `(Not MVP)` Goniometer/Phase correlation meter
      * `(Not MVP)` Loudness meter (with support for various loudness standards such as RMS and EBU)
   * Synth/Generators
      * Noise generator
      * Tone generator
      * Single-shot sampler
      * `(Not MVP)` Multisampler (support for both SF2 and SFZ file formats)
      * `(Not MVP)` Simple "3xOSC"-like synth
      * `(Not MVP)` Simple drum synth
   * MIDI/Note FX plugins
      * Note Send/Recieve plugins (like in Bitwig)
      * Note transpose
      * `(Not MVP)` Arpeggiator
      * `(Not MVP)` Note Expression Mapper
      * `(Not MVP)` Note Transpose Map
      * `(Not MVP)` Auto-Chord plugin
* A standard Mixer view
   * Each mixer track will contain the standard controls you would expect: a fader, db meter, pan knob, solo button, mute button, and an "invert phase" button.
* `(Not MVP)` Support for mixer tracks with more than two audio channels.
* `(Not MVP)` A dedicated audio clip editor view
   * `(Not MVP)` High quality, non-destructive pitch & level automation
   * `(Not MVP)` Apply destructive effects to audio clips such as convolution and "XTreme stretch" (although these won't be truly "destructive" since they will be rendered into a new audio file and can always be reverted back to the original audio file).
* `(Not MVP)` A dedicated automation clip editor view
* `(Not MVP)` Clip launcher view
* Integrated browser for browsing samples, presets, & plugins
   * Search bar
   * Play back audio clips as they are selected in the browser
   * `(Not MVP)` An included factory library of samples, multisamples (using the SFZ format), and presets
      * `(Not MVP)` The ability to install/uninstall packages from the factory library
   * `(Not MVP)` Assign tags and favorites to items in the browser, and filter based on those tags
* Properties panel that shows advanced settings for whatever is currently selected (like Bitwig's properties panel)
* `(Not MVP)` A "Node" view that lets you visualize the entire audio graph of your project as an actual graph of nodes connected with lines.
* `(Not MVP)` Panels/views can be made into floating windows to make better use of multi-monitor setups.
* `(Not MVP)` Support for having multiple projects open at once (like in Bitwig)
* Export the entire project or just a selected region of your project to a WAV file
   * `(Not MVP)` Ability to export the project as stems
   * `(Not MVP)` Export to additional formats such as FLAC, Opus, Vorbis, etc.
* `(Not MVP)` Localization for various languages
* `(Not MVP)` A "command palette" system similar to the command pallete in VSCode.
* `(Not MVP)` A scripting API to automate/extend the functionality of Meadowlark.
* `(Not MVP)` An open-source controller scripting API that allows hardware MIDI controllers to better
integrate with Meadowlark (similiar to Bitwig's controller scripting API)
* `(Not MVP)` Ability for users to easily create and distribute themes for Meadowlark's UI. Every color in the UI can be tweaked (including support for custom gradients). We may also consider allowing themes to draw custom knobs and sliders.
   * `(Not MVP)` A few included stock themes such as "Default Dark", "Default Light", "High Contrast", and various "Colorblind" themes.
* An official website (url is https://meadowlark.app)
   * `(Not MVP)` An official "community" page where the community can share custom themes and presets (although we'll have to see how feasible moderation will be)

## Non-Goals
* While Meadowlark definitely draws a lot of inspiration from Bitwig, we are not aiming for the same "the DAW is a modular instrument" approach that Bitwig takes. We won't have "modulators" like in Bitwig, rather we will simply just have assignable macros, LFOs, and MIDI/Audio triggered envelopes built into our "Chain" plugin. Here are my reasons for this:
   * Bitwig's modulator system adds a lot of complexity and confusion to the UI. This is especially true when the UI is limited to the height of the horizontal FX rack.
   * In the end the quality of using generic modulators is questionable. I believe modular synthesis is best left to dedicated modular synth plugins, as the authors of those plugins are able to fine-tune their modulators to fit well with their system.
* The faders & pan knobs on the mixer will not be automatable. Rather we will encourage users to insert the "Utility" plugin and automate that instead.
* The suite of internal plugins will be focused mainly on audio FX for mixing/mastering. Internal synth plugins are lower priority (and not even that necessary since high quality open source synths like Vital and SurgeXT already exist). We could even consider just packaging synths like SurgeXT with Meadowlark itself. (The developer of Vital probably wouldn't be cool with us packaging Vitalium with Meadowlark, so we probably won't do that).
* LV2, VST2, and VST3 plugins will not recieve the same level of support as CLAP plugins. Those other formats will be supported through an intermediate LV2/VST2/VST3 to CLAP bridge, so functionality will be limited to whatever those bridges can support.
   * Also if and when CLAP becomes widely adopted enough, we may decide to drop support for VST3 altogether because of all its issues with licensing and complexity.
   * I am also unsure about the licensing issues around hosting VST2 plugins.
* We will not support the AUv2, AUv3, LADSPA, WAP (web audio plugin), or VCV Rack plugin formats.
* Non-destructive pitch shifting & time-stretching effects will not be supported for long audio clips that are streamed from disk. Users must use destructive editing in that case.
* Aside from a few plugins such as the Parametric EQ, Limiter, Bus Compressor, and the Vocal Compressor, we will not be doing much in-house DSP research. Rather the plan is to port DSP from existing open source plugins for the majority of our internal plugins (a lot of it will be ported from the Vital synth).

## A Special "Maybe" Goal

There is feature suggestion being thrown around a lot I feel needs special attention, and that is "realtime online collaboration".

While this is something I definitely want, and it would be a revolutionary new way for artists and bands to collaborate, I have some serious doubts how feasible it would actually be to implement this in practice. Some of my concerns include:

* Creating custom networking protocols that are reliable is *hard*. Things like lost packets and strange edge cases makes it difficult to keep one user's state in sync with another's user's state. This is fine if the state you are syncing is fairly simple like a text document or an SVG file, but the state of a DAW application is much more complicated.
* A DAW doesn't have complete control over how the state of 3rd party plugins is defined. We would have no choice but to send over the entire save state of a 3rd party plugin to other user each time something in that plugin is changed, and even then there is no garauntee that the plugin on the other user's end will load that state properly.
   * One possible solution is to only allow some third party plugins to be used with realtime online collaboration, perhaps plugins defined with a custom "realtime collaboration" CLAP extension, but I don't think this is what most users want.

That being said, the idea is not ruled out yet. I'll wait to after MVP is complete before giving it any more serious consideration.

Perhaps we can use a sort-of compromised solution, where instead of a full-on realtime online collaboration system, we simply include a "collaboration panel" where users can drag & drop clips, presets, and even entire mixer tracks into this panel as a collective pool of resources (essentially functioning like a newtork drive)?

# Architecture Overview - Frontend

## UI Library

For our UI frontend we will use the Rust bindings to [`GTK4`](https://github.com/gtk-rs/gtk4-rs).

### Why not use a Rust-native UI library?
* Established mature Rust-native UI libraries don't scale very well in terms of performance. Meadowlark will have a lot of widgets on different panels on the screen, including some particuarly complex ones on the timeline, piano roll, and horizontal FX rack. Projects with hundreds of clips on the timeline or hundreds of MIDI notes on the piano roll should still run with acceptable performance.
   * GTK4 helps us here by having both GPU-accelerated rendering as well as an efficient retained model which only repaints widgets that need to be repainted. Importantly it also has GPU-accelerated scrolling features which should help improve performance when scrolling/zooming the timeline, piano roll, and the horizontal FX rack.
* Other "high-performance" Rust UI toolkits are all still experimental and not production-ready. While we were originally using Meadowlark as a testbed for the [`Vizia`] UI toolkit, I feel the goals and motivations of Meadowlark has changed since then. I no longer want to use pure Rust for everything just for the sake of using Rust. I want Meadowlark to become a shippable product, and I think relying on experimental Rust libraries was seriously hampering that progress. (Especially since there is almost no one with experience writing UIs with those toolkits.)
* In addition, I find all existing Rust-native UI libraries to have sub-par text rendering quality. GTK4 has very high-quality text rendering, and also has excellent support for rendering text in other languages.

### Why not QT or JUCE?
* GTK4 is written in C, which allows its Rust bindings to be much more robust and complete as opposed to the nightmarish bindings to C++ libraries such as QT or JUCE. GTK4's Rust bindings are also very well documented, including a nice [`getting started guide`](https://gtk-rs.org/gtk4-rs/stable/latest/book/introduction.html).
* The [`ZRythm`](https://www.zrythm.org/en/index.html) DAW also uses GTK4 for its UI, so we already know that it has the features we need to create a modern DAW UI.
* GTK4 is fully themeable with CSS, making it easy for users of Meadowlark to create and distribute custom themes.
* We aim to create a new independent and open souce audio development ecosystem from the ground up, and so we are avoiding using anything based on the dominating JUCE ecosystem.

### Why not use Web technologies?
* No. Just no.
* More seriously, web tech is a huge CPU and memory hog, and doesn't fit our performance needs (don't try and tell me otherwise, it just is).
* Javascript is too slow for the complex needs of a DAW UI, and compiling to webassembly (and sharing resources between your webassembly code and your backend code) is a huge hassle. Also Javascript.
* I'm also just against this whole industry trend of "let's use web tech for everything" in general. It gives Chromium too much power, and it slows down the much needed innovation in native UI toolkits.

## UX Design

The full design document for the UI/UX of Meadowark can be found [`here`](https://github.com/MeadowlarkDAW/Meadowlark/blob/main/UX_DESIGN_DOC.md).

## Meadowlark Factory Library

The [`meadowlark-factory-library`] repo will house the factory samples and presets that will be included in Meadowlark.

All samples and preset will be licensed under the [`Creative Commons Zero`] (CC0) license. Please provide proof that we have the right to distribute any content before submitting it to be included into the factory library.

The sample library will mostly consist of "essentials" such as drum samples (both electronic and acoustic), drum loops (both electronic and acoustic), riser/faller effects, atmospheres, vocal phrases, etc.

In addition to the one-shot samples, we plan on including multisample libraries of "essential" instruments such as pianos, strings, guitars, etc. These multisample libraries will most likely use the [`SFZ`] format.

Contributions are always welcome, although keep in mind that only a basic factory library (if any at all) is planned for MVP.

# Architecture Overview - Backend

The backend is split up into several separate modular pieces. This allows any future developers to more easily use the backend code of Meadowlark to create their own DAWs with whatever frontend/workflow they want *(Tracker based DAW anyone?)*. In addition this will help to organize and separate areas of concern in the project, while also helping to improve hot compile times.

## Meadowlark-core-types
*license: MIT*

The [`meadowlark-core-types`] module simply houses basic types that are shared between the rest of the modules.

## Dropseed
*license: GPLv3*

The full design document for dropseed can be found [`here`](https://github.com/MeadowlarkDAW/dropseed/blob/main/DESIGN_DOC.md).

The [`dropseed`] library is the "heart" of Meadowlark's backend. It provides a highly flexible audio graph system with automatic delay compensation and summation of edges, as well as providing plugin hosting (with a special focus on CLAP plugins).

Its unique design treats all user-spawned nodes in the audio graph as if it were a CLAP plugin (or at least an internal plugin format very closely modelled after the CLAP spec). Internal plugins also have the option of presenting whatever interface they wish to the frontend (using `Box<dyn Any>`). In this approach the developer creates a different "plugin" for every aspect of their application (i.e. a "timeline track plugin", a "sample browser plugin", a "mixer plugin", a "metronome plugin", a "monitor plugin", etc.), and then connects them together in any way they wish (as long as there are no cycles in the graph).

Dropseed uses the [`clack`] library for hosting CLAP plugins.

## Rainout
*license: MIT*

The full design document for rainout can be found [`here`](https://github.com/MeadowlarkDAW/rainout/blob/main/DESIGN_DOC.md).

The [`rainout`] library is responsible for connecting to the system's audio and MIDI devices. It's goal is to provide a powerful, cross-platform, highly configurable, low-latency, and robust solution for connecting to audio and MIDI devices.

### Why not contribute to an already existing project like `RTAudio` or `CPAL`?

#### RTAudio
- This API is written in a complicated C++ codebase, making it very tricky to bind to other languages such as Rust.
- This project has a poor track record in its stability and ability to gracefully handle errors (not ideal for live audio software).

#### CPAL
In short, CPAL is very opinionated, and we have a few deal-breaking issues with its core design.

- CPAL's design does not handle duplex audio devices well. It spawns each input and output stream into separate threads, requiring the developer to sync them together with ring buffers. This is inneficient for most consumer and professional duplex audio devices which already have their inputs and outputs tied into the same stream to reduce latency.
- The API for searching for and configuring audio devices is cumbersome. It returns a list of every possible combination of configurations available with the system's devices. This is not how a user configuring audio settings through a GUI expects this to work.
- CPAL does not have any support for MIDI devices, so we would need to write our own support for it anyway.

Why not just fork `CPAL`?
- To fix these design issues we would pretty much need to rewrite the whole API anyway. Of course we don't have to work completely from scratch. We can still borrow some of the low-level platform specific code in CPAL.

## Meadowlark Plugins
*license: GPLv3*

The full design doc for this suite of plugins can be found [`here`](https://github.com/MeadowlarkDAW/meadowlark-plugins/blob/main/DESIGN_DOC.md).

Most of Meadowlarks' plugins will be housed in the [`meadowlark-plugins`] repo, and we will use the [`nih-plug`] plugin development framework for these. (Inline UIs for these plugins will be defined using a custom CLAP extensions). Although some of the more specialized plugins (like all of the "container" plugins) will live in the Meadowlark repo itself.

Our main focus will be on creating a suite of good quality mixing/mastering FX plugins. (Contribution on synths is welcome, but they are not a priority right now). We obviously don't have the resources to compete with the likes of iZotope or Fabfilter. The goal is more to have good enough quality to where a producer can create a "pretty good" sounding mix using Meadowlark's internal plugins alone.

Also while a full suite of plugins is one of our goals, for MVP we will only target just a few plugins.

Because we have a small team at the moment, we will focus more on porting DSP from other existing open source plugins to Rust rather than doing all of the R&D from scratch ourselves. People can still do their own R&D if they wish (and there are cases where we have to because there doesn't exist an open source plugin for that case), but there already exists some great DSP in the open source world (especially in synth [`Vital`]). I've noted other open source plugins we can port the DSP from in the plugin suite design doc linked above.

Also please note the goal of this repo is *NOT* to create a reusable DSP library. I believe those to be more of a hassle than they are worth, and they also serve to deter DSP experimentation and optimizations when developing plugins. Each plugin will have its own standalone and optimized DSP. We are of course still allowed to copy-paste portions of DSP between plugins as we see fit.

## Creek
*license: MIT*

The [`creek`] library handles realtime-safe disk streaming to/from audio files. It uses [`Symphonia`] to support a variety of codecs.

This will be used to playback long audio clips on Meadowlark's timeline.

The technical details of how this library works can be found in creek's [`readme`](https://github.com/MeadowlarkDAW/creek/blob/main/README.md).

## Pcm-loader (name in progress)
*license: MPL-2.0*

The [`pcm-loader`] library handles loading audio files into RAM.

It is mostly an easy-to-use wrapper around the [`Symphonia`] decoding library. This crate also handles resampling to a target sample rate either at load-time or in realtime during playback.

The resulting PcmRAM resources are always de-interleaved, and they are stored in their native sample format when possible to save memory. They also have convenience methods to fill de-interleaved f32 output buffers from any aribtrary position in the resource.

## Meadowlark Offline Audio FX (name in progress)
*license: GPLv3*

The [`meadowlark-offline-audio-fx`] repo will house various offline audio effects such at pitch shifting, time stretching, formant shifting, transient detection, convolution, etc. (Although none of these effects are really planned for MVP).

[`CLAP`]: https://github.com/free-audio/clap
[`Rust`]: https://www.rust-lang.org/
[`dropseed`]: https://github.com/MeadowlarkDAW/dropseed
[`clack`]: https://github.com/prokopyl/clack
[`Vizia`]: https://github.com/vizia/vizia
[`meadowlark-core-types`]: https://github.com/MeadowlarkDAW/meadowlark-core-types
[`rainout`]: https://github.com/MeadowlarkDAW/rainout
[`creek`]: https://github.com/MeadowlarkDAW/creek
[`pcm-loader`]: https://github.com/MeadowlarkDAW/pcm-loader
[`meadowlark-plugins`]: https://github.com/MeadowlarkDAW/meadowlark-plugins
[`meadowlark-offline-audio-fx`]: https://github.com/MeadowlarkDAW/meadowlark-offline-audio-fx
[`meadowlark-factory-library`]: https://github.com/MeadowlarkDAW/meadowlark-factory-library
[`nih-plug`]: https://github.com/robbert-vdh/nih-plug
[`Symphonia`]: https://github.com/pdeljanov/Symphonia
[`Vital`]: https://github.com/mtytel/vital
[`SFZ`]: https://sfzformat.com/
[`Creative Commons Zero`]: https://creativecommons.org/choose/zero/

