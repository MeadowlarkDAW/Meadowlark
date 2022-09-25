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
* We want to help build a new independent and open source audio development ecosystem from the ground up, and thus we will use no JUCE-based libraries.

# Goals
(*TODO*)

# Non-Goals
(*TODO*)


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
* We aim to create a new independent and open souce audio development ecosystem from the ground up, and so we are avoiding using anything based on JUCE.

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

The [`meadowlark-plugins`] repo will house the suite of Meadowlark's internal synth and FX plugins.

This repo will have two parts:
1. The pure plugin-spec-agnostic DSP for each plugin (each plugin in its own crate).
2. An optional crate that bundles all of these plugins into standalone plugins (with GUI) for use outside of Meadowlark. We will likely use [`nih-plug`] for this.

Note that the frontend for the internal plugins (inline plugins on the horizontal FX rack) in Meadowlark itself will live in the Meadowlark repo. The optional crate for bundling plugins is just for using the plugins outside of Meadowlark.

Our main focus will be on creating a suite of good quality mixing FX plugins. (Contribution on synths is welcome, but they are not a priority right now). We obviously don't have the resources to compete with the likes of iZotope or Fabfilter. The goal is more to have good enough quality to where a producer can create a "pretty good" mix using Meadowlark's internal plugins alone.

Also while a full suite of plugins is one of our goals, for MVP we will only target just a few plugins.

Because we have a small team at the moment, we will focus more on porting DSP from other existing open source plugins to Rust rather than doing all of the R&D from scratch ourselves. People can still do their own R&D if they wish (and there are cases where we have to because there doesn't exist an open source plugin for that case), but there already exists some great DSP in the open source world (especially in synth [`Vital`]). I've noted other open source plugins we can port the DSP from in the design doc linked below.

Please note the goal of this repo is *NOT* to create a reusable DSP library. I believe those to be more of a hassle than they are worth, and they also serve to deter DSP experimentation and optimizations when developing plugins. Each plugin will have its own standalone and optimized DSP. We are of course still allowed to copy-paste portions of DSP between plugins as we see fit.

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

## Meadowlark Offline Audio FX
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

