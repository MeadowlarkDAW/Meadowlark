use std::rc::Rc;

use rusty_daw_core::{pcm::AnyPCM, MusicalTime};
use vizia::*;

use femtovg::{Align, Baseline, Paint, Path};

use crate::{
    backend::AnyPcm,
    state::{
        ui_state::{TimelineSelectionEvent, TimelineSelectionUiState},
        AppEvent, StateSystem,
    },
};

use super::timeline_view::TimelineViewState;

use audio_waveform_mipmap::{Waveform, WaveformConfig};

pub struct Clip {
    track_id: usize,
    clip_id: usize,
    clip_start: MusicalTime,
    clip_end: MusicalTime,
}

impl Clip {
    pub fn new(
        cx: &mut Context,
        track_id: usize,
        clip_id: usize,
        clip_name: String,
        clip_start: MusicalTime,
        clip_end: MusicalTime,
    ) -> Handle<Self> {
        Self { track_id, clip_id, clip_start, clip_end }
            .build2(cx, move |cx| {
                cx.focused = cx.current;

                if cx.data::<ClipData>().is_none() {
                    let mut waveform = Waveform::default();

                    if let Some(state_system) = cx.data::<StateSystem>() {
                        if let Some(project) = state_system.get_project() {
                            if let Some((_, track)) = project.timeline_track_handles.get(track_id) {
                                let (_, timeline_tracks_state) =
                                    project.save_state.timeline_tracks();
                                if let Some((clip, _)) = track.audio_clip(
                                    clip_id,
                                    timeline_tracks_state.get(track_id).unwrap(),
                                ) {
                                    let data = clip.resource();
                                    match &*data.pcm {
                                        AnyPcm::Mono(audio_data) => {
                                            let samples = audio_data.data();
                                            //println!("samples: {}", samples.len());
                                            waveform =
                                                Waveform::new(samples, WaveformConfig::default());
                                        }

                                        AnyPcm::Stereo(audio_data) => {
                                            let samples = audio_data.left();
                                            //println!("samples: {}", samples.len());
                                            waveform =
                                                Waveform::new(samples, WaveformConfig::default());
                                        }

                                        _ => {}
                                    }
                                }
                            }
                        }
                    }

                    // Create some internal slider data (not exposed to the user)
                    ClipData {
                        dragging: false,
                        resize_start: false,
                        resize_end: false,
                        start_time: clip_start,
                        end_time: clip_end,
                        waveform: Rc::new(waveform),
                        should_snap: true,
                    }
                    .build(cx);
                }

                Label::new(cx, &clip_name)
                    .height(Pixels(20.0))
                    .width(Stretch(1.0))
                    .background_color(Color::rgb(251, 144, 96))
                    .class("clip_header")
                    .on_press(move |cx| {
                        cx.emit(ClipEvent::SetDragging(true));
                        cx.emit(TimelineSelectionEvent::SetSelection(
                            track_id, track_id, clip_start, clip_end,
                        ));
                        cx.emit(AppEvent::SeekTo(clip_start));
                    });
                Element::new(cx).background_color(Color::rgba(235, 136, 92, 15));
            })
            .position_type(PositionType::SelfDirected)
    }
}

impl View for Clip {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        if let Some(window_event) = event.message.downcast() {
            match window_event {
                WindowEvent::MouseMove(x, _) => {
                    if let Some(clip_data) = cx.data::<ClipData>() {
                        let start_time = clip_data.start_time;
                        let end_time = clip_data.end_time;
                        let resize_start = clip_data.resize_start;
                        let dragging = clip_data.dragging;
                        let resize_end = clip_data.resize_end;

                        let local_mouse_pos = *x - cx.cache.get_posx(cx.current);

                        if dragging {
                            cx.captured = cx.current;
                            cx.emit(WindowEvent::SetCursor(CursorIcon::Grabbing));
                        } else if resize_start || resize_end {
                            cx.emit(WindowEvent::SetCursor(CursorIcon::EwResize));
                        } else {
                            if local_mouse_pos >= 0.0 && local_mouse_pos <= 5.0
                                || local_mouse_pos >= cx.cache.get_width(cx.current) - 5.0
                                    && local_mouse_pos <= cx.cache.get_width(cx.current)
                            {
                                cx.emit(WindowEvent::SetCursor(CursorIcon::EwResize));
                            } else {
                                //cx.emit(WindowEvent::SetCursor(CursorIcon::Default));
                                let cursor =
                                    cx.style.cursor.get(cx.hovered).cloned().unwrap_or_default();
                                cx.emit(WindowEvent::SetCursor(cursor));
                            }
                        }

                        if let Some(timeline_view_state) = cx.data::<TimelineViewState>() {
                            let timeline_start = timeline_view_state.timeline_start;
                            let timeline_end = timeline_view_state.timeline_end;

                            let mut musical_delta =
                                timeline_view_state.delta_to_musical(*x - cx.mouse.left.pos_down.0);

                            let mut musical_pos = timeline_view_state.cursor_to_musical(*x);

                            let pixels_per_beat = timeline_view_state.width
                                / (timeline_view_state
                                    .end_time
                                    .checked_sub(timeline_view_state.start_time)
                                    .unwrap())
                                .as_beats_f64() as f32;

                            // Snapping
                            /*
                            if cx.data::<ClipData>().unwrap().should_snap {
                                if pixels_per_beat >= 100.0 && pixels_per_beat < 400.0 {
                                    musical_delta =
                                        MusicalTime::new((musical_delta.0 * 4.0).round() / 4.0);
                                    musical_pos =
                                        MusicalTime::new((musical_pos.0 * 4.0).round() / 4.0);
                                } else if pixels_per_beat >= 400.0 {
                                    musical_delta =
                                        MusicalTime::new((musical_delta.0 * 16.0).round() / 16.0);
                                    musical_pos =
                                        MusicalTime::new((musical_pos.0 * 16.0).round() / 16.0);
                                } else {
                                    musical_delta = MusicalTime::new(musical_delta.0.round());
                                    musical_pos = MusicalTime::new(musical_pos.0.round());
                                }
                            }
                            */

                            if dragging {
                                if start_time + musical_delta <= timeline_start {
                                    cx.emit(AppEvent::SetClipStart(
                                        self.track_id,
                                        self.clip_id,
                                        timeline_start,
                                    ));

                                    cx.emit(TimelineSelectionEvent::SetSelection(
                                        self.track_id,
                                        self.track_id,
                                        timeline_start,
                                        timeline_start
                                            + (self.clip_end.checked_sub(self.clip_start).unwrap()),
                                    ));
                                }
                                // else if start_time.0 + self.clip_end.0 + musical_delta.0 >= timeline_end.0 {
                                //     println!("start_time: {:?}, clip_end: {:?}, musical_dela: {:?}, timeline_end: {:?}", start_time, self.clip_end, musical_delta, timeline_end);
                                //     //cx.emit(TimelineViewEvent::SetEndTime(self.timeline_end));
                                //     //cx.emit(TimelineViewEvent::SetStartTime(self.timeline_end - (self.end_time - self.start_time)));

                                //     cx.emit(AppEvent::SetClipStart(
                                //         self.track_id,
                                //         self.clip_id,
                                //         timeline_end  - (self.clip_end - self.clip_start),
                                //     ));

                                //     cx.emit(TimelineSelectionEvent::SetSelection(
                                //         self.track_id,
                                //         self.track_id,
                                //         timeline_end  - (self.clip_end - self.clip_start),
                                //         timeline_end,
                                //     ));
                                // }
                                else {
                                    //cx.emit(TimelineViewEvent::SetStartTime(self.start_time + musical));
                                    //cx.emit(TimelineViewEvent::SetEndTime(self.end_time + musical));

                                    cx.emit(AppEvent::SetClipStart(
                                        self.track_id,
                                        self.clip_id,
                                        start_time + musical_delta,
                                    ));

                                    cx.emit(TimelineSelectionEvent::SetSelection(
                                        self.track_id,
                                        self.track_id,
                                        start_time + musical_delta,
                                        end_time + musical_delta,
                                    ));
                                }
                            }

                            if resize_end {
                                cx.emit(AppEvent::TrimClipEnd(
                                    self.track_id,
                                    self.clip_id,
                                    musical_pos,
                                ));
                                cx.emit(TimelineSelectionEvent::SetSelection(
                                    self.track_id,
                                    self.track_id,
                                    self.clip_start,
                                    musical_pos,
                                ));
                            }

                            if resize_start {
                                cx.emit(AppEvent::TrimClipStart(
                                    self.track_id,
                                    self.clip_id,
                                    musical_pos,
                                ));
                                cx.emit(AppEvent::SetClipEnd(
                                    self.track_id,
                                    self.clip_id,
                                    self.clip_end,
                                ));
                                cx.emit(TimelineSelectionEvent::SetSelection(
                                    self.track_id,
                                    self.track_id,
                                    musical_pos,
                                    self.clip_end,
                                ));
                            }
                        }
                    }
                }

                WindowEvent::MouseUp(button) if *button == MouseButton::Left => {
                    cx.emit(ClipEvent::SetDragging(false));
                    cx.emit(ClipEvent::SetResizeStart(false));
                    cx.emit(ClipEvent::SetResizeEnd(false));
                    //self.clip_start = cx.data::<ClipData>().unwrap().start_time;
                    cx.emit(ClipEvent::SetStartTime(self.clip_start));
                    cx.emit(ClipEvent::SetEndTime(self.clip_end));
                    //cx.captured = Entity::null();
                    //let cursor =
                    //cx.style.borrow().cursor.get(cx.hovered).cloned().unwrap_or_default();
                    //cx.emit(WindowEvent::SetCursor(cursor));

                    if event.target == cx.current {
                        cx.captured = Entity::null();
                        if cx.hovered != cx.current {
                            //cx.emit(WindowEvent::SetCursor(CursorIcon::Default));
                            let cursor =
                                cx.style.cursor.get(cx.hovered).cloned().unwrap_or_default();
                            cx.emit(WindowEvent::SetCursor(cursor));
                        }
                    }
                }

                WindowEvent::MouseDown(button) if *button == MouseButton::Left => {
                    cx.focused = cx.current;

                    let local_click_pos = cx.mouse.left.pos_down.0 - cx.cache.get_posx(cx.current);
                    if local_click_pos >= 0.0 && local_click_pos <= 5.0 {
                        cx.emit(ClipEvent::SetResizeStart(true));
                        cx.emit(ClipEvent::SetDragging(false));
                        cx.emit(TimelineSelectionEvent::SetSelection(
                            self.track_id,
                            self.track_id,
                            self.clip_start,
                            self.clip_end,
                        ));
                    }

                    if local_click_pos >= cx.cache.get_width(cx.current) - 5.0
                        && local_click_pos <= cx.cache.get_width(cx.current)
                    {
                        cx.emit(ClipEvent::SetResizeEnd(true));
                        cx.emit(ClipEvent::SetDragging(false));
                        cx.emit(TimelineSelectionEvent::SetSelection(
                            self.track_id,
                            self.track_id,
                            self.clip_start,
                            self.clip_end,
                        ));
                    }

                    cx.captured = cx.current;
                }

                // TEMPORARY - Need to move this to a keymap that wraps the timeline
                WindowEvent::KeyDown(code, _) => match code {
                    Code::KeyD => {
                        if cx.modifiers.contains(Modifiers::CTRL) {
                            //println!("Duplicate");
                            if let Some(timeline_selection) =
                                cx.data::<TimelineSelectionUiState>().cloned()
                            {
                                cx.emit(AppEvent::DuplicateSelection(
                                    timeline_selection.track_start,
                                    timeline_selection.select_start,
                                    timeline_selection.select_end,
                                ));

                                cx.emit(TimelineSelectionEvent::SetSelection(
                                    timeline_selection.track_start,
                                    timeline_selection.track_start,
                                    timeline_selection.select_end,
                                    timeline_selection.select_end
                                        + (timeline_selection
                                            .select_end
                                            .checked_sub(timeline_selection.select_start)
                                            .unwrap()),
                                ));
                            }
                        }
                    }

                    // TODO
                    Code::Delete => {
                        if let Some(timeline_selection) = cx.data::<TimelineSelectionUiState>() {
                            cx.emit(AppEvent::RemoveSelection(
                                timeline_selection.track_start,
                                timeline_selection.select_start,
                                timeline_selection.select_end,
                            ));

                            cx.emit(TimelineSelectionEvent::SelectNone);
                        }
                    }

                    Code::Escape => {
                        cx.emit(TimelineSelectionEvent::SelectNone);
                    }

                    Code::ControlLeft | Code::ControlRight => {
                        cx.emit(ClipEvent::Snap(false));
                    }

                    _ => {}
                },

                WindowEvent::KeyUp(code, _) => match code {
                    Code::ControlLeft => {
                        cx.emit(ClipEvent::Snap(true));
                    }

                    _ => {}
                },

                _ => {}
            }
        }
    }

    // Custom drawing for clip waveforms
    fn draw(&self, cx: &mut Context, canvas: &mut Canvas) {
        let bounds = cx.cache.get_bounds(cx.current);
        let header_height = 20.0;

        let clipy = bounds.y + header_height;
        let cliph = bounds.h - header_height;

        //canvas.save();

        //canvas.scissor(bounds.x, bounds.y, bounds.w, bounds.h);

        let background_color =
            cx.style.background_color.get(cx.current).cloned().unwrap_or_default();

        let opacity = cx.cache.get_opacity(cx.current);

        let mut background_color: femtovg::Color = background_color.into();
        background_color.set_alphaf(background_color.a * opacity);

        let mut path = Path::new();
        path.rect(bounds.x, bounds.y, bounds.w, bounds.h);
        canvas.fill_path(&mut path, Paint::color(background_color));

        if let Some(state_system) = cx.data::<StateSystem>() {
            if let Some(project) = state_system.get_project() {
                if let Some((_, track)) = project.timeline_track_handles.get(self.track_id) {
                    let (tempo_map, timeline_tracks_state) = project.save_state.timeline_tracks();
                    if let Some((clip, clip_state)) = track
                        .audio_clip(self.clip_id, timeline_tracks_state.get(self.track_id).unwrap())
                    {
                        /*
                        let (duration_time, duration_frac) =
                            clip_state.duration.to_sub_sample(tempo_map.sample_rate);
                        let sample_duration = duration_time.0 as f64 + duration_frac;

                        println!(
                            "Sample Duration: {} {:?} {}",
                            sample_duration, clip_state.duration, bounds.w
                        );

                        let (offset_time, offset_frac) =
                            clip_state.clip_start_offset.to_sub_sample(tempo_map.sample_rate);

                        let sample_offset = offset_time.0 as f64 + offset_frac;
                        */

                        let sample_offset = clip_state
                            .clip_start_offset
                            .to_nearest_frame_round(tempo_map.sample_rate)
                            .0 as f64;
                        let sample_duration =
                            clip_state.duration.to_nearest_frame_round(tempo_map.sample_rate).0
                                as f64;

                        let clip_resource = clip.resource();

                        //println!("{} {}", clip_resource.original_offset.0 as f64, sample_duration.0 as f64);
                        match &*clip_resource.pcm {
                            AnyPcm::Mono(audio_data) => {
                                let samples = audio_data.data();
                                if let Some(clip_data) = cx.data::<ClipData>() {
                                    for (x, min, max) in clip_data.waveform.query(
                                        samples,
                                        0.0,
                                        18000.0,
                                        bounds.w as usize,
                                    ) {
                                        //println!("x {} min: {} max: {}", x, min, max);
                                    }
                                }
                            }

                            AnyPcm::Stereo(audio_data) => {
                                //println!("Draw Clip: {} {} {}", cx.current, self.track_id, self.clip_id);
                                let samples = audio_data.left();
                                if let Some(clip_data) = cx.data::<ClipData>() {
                                    let mut path = Path::new();
                                    path.move_to(bounds.x, clipy + cliph / 2.0);
                                    for (x, min, max) in clip_data.waveform.query(
                                        samples,
                                        clip_resource.original_offset.0 as f64 + sample_offset,
                                        sample_duration as f64,
                                        bounds.w as usize,
                                    ) {
                                        //println!("x {} min: {} max: {}", x, min, max);
                                        path.line_to(
                                            bounds.x + x,
                                            clipy + cliph / 2.0 - min * cliph / 2.0,
                                        );
                                        path.line_to(
                                            bounds.x + x,
                                            clipy + cliph / 2.0 - max * cliph / 2.0,
                                        );
                                    }
                                    let mut paint =
                                        Paint::color(femtovg::Color::rgba(254, 217, 200, 255));
                                    paint.set_line_width(1.0);
                                    paint.set_anti_alias(false);
                                    canvas.stroke_path(&mut path, paint);
                                }
                            }

                            _ => {}
                        }
                    }
                }
            }
        }

        //canvas.restore();
    }
}

#[derive(Clone, Data, Lens)]
pub struct ClipData {
    dragging: bool,
    resize_start: bool,
    resize_end: bool,
    // Start time when the clip is pressed
    start_time: MusicalTime,

    end_time: MusicalTime,
    #[data(ignore)]
    waveform: Rc<Waveform>,

    #[data(ignore)]
    should_snap: bool,
}

#[derive(Debug)]
pub enum ClipEvent {
    SetDragging(bool),
    SetResizeStart(bool),
    SetResizeEnd(bool),
    SetStartTime(MusicalTime),
    SetEndTime(MusicalTime),
    Snap(bool),
}

impl Model for ClipData {
    fn event(&mut self, _: &mut Context, event: &mut Event) {
        if let Some(clip_event) = event.message.downcast() {
            match clip_event {
                ClipEvent::SetDragging(val) => {
                    self.dragging = *val;
                }

                ClipEvent::SetResizeStart(val) => {
                    self.resize_start = *val;
                }

                ClipEvent::SetResizeEnd(val) => {
                    self.resize_end = *val;
                }

                ClipEvent::SetStartTime(start_time) => {
                    self.start_time = *start_time;
                }

                ClipEvent::SetEndTime(end_time) => {
                    self.end_time = *end_time;
                }

                ClipEvent::Snap(flag) => {
                    self.should_snap = *flag;
                }
            }
        }
    }
}
