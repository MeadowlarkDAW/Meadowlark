# Meadowlark MVP Design Document

Meadowlark is a (currently incomplete) project that aims to be a FREE and open-source DAW (Digital Audio Workstation) for Linux, Mac and Windows. Its goals are to be a powerful recording, composing, editing, sound designing, mixing, and mastering tool for artists around the world, while also being intuitive and customizable.

***This project is still in the early stages of development and is not ready for any kind of production use or any alpha/beta testing.***

# Objective

*TL;DR: we want a solid, stable, free & open-source, more-modular audio digital audio workstation (DAW).*

A DAW is a unique project: it's a large and complex one, but if successful, it can drive change for many different technologies and define new standards. It's important for this work to be done **in the open** to avoid the mistakes of other technologies/standards, and to accelerate the pace of innovation.

Why not contribute to an open-source DAW that already exists?
 - Existing DAWs rely on older technologies and standards, such as C++. While fine for the task of writing audio software, C++ makes it trickier to create a stable and maintainable large codebase (cross-platform support, ease of writing modular code, etc). Stability is a high priority for us.
 - People have different tastes in workflow, and having a monolithic DAW architecture locks projects into a specific use-case. While an existing open source DAW may be great for one type of workflow, it is not for other types of workflows. With a **more modular** design, developers can more easily swap out components/libraries to suit the needs of *their* particular use-case, and a whole ecosystem of open source DAWs with different workflows can be acheived.
    - This modular ecosystem is called the [`RustyDAW`] project. One of Meadowlark's main purposes is to be both a testbed and eventually the flagship product of RustyDAW project.
- Meadowlark is also being used as a testbed for the [`Tuix`] GUI library project, in which the developer is working closely with the Meadowlark project.

We believe Rust to be the perfect language for this project because of its design philosophy:
 - No garbage collection, so still (more easily) audio-safe
 - Cross-platform by default: works on Windows, Mac OS, and Linux, across multiple different CPU architectures, without much compilation fuss
 - The modules and crates system makes it easy to split your code into distinct modular components, and `cargo` handles all of the compilation for you
 - Rust's safety guarantees will significantly reduce the occurrence of DAWs crashing


# Scope for MVP (minimum viable product)

While our long term goals are to grow into a fully-featured DAW with a feature set that competes with existing commercial DAWs like FL, Live, and Bitwig, we need to limit the scope to achieve minimum viable product (MVP) release.

### Goals
Note these goals are for a specific Meadowlark application. However, the backend engine as part of the [`RustyDAW`] project will be designed to allow any developer to easily use the same engine for whatever GUI interface/workflow they wish to create.

* Cross-platform (Mac, Windows, and Linux)
* Multi-track timeline with audio clips. Audio clips can added, moved, removed, sliced, and copied freely around the timeline
* Load wav, aac, flac, mp3, pcm, and ogg vorbis files as audio clips (afforded to us by the [`Symphonia`] crate)
* Export to wav file
* Effects on audio clips (MVP will include gain, crossfade, pitch shift (doppler), and reverse)
* Robust audio graph
* Grid snapping controls on timeline
* Tempo selection
* Loop range on timeline
* Control clips on the timeline (like MIDI / automation clips). MVP features will include:
    * Clips can be added, moved, removed, sliced, and copied freely around the timeline
    * Drop-down to select MIDI CC that this "automation node" will be attached to.
    * Looping options for control clips
* Piano roll editor for control clips. MVP features will include:
    * Select scale (12 EDO by default)
    * Paint, move, remove, and resize notes in the piano roll (FL style)
    * Grid snap with ability to select grid size (bar, 8th note, 16th note, triplet, etc)
    * Simple quantization of notes to the grid
    * A per-note velocity/pan editor on the bottom
* Track headers on each track in the timeline. MVP features will include:
    * Track name. User double-clicks to rename.
    * Fader & pan controls
    * Solo, Mute, and arm record buttons
    * Change color of header/track for organization
    * Drop-down to select where to route the track in the mixer (master or effect bus)
* Recording into audio & midi clips
* Vertical effect rack
* Sample browser panel
* Plugin browser panel
* Properties panel
* Host VST2 (or LV2) plugins (GUI prefered, but not MVP).
* Built-in MVP plugins will include
    * A basic gain & pan plugin
    * A basic digital clipping plugin
    * An audio send plugin
* Settings
    * Interface for selecting audio hardware input/outputs
    * Interface for selecting MIDI devices. Note that we only need to support basic MIDI keyboard functionality for MVP.

### Non-Goals
To keep the scope manageable with such a small team, we will NOT focus on these features for the MVP:

***Please note these are only non-goals for the minimum viable project. Most or all of these features will be added afterwards before the first official release of Meadowlark.***

* Export mp3, ogg vorbis, aac, etc.
* Audio clip effects such as time stretching and non-doppler pitch shifting
* Streaming audio files from disk
* Recording any type of audio or MIDI data
* Mixer view
* Sidechain routing to plugins
* Grouped tracks
* Non-4/4 time signatures and time signature changes
* Project tempo automation
* Advanced quantization features in piano roll
* Per-note modulation
* Edit/view multiple control clips at once in piano roll
* Custom time marks & chord view on timeline
* Preset browser
* "Live/Bitwig" style clip launcher
* A full suite of built-in synth and effect plugins
* A "Live/Bitwig" style horizontal effect rack
* A "Live/Bitwig" style of grouping together effects and splitting them by mid/side, multiband, etc.
* Custom application themes
* Hosting VST3, AU, and (maybe) LV2 plugins
* Plugin sandboxing
* Advanced loop recording
* Swing tempo


# Backend Design (MVP)


## Time Keeping

Keeping accurate track of time of events is crucial. However, automated tempos, swing tempos, and different audio device samplerates poses a challenge. The proposed solution will lie inside the [`rusty-daw-timeline`] repos.

The timekeeping system in this crate is designed as such:

### Intended workflow:
1. The GUI/non-realtime thread stores all events in `MusicalTime` (unit of beats).
2. The GUI/non-realtime thread creates a new `TempoMap` on project startup and whenever anything about the tempo changes. It then sends this new `TempoMap` to the realtime-thread.
3. When the realtime thread detects a new `TempoMap`, all processors with events stored in `MusicalTime` use the `TempoMap` plus the `SampleRate` to convert each `MusicalTime` into the corresponding discrete `SampleTime` (or sub-samples). It keeps this new `SampleTime` for all future use (until a new `TempoMap` is recieved).
4. When playback occurs, the realtime-thread keeps tracks of the number of discrete samples that have elapsed. It sends this "playhead" to each of the processors, which in turn compares it to it's own previously calculated `SampleTime` to know when events should be played.
5. Once the realtime thread is done processing a buffer, it uses the `TempoMap` plus this `SampleRate` to convert this playhead into the corresponding `MusicalTime`. It then sends this to the GUI/non-realtime thread for visual feedback of the playhead.
6. When the GUI/non-realtime thread wants to manually change the position of the playhead, it sends the `MusicalTime` that should be seeked to the realtime thread. The realtime thread then uses it, the `TempoMap`, and the `SampleRate` to find the nearest (floored) sample to set as the new playhead.

### Time Keeping Data types:
- `MusicalTime` - unit of "musical beats" - internally represented as an `f64`
- `Seconds` - unit of time in seconds - internally represented as an `f64`
- `SampleTime` - unit of time in number of discrete samples - internally represented as an `i64`
- `TempoMap` - a map of all tempo changes in the project (like an automation track) - (internal representation to be decided)

## Hardware I/O

We will store this functionality in the [`rusty-daw-io`] repo.

** *Edit note: We are also looking into creating bindings to [RtAudio](https://github.com/thestk/rtaudio) as a potential solution, so this section below may become irrelevant.*

The goal of this crate are as follows:
* Search for all available audio servers, audio devices, and MIDI devices on the user's system. It then sends back platform-agnostic information about all available configuration options, as well as default options for each given server/device.
* Create an easy-to-use interface that any GUI system can use to present available devices to the user, and then select and apply those settings. Whenever "apply" is selected, we will take the easy route and restart the whole audio engine (as opposed to trying to seamlessly update the existing one).
* Once devices have been selected, the user may then create "busses" that are attached to the ports of the selected device. This has the advantage of letting the user selectively use ports of each device (say if the user's audio interface has multiple microphone inputs) and put them into their own "bus" that they can rename to how they see fit. These bus IDs are mapped to an array index, which is used to tell the realtime thread's audio graph which io bus it should use.
* When selecting an audio server/device, this crate will attempt to choose a default stereo output bus and a mono input bus, as well as a single MIDI input controller bus. Note that there must always be at least one output bus, otherwise the stream will have nothing to callback to.
* Spawn a realtime callback closure with all input/output buffers neatly packaged in the arguments.
* Save and load configurations to an XML file.
* Ability to send a warning to the user if a server/device is unavailable when loading from a config file, while still trying to load the rest of config.

There is the tricky problem of how to sync audio inputs and outputs with low latency. Here, we will use a similar solution to how the Bitwig DAW (and probably others) handles this:
* Only allow selecting either a single duplex or a single playback device. Most hardware audio interfaces are already duplex, which are becoming increasingly popular and affordable. In addition, Mac OS's kernel already packages all devices into a duplex server for us (using CoreAudio). Linux can also acheive this using Jack or Pipewire. That only leaves Windows, where ASIO4ALL is the only real stable solution unfortunately. But trying to combine non-duplex devices ourselves is incredibly difficult without adding a bunch of latency. So we feel it is not worth dealing with this Windows-specific use case.

Note that we have decided against using [`cpal`] for this project, despite it's purpose being for low-latency realtime audio. There are three issues we take with it;
* Cpal returns a list of available devices as an array of each single possible configuration of devices, instead of just listing the supported options per-device. This makes it tricky to package the information in a way to present to the user via a GUI.
* Cpal's architecture does not support duplex audio well. It's decision to split all inputs and outputs into their own stream means that we have to manually sync input/output streams despite the device already being duplex.
* Cpal does not handle any MIDI, so we need to do that ourselves anyway. `Midir` is another option, but some platform's audio servers already package MIDI along with the duplex devices, allowing for better latency. We want to take advantage of those situations.

## Audio Graph

An audio graph is an algorithm that takes individual "nodes" of audio/control processors and arranges them in the order they should be processed. The audio graph may also provide information on how to best utilize multi-threading on the CPU (although multithreading is not MVP).

We will be using (and developing) the [`rusty-daw-audio-graph`] crate for our audio graph. (Meadowlark is pretty much the testground for all the rusty-daw repos).

For MVP, there will be these types of nodes in the graph:
* `TimelineTrack` - A single track in the timeline that outputs audio and control buffers (before any effects except for internal audio-clip effects). See the "Timeline Engine" section below for more details.
* `InternalPlugin` - A single internal effect or synth plugin
* `VST2Plugin` (or `LV2Plugin`) - A single VST2 (or LV2) plugin
* `GainNode` - A node that applies gain onto a signal. This will use de-clicking strategies.
* `PanNode` - A node that applies panning onto a signal. This will use de-clicking strategies.
* `DelayNode` - A node that applies delay compensation onto a signal
* `SumNode` - A node that sums signals together. This will use de-clicking strategies.

## Sampler Engine

This will act as a sampling engine for playing audio clips (whether on a timeline or a clip launcher), as well as any future sampling plugins. 

The proposed interface will look like this:

* Ability to create a `PCMResource` type, which is an *immutable* [`basedrop`] smart pointer to raw samples in memory. The immutability reflects the non-destructive nature of this engine. This type will also store the sample-rate of the data. These raw samples provided by the user of this crate should be:
    * Decompressed
    * De-interleaved (each channel in its own buffer)
    * Preferably in the original bit depth (we will handle automatically converting to f32/f64 on the fly)
    * Preferably in the original sample rate, regardless of the project's sample rate. A key design of this sampling engine is the ability to only need one resampling pass to do any type of effects like pitch-shifting or time stretching (as opposed to two or more if the data is converted to the project's sample rate on load, which will have a noticeable drop in quality)
* When applying an effect that does not change the duration or ordering of the samples (gain, fades, pan, etc.), the RT Thread will apply these effects in real-time.
* When applying an effect that *does* change the duration or ordering of the samples (pitch shift, time shift, reverse, etc.), then the non-RT thread will render these samples into a new buffer before sending them to the RT thread. However, we will keep the original samples around so we don't lose quality if the user modifies an effect.

While streaming samples from disk is an eventual goal of this project, it will not be part of the MVP.

## Control Data

We will like eventually use our own custom control spec instead of standard MIDI for our internal plugins/future custom plugin format. This is for the following reasons:

* The MIDI standard doesn't support all the features we want like:
    * Non 12-TET scales
    * Per-note modulation of pitch, volume, pan, etc (This is afforded by the MPE extension, though)
    * Fine control over sample-accurate audio-rate modulation
    * MIDI wants to take control of aspects we believe should be left to the DAW/plugin spec instead, such as plugin parameters.
* The MIDI2 standard is more promising, but there are several issues we take with it:
    * The spec is huge and complicated.
    * We would have to rely on a third party organization to include any potential features in the future.
    * It heavily prioritizes being fully backwards compatible with MIDI. We have some doubts on how acheivable this actually is.
    * MIDI2 wants to take control of aspects we believe should be left to the DAW/plugin spec instead, such as plugin parameters.
    * MIDI2 is new and has little adoption. We have no idea if it will even be successful in the long run.

However, it is unclear at the moment of what a spec should look like. So for MVP, we will only focus on how the DAW stores control information and how it assembles this information into MIDI for use with VST2 (or LV2) plugins. If we succesfully do this, it will become clearer what needs to be done for any of our internal plugin specs.

### Piano Roll Notes

To support non-western 12-TET scales, the piano roll needs to store information about the current scale it is working with. We will use an index map instead of simply storing the pitch of each note in the note itself. This is so the piano roll can easily transpose notes in any scale, and it allows the user to experiment with different scale types and tuning with the same note information. Later (not MVP), notes with pitches that lie outside the current scale will be supported using MPE.

The "scale" will be internally stored as:
* `MusicalScale`
    * The pitch of the root note, stored as an `f64`. For standard western 12-TET tuning, this will be 440.0Hz (C4).
    * A `Vec<f64>` of how each note (including the root note, but not the octave note) in the scale relates to the root note as a ratio of pitch (1.0 being the same pitch as root, 2.0 being an octave above the root.) The length of this Vec will also tell the piano roll how many notes there are in the scale. For example, a Vec for standard western 12-TET equal tempered tuning would look like this:
    ```
        1.0,       // 2 ^ (0/12)
        1.059463,  // 2 ^ (1/12)
        1.122462,  // 2 ^ (2/12)
        1.189207,  // 2 ^ (3/12)
        1.259921,  // 2 ^ (4/12)
        1.33484,   // 2 ^ (5/12)
        1.414214,  // 2 ^ (6/12)
        1.498307,  // 2 ^ (7/12)
        1.587401,  // 2 ^ (8/12)
        1.681793,  // 2 ^ (9/12)
        1.781797,  // 2 ^ (10/12)
        1.887749,  // 2 ^ (11/12)
    ```

See the "Time Keeping" section above for an explanation of how `MusicalTime`, `SampleTime`, and the `TempoMap` come into play.

For MVP, the DAW will internally store each individual note in this format:
* `MusicalNote`
    * The `MusicalTime` when this note is ON.
        * Whenever this value is changes, or whenever the `TempoMap` changes, the playback engine will convert this into the corresponding discrete `SampleTime`. The playback engine uses this new `SampleTime` and compares it to the `SampleTime` of the current playhead to know the exact sample that this event should be triggered on.
    * The `MusicalTime` when this note is OFF.
        * The same `SampleTime` strategy as described in the entry above.
    * The octave of the note, stored as an `i8`. For example, in 12-TET, a value of 0 means the octave with root note C4, 1 is the octave with root note C5, -1 is the octave with root note C3, etc.
    * The index of the note in the scale stored as a `u16`. For example, in 12-TET, a value of 0 means C, a value of 1 means C#, a value of 2 means D, etc.
    * The initial velocity of the note (stored as `f64` in the range [0.0, 1.0])
    * The initial pan of the note (stored as `f64` in the range [-1.0, 1.0], where 0 is center)

We will not be dealing with polyphonic aftertouch, micro pitch expression, or other per-note automation in this MVP.

### Automation Nodes

We will only use standard MIDI CC automation lanes in this MVP. While the eventual goal is to have something more flexible with a custom internal control/plugin spec, MIDI CC lanes are still required for compatibility with 3rd party plugin formats such as VST2/VST3/LV2/AU.

For MVP, each node in the automation lane will be stored in this format:
* `AutomationNode`
    * The `MusicalTime` when this node occurs.
        * Whenever this value is changes, or whenever the `TempoMap` changes, the playback engine will convert this into the corresponding discrete `SampleTime`. The playback engine uses this new `SampleTime` and compares it to the `SampleTime` of the current playhead to know the exact sample that this event should be triggered on.
    * The "curve" of this node. This will be a non-exhaustive enum. MVP options will include:
        * `Linear`: Linear automation between this node and the next
        * `Step`: Constant automation from this node until the next one. When the next node is reached, the value immediately jumps to the value of that new node.
    * The "value" of the node stored as an `f64`. This will only be in the normalized range [0.0, 1.0] to allow automation clips to easily be copied and moved between lanes.

## Timeline Engine

We will store this functionality in the [`rusty-daw-timeline`] repo.

The goal of this engine is to take information like audio clips and control clips, and turns them into a sort-of "virtual instrument" that takes the playhead time (in `SampleTime`) as input, and outputs buffers of audio, control, and MIDI data. As such, each "track" in the timeline will act like a single node in the "AudioGraph" (explained in the AudioGraph section above).

For MVP, this crate will include these data structures that can be added to an individual `TimelineTrack` struct:

* `AudioClip`
    * An immutable [`basedrop`] smart pointer to a `PCMResource`. The immutability reflects the non-desctructive nature of this engine. There will only be one sample resource per audio clip.
    * The time in `Seconds` where this audio clips starts in the raw samples
    * The `ShiftMode` to use (pitch & time stretching). This is explained in the "Sampler" section above.
    * The `InterpQuality` to use (interpolation quality). This is explained in the "Sampler" section above.
    * Any information on clip fades. The structure of this data is still yet to be determined.
    * A `bool` on whether or not the audio should be reversed
* `PianoRollClip`
    * A vec of `MusicalNote`s (described in the "Piano Roll Notes" section above)
    * The `MusicalScale` to use (described in the "Piano Roll Notes" section above)
* `AutomationClip`
    * A vec of `AutomationNode`s (described in the "Automation Nodes" section above)

All mutable methods that add or change data in these clips must also include the `TempoMap` of the project as an argument. This is so all events in `MusicalTime` are correctly converted into the corresponding `SampleTime` for use with the playback engine (as described in the "Time Keeping" section).

In addition, the `TimelineTrack` struct will have a method that updates all events to the new `SampleTime` when the `TempoMap` of the project changes.

The `TimelineTrack` struct will include a method for "seeking" the playhead to a particular `SampleTime`.

And finally, the `TimelineTrack` will have a "process" method that outputs buffers of audio, control, and/or MIDI data.

## Internal Plugins

We will store this functionality in the [`rusty-daw-plugins`] repo.

MVP will only include very basic effect plugins. These are mostly just to test the GUI design of internal plugins.
* Gain & Pan - pretty self explanatory
* Hard Clipper - hard clipper with (maybe) antialiasing

An EQ may be added since people on this team are working on one anyway.

## Plugin Hosting

We will store this functionality in the [`rusty-daw-plugin-host`] repo.

For MVP, we will only focus on hosting VST2 (or LV2) plugins. Displaying the plugin's GUI would be nice, but is not strictly MVP.

# State Management

## Memory Management Data Structures

Managing project state and keeping it synced up with the GUI and the backend poses one of the biggest challenges to any large-scale application project. This is made more difficult with the fact that for any (fast) audio application, the thread which actually processes the sound must be realtime (meaning it cannot use any operations that may block the thread such as memory allocation, deallocation, and mutexes).

The problem with just using message channels is that in order to add more resources to the rt thread, the rt thread must allocate memory to store it somewhere (even if it's just allocating a place to store the pointer). One potential solution (that we won't use in Meadowlark) is to pre-allocate a maximum number of slots before-hand (e.g. having a maximum of 1000 audio clips). However, in something as complex as a DAW, it is undesirable to put this kind of limitation on the user, and it could waste a lot of memory to preallocate a bunch of slots for every kind of data structure we plan to use. In addition, it can be cumbersome to create an explicit message for every single operation (which can easily be in the thousands for a complex application).

Instead, Meadowlark will rely heavily on the [`basedrop`] memory management crate. Basedrop is a collection of thread-safe smart pointers specifically designed with realtime threads in mind. In particular, we care about these two data structures:

* `Shared` - Analogous to `Arc`, with the difference being once the pointer count goes to zero, it will queue its data to be deallocated by a separate `Collector` at a later time, instead of the data being deallocated right away. This prevents the situation of potentially deallocating on the rt thread when a pointer is dropped.
* `SharedMut` - A persistent data structure with interior mutability analagous to an atomic data structure like `AtomicBool` and `AtomicInt`. When the reader (rt thread) borrows this pointer to read its contents, it will return an immutable copy of that data for as long as it is borrowed (ensuring there are no data races). When the writer (GUI thread) wants to update the data in that pointer, it first clones the data, modifies that clone, and then pushes that new "version" onto the pointer. The next time the reader borrows the pointer, it will automatically grab the latest version of that data. Once a previously-used version is done being borrowed, it is queued to be deallocated by the `Collector`.

Of course one drawback is that we have to clone the entire data in order just to modify a single part of it. We can get clever though with how we structure our the data so we only clone the part we want to change, and for all the rest we just clone a `Shared` pointer to that data instead of cloning the whole thing (cloning a pointer is very cheap operation). We can even nest these `SharedPointers` in a tree-like hiearcy so we only need to clone the branch of the data we care about while leaving the rest of the tree untouched (e.g. the data of an Audio Graph Node).

In addition to [`basedrop`], we will use also use the [`atomic_refcell`] crate in a few places. This contains the `AtomicRefCell` data type, which is analagous to `RefCell` but is a thread-safe version. This is useful in the situation where we want the rt thread to be able to mutate a piece of data (like an audio buffer), but we still want to hold on to the pointer in the GUI thread so we can cheaply clone its `Shared` pointer when updating the state (e.g. when compiling a new schedule for the audio graph). In this case where the GUI thread will never actually read the contents of this data, using `AtomicRefCell` to defer mutability checking to runtime should never cause a panic. It's important to note that `AtomicRefCell` must *never* be used if the GUI thread does read the data (including just reading it to clone it). In that case use `SharedCell` instead.

In addition, some data structures that can have a bunch of elements (like a piano roll) could be optimized with special data structures like B-Trees which doesn't need to clone the whole thing to create a new "version" of that data. However, we won't worry about these kinds of optimizations for MVP.

Obviously with this approach, the GUI thread must continually allocate new data, and the `Collector` must continually collect the garbage (we've essentially created a garbage collector). But if we design the data just right, this shouldn't cause too much of a performance hit. And with the way these smart pointers as designed, this performance hit will only affect the GUI thread, not the rt thread.

It may also be worth looking into using a custom allocator which attempts to use a pool of previously "deallocated" memory instead of asking the OS to constantly allocate/deallocate, but that kind of optimization is not MVP.

## State Management System Architecture

This architecture is designed so each `AudioGraphNode` in the project is solely in charge of its own self-contained state. This will give us a great deal of flexibility and scalability in this project.

![State Management System Flowchart](/assets/design/state_management_system.png)

1. When an event is triggered (such as when interacting with a UI element, executing a script, loading a project, handling a backend error, etc.), it gets sent up the to the root "State System". This state system is responsible for handling and mutating the state of the entire program in a safe and predictable manner.
2. When dispatching an event, the state system may modify the audio graph (adding nodes, deleting nodes, connecting nodes, etc.). When this happens, the backend will automatically compile the whole graph into a "Compiled Schedule".
3. This new "Compiled Schedule" is then sent to the RT Thread via a `SharedCell`. The new schedule will be available to the RT Thread on the top of the next process loop.
4. When dispatching an event, the state system may modify the save state of a particular element. When this happens, all UI widgets which are bound to that specific save state will automatically be updated.
5. When dispatching an event, the state system may call various methods on the stored "handles" of elements to mutate them. It is up to the state system to make sure that the UI state, save state, and the state of all these backend handles are synced up.
6. A "handle" to an element is linked to the actual node in the RT Thread in some way. It is up to the particular node on what method it wishes to use for syncing (message/data ring buffer, `SharedCell`, etc.). This allows us great flexibility on how to structure each node. All mutated data will be available to the RT Thread at the top of the next process loop.
7. The backend has a "Resource Cache" that is used to store any loaded assets like audio files. The AudioGraphHandle also has its own internal pool of allocated nodes & buffers. When one of these elements gets deleted, they are automatically collected by [`basedrop`] and sent to the Collector Thread which deallocates them periodically (every 3 seconds or so). In addition, using persistent data structures like `SharedCell` will create garbage every time it is mutated, so this collector thread will also deallocate that.

# UI (MVP)

## Design Mockup

This is the current mockup of the UI design. Note that this design is experimental and is subject to change.

![UI Design Mockup](/assets/design/gui-mockup-main.png)

### Workflow

Here I'll list the purpose of each element in this design mockup:

### Top Bar
(from left to right)
- File Section
    - Drop-down menu for file/project related stuff
    - Open dialog
    - Save
    - Save As
    - Undo/Redo
- Tempo Section
    - Current tempo (can be edited by double-clicking and typing in the tempo)
    - A button that can be tapped in series to set the tempo
    - Time signature (placeholder, not MVP)
    - Open a dialog for editing swing/groove (placeholder, not MVP)
- Record Section
    - Open a dialog for editing additional record settings (placeholder, not MVP)
    - Main record button
    - Select what to record (audio, midi, audio & midi)
    - Loop recording mode (overwrite, new track, etc.) (placeholder, not MVP)
- Transport Section
    - Current position of the playhead in musical time. When clicked it toggles between that and real time (in seconds).
    - Play/Pause button
    - Stop Button
    - Button that brings the playhead to the previously seeked position
    - Button that toggles looping on/off
    - Button that toggles whether or not the playhead is automatically brought back to the previously seeked position when paused
- Monitor
    - Select type of audio montitor (oscilloscope, spectrograph, etc.) (only oscilloscope will be MVP)
    - Audio monitor
    - CPU monitor
- View Section
    - Toggle to open/close the timeline view
    - Toggle to open/close the clip launcher view  (placeholder, not MVP)
    - Toggle to open/close the mixer view (placeholder, not MVP)
    - Toggle to open/close the piano roll view
    - Toggle to open/close the automation editor view (placeholder, not MVP)
    - Toggle to open/close audio clip editor view (placeholder, not MVP)
    - Toggle to open/close the horizontal effect rack (placeholder, not MVP)
    - Toggle to open/close the vertical effect rack
    - Toggle to open/close the command palette (placeholder, not MVP)

### Left Panel
(tabs from top to bottom)
- Search panel (placeholder, not MVP)
- Sample browser
- Preset browser (placeholder, not MVP)
- Plugin browser
- File-system browser (placeholder, not MVP)
- Properties - This will contain a list of editable properties of whatever element (timeline track, audio clip, piano roll note) is currently selected

### Timeline
#### Timeline Toolbar
- Button that opens timeline settings (placeholder, not MVP)
- Select mode - Used to select & move clips
- Pencil mode - Pencil-in clips onto the timeline
- Erase tool - Erases clips
- Slice mode - Slices a clip into two pieces
- Button that toggles grid snapping on/off
- Drop-down to select the grid snapping mode
- Zoom-out
- Zoom-in
- Select area to zoom
- Reset zoom to default

#### Timeline Markers
(top to bottom)
- Bar indicator. The user can also drag on this bar to zoom/pan Bitwig-style.
- Loop indicator - Displays the current loop region. The user can drag handles on the loop region to move it.

#### Timeline Track Headers
##### Track Header
- Name of the track. The user can double-click on this to edit the name.
- Decibel meter
- Mixer fader
- Pan knob (to the right of the mixer fader)
- Record arm button
- Solo & mute buttons
- Button that toggles showing/hiding automation lanes underneath
- Botton that opens the synth plugin UI

Other notes:
- Tracks can be re-ordered by dragging on their respective headers
- Add button on the bottom of the last track header that adds a new track
##### Automation Lanes
- Close/delete button
- Drop-down to select what parameter to automate
- The arrow on the right side can be used to toggle between compact mode (small vertical height) and normal mode
- Automation lanes can be re-ordered by dragging on their respective headers
- Add button on the bottom of the last automation lane that adds a new automation lane

#### Timeline Grid
##### Clip
- Titlebar with name. This name can be double-clicked to be edited by the user
- The little arrows on the left/right sides of the titlebar can be dragged to resize clips
- Dragging anywhere else on the titlebar will move the clip
- The audio clip also has little handles that adjust the fades on the edges of audio clips
- Automation nodes can added/removed/moved directly on the clip itself (like FL Studio)

### Piano Roll
#### Piano Roll Toolbar
- Button that opens piano roll settings (like scale & tuning settings) (placeholder, not MVP)
- Standard select mode. The user can drag to select and move notes. The user can also double-click to add new or remove notes (Bitwig/Live style)
- Pencil mode - Pencils in notes by placing the note where the drag starts, and setting the length of the note where the drag ends (Reaper style)
- Paintbrush mode - Paints in notes by single-clicking and/or dragging to place multiple notes (FL Studio style)
- Erase mode - Erases notes
- Slice mode - Slices notes in half
- Quick Quantize - Quantizes the beginning of notes to the selected grid snapping
- Quantize - Opens a dialog for more advanced quantizing options (placeholder, not MVP)
- Button that toggles grid snapping on/off
- Drop-down to select the grid snapping mode
- Zoom-out
- Zoom-in
- Select area to zoom
- Reset zoom to default

#### Piano view (left panel)
- Display a musical keyboard and the octave

#### Grid
- Top bar - Like the timeline bar, this can be dragged to zoom/pan Bitwig-style.
##### Clips
- Clip headers will appear on top. Like on the timeline, the little arrows on the sides can be dragged to resize a clip.
- Piano roll notes

#### Per-note value editor (bottom panel)
- Dropdown to select what to edit (velocity, pan, etc.)
- Sliders below each note to adjust the value

### Vertical Track FX Panel (right side)
(top to bottom)
- Drop-down to select the track/bus
- A traditional effect rack. Clicking on a plugin will open its UI. Some simple built-in effects can have inline controls like a slider on a "send" plugin.
- A button below the bottom plugin that can be clicked to add a new plugin
- Pan/Mixer fader
- DB Meter
- Drop down to select which track/bus to send this track to

### Status Bar
- A status bar


[`Symphonia`]: https://github.com/pdeljanov/Symphonia
[`cpal`]: https://github.com/RustyDAW/cpal
[`rusty-daw-io`]: https://github.com/RustyDAW/rusty-daw-io
[`rusty-daw-timeline`]: https://github.com/RustyDAW/rusty-daw-timeline
[`rusty-daw-audio-graph`]: https://github.com/RustyDAW/rusty-daw-audio-graph
[`rusty-daw-core`]: https://github.com/RustyDAW/rusty-daw-core
[`rusty-daw-plugins`]: https://github.com/RustyDAW/rusty-daw-plugins
[`rusty-daw-plugin-ports`]: https://github.com/RustyDAW/rusty-daw-plugin-ports
[`basedrop`]: https://github.com/glowcoil/basedrop
[`deip`]: https://github.com/BillyDM/Awesome-Audio-DSP/blob/main/deip.pdf
[`RustyDAW`]: https://github.com/RustyDAW
[`Tuix`]: https://github.com/geom3trik/tuix
[`basedrop`]: https://github.com/glowcoil/basedrop
[`atomic_refcell`]: https://github.com/bholley/atomic_refcell
