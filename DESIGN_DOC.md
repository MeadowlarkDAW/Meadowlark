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

# How to Contribute

Please note I have decided to NOT accept any contributions to the development of the UI or the core backend engine of Meadowlark for the foreseeable future. I have a very specific vision for how all of these pieces will fit together, and my previous attempts to communicate this vision through design documents and then attempting to delegate these tasks to volunteers were both too time consuming and ineffective.

That being said, if you wish to contribute to the development of Meadowlark, I do still very much need help in these other areas:
* [`rainout`]
   * This crate is responsible for connecting to the system's audio and MIDI devices. It's goal is to provide a powerful, cross-platform, highly configurable, low-latency, and robust solution for connecting to audio and MIDI devices.
   * [`design document`](https://github.com/MeadowlarkDAW/rainout/blob/main/DESIGN_DOC.md)
* [`meadowlark-plugins`]
   * The DSP for our internal plugins.
   * [`design document`](https://github.com/MeadowlarkDAW/meadowlark-plugins/blob/main/DESIGN_DOC.md)
* [`meadowlark-offline-audio-fx`]
   * The DSP for various offline audio effect DSP such at pitch shifting, time stretching, formant shifting, transient detection, convolution, etc.
   * *design document WIP*
* [`meadowlark-factory-library`]
   * This will house the factory samples and presets that will be included in Meadowlark.
   * See the [`readme`](https://github.com/MeadowlarkDAW/meadowlark-factory-library) for more details.
* And of course any donations are very much appreciated! [`(donation link)`](https://liberapay.com/BillyDM)
   * DISCLOSURE: Please note that Meadowlark is currently not an official organization with employees, and I (BillyDM) am currently the only one dedicating their full-time to this project. So for the foreseeable future, all proceeds donated to this Liberapay account will go to finance me, Billy Messenger, personally.

# Goals/Non-Goals

## Goals
> Because this is a large and ambitious project, please keep in mind that goals marked with `(Not MVP)` are not considered goals for the first MVP (minimum viable product) release. Once MVP is done we will have a more detailed roadmap for the development cycle.

* A highly flexible and robust audio graph engine that lets you route anything anywhere, while also automatically adding delay compensation where needed.
* First-class support for the open source [`CLAP`] audio plugin standard.
   * Additional support for LV2 plugins via an LV2 to CLAP bridge. We will most likely use bindings [`Carla`] to achieve this.
   * `(Maybe MVP?)` Additional support (*maybe) for VST3 plugins via a VST3 to CLAP bridge. We will most likely use bindings [`Carla`] to achieve this. (Although if and when CLAP becomes widely adopted enough, we may decide to drop support for VST3 altogether because of all its issues with licensing and complexity.)
   * `(Maybe MVP?)` Additional support (*maybe) for VST2 plugins via a VST2 to CLAP bridge. We will most likely use bindings to [`Carla`] to achieve this.
      * \* (See the `Non-Goals` section below where I address my concerns with VST2 and VST3).
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
* We will (probably) not support 64 bit audio in our audio graph engine.
   * Adding 64 bit audio to our audio graph engine would add a whole lot of complexity, and I don't think it's even worth the hassle.
   * It is shown time-and-time again that 64 bit audio has little to no benefit in terms of quality over 32 bit audio. The noise introduced by 32 bit quantization errors is already well below audible range, and there are far more important factors that contribute to sound quality such as antialiasing techniques.
      * The one area where 64 bit audio on the audio graph level maybe could actually make a difference is CV (control voltage) ports, but like I mentioned above, having "the DAW is a modular instrument" is not a goal for Meadowlark.
   * 32 bit allows you to fit twice as many numbers into a single vectorized CPU operation, so performance can theoretically double (pun not intended) if used properly.
   * (I may change my mind on this one if we find a legimate use case for 64 bit audio on the audio graph level.)
* The faders & pan knobs on the mixer will not be automatable. Rather we will encourage users to insert the "Utility" plugin and automate that instead.
* The suite of internal plugins will be focused mainly on audio FX for mixing/mastering. Internal synth plugins are lower priority (and not even that necessary since high quality open source synths like Vital and SurgeXT already exist). We could even consider just packaging synths like SurgeXT with Meadowlark itself. (The developer of Vital probably wouldn't be cool with us packaging Vitalium with Meadowlark and I totally understand that, so we probably won't do that).
* LV2, VST2, and VST3 plugins will not recieve the same level of support as CLAP plugins. Those other formats will be supported through an intermediate LV2/VST2/VST3 to CLAP bridge, so functionality will be limited to whatever those bridges can support.
* We will not support the AUv2, AUv3, LADSPA, WAP (web audio plugin), or VCV Rack plugin formats.
* Non-destructive pitch shifting & time-stretching effects will not be supported for long audio clips that are streamed from disk. Users must use destructive editing in that case.
* Aside from a few plugins such as the Parametric EQ, Limiter, Bus Compressor, and the Vocal Compressor, we will not be doing much in-house DSP research. Rather the plan is to port DSP from existing open source plugins for the majority of our internal plugins (a lot of it will be ported from the Vital synth).

> ### My concerns with VST2 and VST3
>
> If and when CLAP becomes widely adopted enough, we may decide to drop support for VST3 altogether because of all its issues with its developer-unfriendly licensing policies and its over-engineered complexity and complicated C++ codebase. We may also decide to not support VST2 at all since it also has licensing issues and because it is an old and outdated standard. We could instead encourage users to use a wrapper plugin if they still need to use legacy plugins, but of course I'm unsure this is a good idea.
>
> My line of reasoning is that I *really* want to give plugin companies an actual incentive to adopt the new developer-friendly CLAP standard. If Meadowlark does happen to become popular, companies would have no choice but to adpot the CLAP standard if they want to tap into the Meadowlark userbase. This may sound a bit petty, but the VST2/VST3 standard has been a plauge on this industry for too long, and I believe I have one of the biggest opportunities out of anyone else in the world to change it.
>
> Of course this is a bit of a catch-22 situation where it could be difficult to even get an initial userbase if we didn't support these ubiquitous standards up-front. So we will very likely at least support VST3 for the foreseeable future.

## A Special "Maybe" Goal

There is feature suggestion being thrown around a lot I feel needs special attention, and that is "realtime online collaboration".

While this is something I definitely want, and it would be a revolutionary new way for artists and bands to collaborate, I have some serious doubts how feasible it would actually be to implement this in practice. Some of my concerns include:

* Creating custom networking protocols that are reliable is *hard*. Things like lost packets and strange edge cases makes it difficult to keep one user's state in sync with another's user's state. This is fine if the state you are syncing is fairly simple like a text document or an SVG file, but the state of a DAW application is much more complicated.
* A DAW doesn't have complete control over how the state of 3rd party plugins is defined. We would have no choice but to send over the entire save state of a 3rd party plugin to the other user each time something in that plugin is changed, and even then there is no garauntee that the plugin on the other user's end will load that state properly.
   * One possible solution is to only allow some third party plugins to be used with realtime online collaboration, perhaps plugins defined with a custom "realtime collaboration" CLAP extension, but I don't think this is what most users want.

That being said, the idea is not ruled out yet. I'll wait to after MVP is complete before giving it any more serious consideration.

Perhaps we can use a sort-of compromised solution, where instead of a full-on realtime online collaboration system, we simply include a "collaboration panel" where users can drag & drop clips, presets, and even entire mixer tracks into this panel as a collective pool of resources (essentially functioning like a shared newtork drive)?

# Repository Overview
Here I'll list an overview of the purpose of each of Meadowlark's repositories, as well as some notable dependencies.

* [`main repository`](https://github.com/MeadowlarkDAW/Meadowlark)
   * license: [`GPLv3`]
   * This houses the core application of Meadowlark including the UI, state management system, and the glue tying it all to the backend engine.
* [`dropseed`]
   * license: [`GPLv3`]
   * This houses the core backend engine. More specifically it provides a highly flexible audio graph system with automatic delay compensation and summation of edges, as well as providing plugin hosting (with a special focus on CLAP plugins).
   * It uses the [`audio-graph`](`https://github.com/MeadowlarkDAW/audio-graph`) crate, which houses the pure abstract graph compilation algorithm. This helps us separate areas of concern and focus on the pure algorithm at hand.
   * It uses the [`clack`](https://github.com/prokopyl/clack) crate for its bindings to the CLAP plugin API.
* [`rainout`]
   * license: [`MIT`]
   * This crate is responsible for connecting to the system's audio and MIDI devices. It's goal is to provide a powerful, cross-platform, highly configurable, low-latency, and robust solution for connecting to audio and MIDI devices.
   * [`design document`](https://github.com/MeadowlarkDAW/rainout/blob/main/DESIGN_DOC.md)
* [`creek`]
   * license: [`MIT`]
   * This crate handles realtime-safe disk streaming to/from audio files.
   * It uses the [`Symphonia`] crate for decoding a wide variety of codecs.
* [`pcm-loader`] (name in progress)
   * license: [`MPL-2.0`]
   * This crate handles loading audio files into RAM. It is mostly an easy-to-use wrapper around the [`Symphonia`] decoding library.
   * This crate also handles resampling to a target sample rate either at load-time or in realtime during playback. It uses the [`samplerate-rs`](https://github.com/MeadowlarkDAW/samplerate-rs) crate for samplerate conversion.
* [`meadowlark-plugins`]
   * license: [`GPLv3`]
   * This repository houses the majority of Meadowlark's internal plugins. These will be built using the [`nih-plug`] plugin development framework.
   * [`design document`](https://github.com/MeadowlarkDAW/meadowlark-plugins/blob/main/DESIGN_DOC.md)
* [`meadowlark-clap-exts`](https://github.com/MeadowlarkDAW/meadowlark-clap-exts)
   * license: [`MIT`]
   * This repository houses our custom CLAP extensions that both our internal plugins and any external plugins can use to better integrate with Meadowlark.
   * Most important is the extension that allows defining custom inline UIs inside Meadowlark's horizontal FX rack.
* [`meadowlark-offline-audio-fx`] (name in progress)
   * license: [`GPLv3`]
   * This repository will house various offline audio effect DSP such at pitch shifting, time stretching, formant shifting, transient detection, convolution, etc.
   * *design document WIP*
* [`meadowlark-factory-library`]
   * license: [`Creative Commons Zero`] (CC0)
   * This repository will house the factory samples and presets that will be included in Meadowlark.
* [`project-board`]
   * This houses the kanban-style project board for the entire project.
   * (I'm not a fan of how GitHub Projects assigns every task as an "issue" in that repository and clutters up the issues tab. So I'm using this repository as a dedicated place to hold all of the generated issues instead.)

[`CLAP`]: https://github.com/free-audio/clap
[`Rust`]: https://www.rust-lang.org/
[`dropseed`]: https://github.com/MeadowlarkDAW/dropseed
[`clack`]: https://github.com/prokopyl/clack
[`Vizia`]: https://github.com/vizia/vizia
[`rainout`]: https://github.com/MeadowlarkDAW/rainout
[`creek`]: https://github.com/MeadowlarkDAW/creek
[`pcm-loader`]: https://github.com/MeadowlarkDAW/pcm-loader
[`meadowlark-plugins`]: https://github.com/MeadowlarkDAW/meadowlark-plugins
[`meadowlark-offline-audio-fx`]: https://github.com/MeadowlarkDAW/meadowlark-offline-audio-fx
[`meadowlark-factory-library`]: https://github.com/MeadowlarkDAW/meadowlark-factory-library
[`project-board`]: https://github.com/MeadowlarkDAW/project-board
[`nih-plug`]: https://github.com/robbert-vdh/nih-plug
[`Symphonia`]: https://github.com/pdeljanov/Symphonia
[`Vital`]: https://github.com/mtytel/vital
[`SFZ`]: https://sfzformat.com/
[`Carla`]: https://github.com/falkTX/Carla/
[`GPLv3`]: https://choosealicense.com/licenses/gpl-3.0/
[`MIT`]: https://choosealicense.com/licenses/mit/
[`MPL-2.0`]: https://choosealicense.com/licenses/mpl-2.0/
[`Creative Commons Zero`]: https://creativecommons.org/choose/zero/

