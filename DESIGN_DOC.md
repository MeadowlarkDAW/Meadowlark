# Meadowlark Design Document

Meadowlark is a FREE and open-source DAW (Digital Audio Workstation) for Linux, Mac and Windows. It aims to be a powerful recording, composing, editing, sound designing, mixing, and mastering tool for artists around the world, while also being intuitive and customizable.

# Objective

Why create a new DAW from scratch? Why not contribute to an open-source DAW that already exists?

* We want a DAW with a workflow reminiscent of FL Studio, as well as a clean, intuitive, and fully themeable UI. This would require an overhaul on existing DAWs.
* We want a more powerful and flexible audio graph engine for advanced routing capabilities. This would also require quite an overhaul on existing DAWs.
* We want a DAW that is truly FREE and open source. No "you have to pay for a pre-compiled binary" models.
* We want to embrace the new open-source and developer-friendly CLAP audio plugin standard with first-class support.

# Goals / Non-Goals

*TODO*

# Repository Overview

* [`main repository`](https://github.com/MeadowlarkDAW/Meadowlark)
   * license: GPLv3
* [`Dropseed`](https://github.com/MeadowlarkDAW/dropseed)
    * license: GPLv3 (maybe MIT later on?)
    * This repository houses the audio graph engine, plugin hosting engine, system IO, and a general purpose DAW engine that is eventually planned to be used by Meadowlark
* [`meadowlark-factory-library`](https://github.com/MeadowlarkDAW/meadowlark-factory-library)
   * license: Creative Commons Zero (CC0)
   * This repository will house the factory samples and presets that will be included in Meadowlark.
* [`MeadowlarkDAW.github.io`](https://github.com/MeadowlarkDAW/MeadowlarkDAW.github.io)
    * Meadowlark's website

*TODO: rest of the repositories*

# Tech Stack

*TODO*

# Architecture

*TODO*