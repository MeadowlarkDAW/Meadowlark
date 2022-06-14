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

# Architecture Overview

The code architecture of Meadowlark will mainly be dived into three layers: The backend layer, the program layer, and the ui layer.

## Backend (Engine) Layer

This layer owns the bulk of the "engine", which owns the audio graph and the plugins it hosts. It also automatically recompiles the audio-graph behind-the-scenes when necessary.

The bulk of this engine lives in a separate crate called [`dropseed`]. Having this live in its own crate will make it easier for developers to create their own frontend for their own open-source DAW if they wish.

The engine takes messages from the program layer to spawn plugins, remove plugins, and to connect plugins together. The engine then sends events back to the program layer describing which operations were successful and which were not. This message-passing model also allows the engine to run fully asynchronously from the rest of the program.

The events that the engine sends back may contain `PluginHandle`'s, which the program layer can use to interface with the plugin such as controlling its parameters.

Everything in the audio graph is treated as if it were a "plugin", including the timeline, the metronome, and the sample browser. This internal plugin format is very closely modelled after the [`CLAP`] plugin format.

## Program (State) Layer

This layer owns the state of the program.

It is solely in charge of mutating this state. The backend layer and the UI layer cannot mutate this state directly (with the exception of some UI-specific state that does not need to be undo-able such as panel or window size). The backend layer indirectly mutates this state by sending events to the program layer, and the ui layer indirectly mutates this state by calling methods on the ProgramState struct which the UI layer owns. 

The program layer also owns the handle to the audio thread and is in charge of connecting to the system's audio and MIDI devices. It is also in charge of some offline DSP such as resampling audio clips.

## UI (Frontend) Layer

This layer is in charge of displaying a UI to the user. It is also responsible for running scripts.

The UI is implemented with the [`VIZIA`] GUI library.

[`CLAP`]: https://github.com/free-audio/clap
[`Rust`]: https://www.rust-lang.org/
[`dropseed`]: https://github.com/MeadowlarkDAW/dropseed
[`VIZIA`]: https://github.com/vizia/vizia