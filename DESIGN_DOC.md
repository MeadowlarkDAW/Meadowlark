# Meadowlark Design Document

Meadowlark is a FREE and open-source DAW (Digital Audio Workstation) for Linux, Mac and Windows. It aims to be a powerful recording, composing, editing, sound designing, mixing, and mastering tool for artists around the world, while also being intuitive and customizable.

# Objective

Why am I creating a new DAW from scratch? Why not contribute to an open-source DAW that already exists?

* I want a DAW with a novel timeline workflow reminiscent of FL Studio. I also want a clean and fully themeable UI. This would of course require an overhaul on existing DAWs.
* I want a more powerful and flexible audio graph engine for advanced routing capabilities. This would also require quite an overhaul on existing DAWs.
* I want a DAW that is truly FREE and open source. No "you have to pay for a pre-compiled binary" models.
* I want to embrace the new open-source and developer-friendly CLAP audio plugin standard with first-class support.
* I am passionate about music software and want to make something I want to use myself (and hopefully others will too). It would be cool to one day make this my career, but that's not my main goal.

# Goals

* A highly flexible and robust audio graph engine that lets you route anything anywhere, while also automatically adding delay compensation where needed
* First-class support for the open source CLAP audio plugin standard
    * Additional support for LV2, VST, and VST3 plugins via a CLAP bridge
* An easy-to-use settings panel for connecting to system audio and MIDI devices
* A novel "hybrid" approach to arranging clips on the timeline, combining ideas from the "free-flow" style of arranging in FL Studio with the more traditional track-based approach found in DAWs such as Ableton Live and Bitwig
    * Audio clips with quick controls for crossfades
    * Piano roll clips
    * Automation clips, with support for curved automation lines
    * Bounce entire tracks or sections of a track to audio
* Record audio and MIDI into clips on the timeline
    * Multiple sources can be recorded at once into multiple clips
* Global "swing" parameter
* A versatile piano roll for editing piano roll clips
    * Per-note modulation editing
    * Quantization features
* A standard mixer
    * Each mixer track will contain the standard controls you would expect: a fader, db meter, pan knob, solo button, mute button, and an "invert phase" button.
    * Busses and send tracks
    * Instruments with multiple outputs (like drum machines) can be assigned to multiple mixer tracks
* A flexible patching plugin reminiscent of FL Studio's Patcher plugin
    * Built-in macro system
* A suite of internal plugins, focused mostly on mixing and mastering
    * FX
        * Parametric EQ
        * Basic single-band compressor
        * 3-band compressor (like the Multiband Dynamics plugin in Ableton Live)
        * Limiter
        * Gate
        * Distortion (including waveshaping, bitcrushing, and maybe some amp models)
        * Delay
        * Reverb
        * DJM-Filter-like performance filter plugin (like Xfer's DJM-Filter)
        * Chorus
        * Flanger
        * Phaser
        * Filter (including comb filters)
        * Tremolo/Auto-Pan plugin
        * Utility (gain, pan, invert phase, swap l/r)
        * Convolver (for convolution reverbs and other effects that use an impulse response as input)
        * Multiband, Mid-Side, & L/R splitters
        * A/B plugin
        * (Likely more in the future)
    * Generators
        * Noise generator
        * Tone generator
        * Single-shot sampler
        * Drum multisampler
        * Multisampler (Soundfont, SFZ, Decent Sampler)
        * (Maybe bundle Surge XT with Meadowlark?)
    * Visualizers
        * Oscilloscope
        * Spectrometer/Spectrogram
        * Goniometer/Phase correlation meter
        * Loudness meter (with support for various loudness standards such as RMS and EBU)
* Integrated browser for browsing samples, presets, & plugins
    * Search bar
    * Play back audio clips as they are selected in the browser
* An included factory library of samples, multisamples (using the SFZ or Direct Sampler format), and presets
* Properties panel that shows advanced settings for whatever is currently selected (like Bitwig's properties panel)
* Export the entire project or just a selected region of your project to a WAV file
    * Export project as stems
* An official website (url is https://meadowlark.app)

# Non-Goals

* No 64 bit audio (unless I find a legitimate reason for it)
* The faders & pan knobs on the mixer will not be automatable. Instead users will insert the "Utility" plugin and automate that instead.
* The suite of internal plugins will be focused mainly on audio FX for mixing/mastering. Internal synth plugins are lower priority (and not even that necessary since high quality open source synths like Vital and SurgeXT already exist).
* LV2, VST2, and VST3 plugins will not receive the same level of support as CLAP plugins.
* No support for the AUv2, AUv3, LADSPA, WAP (web audio plugin), or VCV Rack plugin formats.

# Repository Overview

* [`main repository`](https://github.com/MeadowlarkDAW/Meadowlark)
   * license: GPLv3
* [`meadowlark-plugins`](https://github.com/MeadowlarkDAW/meadowlark-plugins)
   * license: GPLv3
   * This repository houses the majority of Meadowlark's internal plugins
* [`meadowlark-factory-library`](https://github.com/MeadowlarkDAW/meadowlark-factory-library)
   * license: Creative Commons Zero (CC0)
   * This repository will house the factory samples and presets that will be included in Meadowlark.
* [MeadowlarkDAW.github.io](https://github.com/MeadowlarkDAW/MeadowlarkDAW.github.io)
    * Meadowlark's website
* [`project-board`](https://github.com/MeadowlarkDAW/project-board)
    * This houses the kanban-style project board for the entire project.
    * (I'm not a fan of how GitHub Projects assigns every task as an "issue" in that repository and clutters up the issues tab. So I'm
    using this repository as a dedicated place to hold all of the generated issues instead.)

# Tech Stack

* Modern C++20
* JUCE
* nanovg
* DSP from various open source plugins such as Surge XT and Vital
* (probably a lot more I'll come across in the future)