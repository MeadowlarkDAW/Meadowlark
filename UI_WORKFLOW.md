# Main Toolbar

Which of these main toolbar controls should be always accessible, and which if any should be accessible through other means like a drop-down menu? How should these controls be grouped? Should the user be able to modify the toolbar how they like? Should some of these not live in the main toolbar?

### Transport Controls

What all is needed to control the timeline transport?

List of probable controls:

- Transport play/pause (obviously)
- Transport stop (silence all buffers and return to the most-recently seeked position). When clicked again this button will send the playhead to the beginning of the transport.
- Button to toggle looping on/off
- Button to toggle pausing behavior (whether to keep the playhead where it is when paused, or whether to return the playhead to the most-recently seeked position).
- A number display of the current playhead position. Also it would be nice to be able to toggle this from displaying time in units of beats to units of seconds/minutes. How should that toggling work?

### Recording Controls

How should recording work? Obviously there will be a "record button", but it needs to be easy and intuitive to select what you want to record, what source you want to record from, and what kind of behavior should be applied when loop-recording.

Also, how should selecting what track to record to work? The most obvious solution is to put a "record arm" button on the track header. If we go this route, should we warn the user with a pop-up if they try to record without having armed any tracks?

Some possible strategies:

- Pop-up a dialog box when selecting the record button that asks: "What would you like to record?" with options such as "audio only", "midi only", "audio and midi". And then a second set of options for looping behavior like "comping mode", "overwrite mode", etc. This is how FL Studio handles it.
- Have drop-downs and/or toggle switches next to the record button to select between different recording modes.
- Have separate buttons for different types of recording modes.

### Tempo Controls

What controls should there be to control tempo? I'm thinking for some of the more advance stuff like "groove", we will have a drop-down dialog box for those controls.

List of probable controls:

- A number display of the current tempo. Perhaps the user can double-click this to enter the tempo manually with the keyboard? I'm thinking the user should also be able to click and drag up-and-down with the mouse to increase/decrease the tempo that way too.
- A "tap" button for tapping in the tempo with mouse clicks.
- A number displaying the current time signature. Perhaps the user can double-click this to enter a time signature manually with the keyboard? Or maybe they can select from a drop-down of preset time signatures?
- Various controls for groove/swing (not mvp though).
- A button to toggle an automation lane of the project tempo? (not mvp though)
- A button to toggle a sort of "automation lane" for the project time signature? (not mvp though)

### File/Project Controls

We could probably have a traditional drop-down menu for some or all of these.

Also, should we allow having more than one project open at a time, with tabs to select between the separate projects? If so, what would that look like? If we do allow multiple projects open at the same time, we will also need some sort-of "activate audio engine" button to activate the audio engine of a particular project (like Bitwig does). I don't think it's feasible to allow multiple "audio engines" to run at the same time, especially since different projects may have different audio device configurations.

List of probable controls:

- Save
- Save As
- New Project
- Load Project
- Import File (audio or midi file)
- Export (opens the export dialog box)
- Undo/Redo buttons
- Settings (opens the settings dialog for the app)
- A button or menu item to open up the "command palette"

### Inline Meters

Should we even have these on the main toolbar? Would it be better to have these somewhere else, or even not have these at all?

List of probable meters:

- Master DB meter
- CPU utilization meter (should this just be a number, a bar, or even a graph?)
- Oscilloscope/Spectrogram (how should the user toggle between these modes?)

### View/Workspace controls

How should the user be able to open/toggle different "views" such as the timeline, mixer, browser, piano roll editor, audio clip editor, automation editor, clip launcher, track fx, etc?

Some possible strategies:

- Have a drop-down menu to toggle each dockable "view".
- Have a toggle button for each dockable "view" on the main toolbar (Like FL Studio does).
- Have tabs for different preset "workspaces" (like Blender does), and also let the user manually create their own workspaces. I personally think this could be an interesting route to take since I haven't seen any other DAW use this strategy. If we do use this strategy, then the obvious next question is what should the preset workspaces consist of?

# Track & Timeline workflow:

Here we need to answer a big question that would completely alter the design of the track/timeline section: Should we have a clip/pattern-based workflow like FL Studio, or should we have a linear track-based workflow like Ableton Live (and most other DAWs).

I'll list some pros and cons for each approach:

### Clip/Pattern-Based Workflow (like FL Studio)

Pros:
- It is a very "free-flow" way of working, allowing the user to place patterns/clips wherever they like.
- It is easy to select various patterns and copy-paste them wherever in the project.
- It gives the user more freedom on how they wish to organize (or not organize) their project.
- It it easier to "layer" instruments together so you can use a single midi clip to control all the layers for example.
- It it easier to route multiple audio clips into the same mixer track.
- Non-destructive loop recording is a no-brainer (just add the recording clip on the next available timeline row).
- Creating a clip-launching system will also be a no-brainer (since the timeline is already a sort of clip launcher).
- I (BillyDM) personally grew up using FL Studio, so I'm a bit more biased towards this workflow.

Cons:
- It is not as obvious and intuitive for users who are completely new to DAWs.
- It can be confusing for the user to keep track of all the patterns and clips in the project.
- It can obviously lead to very messy projects if the user doesn't take the time to organize.
- We would need some kind-of separate view that lists the patterns/clips themselves and the instruments/tracks they are assigned to. We don't have to do this *exactly* like FL does though. An idea I had is to have a vertical list of patterns and clips where the traditional "track headers" would be, but have this list be smartly "auto-organized" and sorted by tabs for the various types of clips (midi instrument patterns, audio clips, automation clips, etc). By "auto-organization" I mean grouping and vertically separating midi and automation clips by the track/instrument they are assigned to. I will probably draw up a mockup of what I mean soon.
- We would also need some way to distinguish when the user wanted to open the piano roll/automation editor, and when they want to open the instrument plugin that the clip is assigned to.

### Linear Track-based Workflow (like Ableton Live)

Pros:
- There is a clearer correlation between the midi/automation clips and the track/instrument they are assigned to.
- It is easier to copy-paste clips between different tracks.
- It is easier to group tracks/instruments together into "folders".
- The "track header" (the little panel that shows the track name, a fader to control volume, a db meter, arm/record/solo/mute buttons) is very standard and familiar.
- It is easier to organize projects.

Cons:
- It is much more restrictive on how the user is allowed to organize their project.
- Hiding/showing/adding automation lanes for a particular track is very finicky (possibly my biggest personal gripe with this type of workflow).
- Copying-pasting and looping clips is more finicky.
- Extra work would need to go into making the whole "grouping tracks into folders" feature.
- Creating "layers" of instruments (using a single midi clip to control all the layers) is more clunky and requires special "routing plugins" on the track fx.
- Non-destructive loop recording will be diffucult to solve (Do we add another tab to hide/show various takes on the track header? How does the user select which take takes precedent?)
- The clip-launcher system will need its own separate engine to work.

I'm personally leaning more towards the Clip/Pattern-based design, but of course I would love to hear discussion and arguments for other approaches.

# Track FX

This is another big question we need to answer. What should the FX panel look like? Should it be a traditional vertical rack, or should it be a more fancy horizontal rack with inline controls (like Ableton Live/Bitwig).

### Traditional Vertical FX Rack

Pros:
- Much easier to develop.
- The signal path is usually more clear to follow.
- Integration into the mixer view is a no-brainer, and will be much more compact.
- Since this will require designing plugin UIs to run in its own window, porting these internal plugins to external plugins is a no-brainer.
- Plugin UIs will not be constrained by the height of a "horizontal rack". Also having plugins in their own window could allow us to be more creative with the plugin designs.
- Opening two plugins side-by-side is also a no-brainer.
- Better UI performance.
- Having a separate "FL Studio Patcher"-like plugin for complex routing is easier to follow the signal path imo.

Cons:
- Definitely less fancy and "cool" than an inline horizontal rack.
- Editing different plugins requires opening the plugin windows each time (and in the case of "Patcher"-like plugins opening two windows).
- Complex routing will need to be done through a separate "FL Studio Patcher"-like plugin. Of course we can make this much more feature-rich with tighter integration than FL Studio's Patcher plugin.
- We wouldn't have any fancy inline marco and modifier system like Bitwig Studio has. (However, that doesn't mean we can't achieve all the same functionality in a "Patcher"-like plugin, it just won't be inline).

### Horizontal Inline FX Rack

Pros:
- Definitely more flashy and has more of a "wow" factor.
- Having all the controls inline means not having to constantly open plugin windows to edit an fx chain.
- We could have a fancy inline macro and modifier system similar to Bitwig Studio.

Cons:
- Integration into a mixer means having the horizontal FX rack underneath the mixer. As such it is more clunky to use than a "traditional" mixer, and also more difficult to have multiple plugins across different tracks open at the same time (possibly my biggest gripe with horizontal FX racks).
- Harder to develop.
- Worse UI performance (because of all the inline controls).
- The signal path can be less clear since plugins can be hidden behind the tabs of "grouping/splitting" FX (eg: Chain, Layer, Stereo Split, Multiband Split).
- More plugin UI design restrictions (vertical height limit, and also probably less freedom to be "creative" with the plugin UIs).
- It is unclear how this "fancy inline macro and modifier" system would work. I'm not sure if we can just copy Bitwig's design, and even if we did I personally find the system to be kind-of clunky at times.
- Minor gripe, but this design usually requires plugin titles oriented vertically which is hard to read.

Of course a solution I've been hearing is "why not both"? The answer is that it's unclear how it would even be possible to represent the horizontal "grouping/splitting" FX (eg: Chain, Layer, Stereo Split, Multiband Split) into a vertical mixer, and vice-versa representing the "Patcher"-like plugins in a horizontal FX rack. Or at least I can't think of a way it would be possible to do it cleanly. That, and it means we would need to create two separate UIs for each plugin, one for the traditional "plugin windows" for the vertical FX rack, and one for the horizontal inline plugins.

I'm personally leaning more towards the "traditional vertical FX rack" approach. Of course I would love to have discussion and hear the other sides of the argument. Keep in mind this all ties into what the "overall goal" of Meadowlark should be. Personally I see Meadowlark as more of a "flexible composing and mixing workhorse" rather than a "modular sound design environment" like Bitwig is.

# Piano Roll

I'm personally already quite satisfied with the current design of the piano roll editor. There are a few questions however:

- How should the user be able to select non-12-EDO scales? What will the actual "piano keys" part look like?
- How should the user be able to show/edit multiple clips at the same time? Reaper could be a good inspiration here.
- How should per-note automation work? Bitwig could be a good inspiration here.

# Mixer

Not much to be said here imo, mixers already have a pretty uniform design across DAWs. There are a few questions though:

- How should routing tracks and creating bus tracks work exactly? Drop-down menus?
- How resizable should the mixer tracks be, if at all?
- How should the user be able to see a number readout of a particular DB meter? Status bar?
- What should be the exact design of the mixer track controls? What should it all include? Probable controls include:
  - Fader
  - Pan
  - DB Meter
  - Solo & Mute buttons
  - A/B toggle switch (to quickly compare two separate FX chains, a very handy feature that I definitely want to include)
  - Stereo width control (maybe, we could also just have a stereo width plugin)

# Properties Panel

Not much to be said here since I think the design of this is quite trivial, just keep in mind this should exist somewhere. The idea is that whatever the user has selected some item in the project (a track, a clip, a midi note, automation node, a plugin, etc.), a "form"-style list of settings will show up in the properties panel. These settings will include everything that can be edited with the mouse in the UI, as well as any "advanced" settings that would not fit in with the rest of the UI.

# Browser

How should the user be able to browse various plugins, presets, audio clips, etc? Should we have a universal search function like Ableton Live has, or should we have separate tabs for each category of resource? Should this occupy the same space as the "Properties Panel".

An important feature I want to include is the ability to add "tags" to any resource, as well as being able to "favorite" any resource, much like what Bitwig and u-he's plugins let you do.

# Clip Launcher (not mvp)

Nothing to be said right now since this feature won't be added in mvp, but it should at-least be kept in mind.

I personally don't really use clip launchers, so I would like to hear what people who use this feature want out of a clip launcher.

# Audio Clip Editor (not mvp)

This will be a view that lets you finely edit an audio clip with offline effects. Not much to be said right now since this feature won't be added in mvp, but it should be at-least kept in mind.

# Automation Editor (not mvp)

This will probably just be a view that lets you finely edit automation clips. Not much to be said right now since this feature won't be added in mvp, but it should be at-least kept in mind.
