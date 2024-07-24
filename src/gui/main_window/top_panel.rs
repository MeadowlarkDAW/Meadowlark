use yarrow::prelude::*;

use crate::gui::icons::AppIcon;
use crate::gui::styling::AppStyle;

pub struct TopPanel {
    panel_bg: QuadElement,

    #[cfg(debug_assertions)]
    dev_mode_icon: Icon,

    project_section_label: Label,
    main_menu_btn: IconButton,
    project_seperator_1: Separator,
    import_btn: IconButton,
    save_btn: IconButton,
    save_as_btn: IconButton,
    project_seperator_2: Separator,
    undo_btn: IconButton,
    redo_btn: IconButton,

    section_seperator_1: Separator,

    view_section_label: Label,
    browser_panel_toggle_btn: IconToggleButton,
    tracks_panel_toggle_btn: IconToggleButton,
    clips_panel_toggle_btn: IconToggleButton,
    fx_rack_toggle_btn: IconToggleButton,
    piano_roll_toggle_btn: IconToggleButton,
    mixer_toggle_btn: IconToggleButton,
    // TODO:
    // audio_clip_editor_toggle_btn,
    // automation_editor_toggle_btn,
    // clip_launcher_toggle_btn,
    // timeline_toggle_btn,
    properties_panel_toggle_btn: IconToggleButton,
    command_palette_toggle_btn: IconToggleButton,

    section_seperator_2: Separator,

    transport_section_label: Label,
    skip_back_btn: IconButton,
    transport_seperator_1: Separator,
    play_pause_btn: IconToggleButton,
    stop_btn: IconButton,
    skip_forward_btn: IconButton,
    loop_toggle_btn: IconToggleButton,
    auto_return_toggle_btn: IconToggleButton,
    transport_seperator_2: Separator,
    transport_box: QuadElement,
    transport_menu_btn: IconButton,
    bpm_label: Label,
    time_signature_label: Label,
    bmp_numeric_input: TextInput,
    time_signature_numeric_input: TextInput,
    transport_box_seperator: Separator,
    mbs_label: Label,
    hmsm_label: Label,
    mbs_numeric_input: TextInput,
    hmsm_numeric_input: TextInput,

    section_seperator_3: Separator,

    record_section_label: Label,
    metronome_toggle_btn: IconToggleButton,
    record_seperator_1: Separator,
    record_btn: IconToggleButton,
    record_seperator_2: Separator,
    record_mic_toggle_btn: IconToggleButton,
    record_notes_toggle_btn: IconToggleButton,
    record_automation_toggle_btn: IconToggleButton,
    record_seperator_3: Separator,
    record_mode_dropdown_btn: IconLabelButton,

    section_seperator_4: Separator,
    monitor_section_label: Label,
    cpu_icon: Icon,

    prev_window_width: f32,
}

impl TopPanel {
    pub fn new(style: &AppStyle, cx: &mut WindowContext<'_, crate::AppAction>) -> Self {
        cx.reset_z_index();

        let panel_bg = QuadElement::builder(&style.top_panel_bg)
            .z_index(0)
            .build(cx);
        let transport_box = QuadElement::builder(&style.top_panel_transport_box)
            .z_index(1)
            .build(cx);

        let tooltip_align = Align2::BOTTOM_CENTER;

        cx.with_z_index(2, |cx| Self {
            #[cfg(debug_assertions)]
            dev_mode_icon: Icon::builder(&style.dev_icon).icon(AppIcon::Dev).build(cx),

            project_section_label: Label::builder(&style.top_panel_section_title_label)
                .text("PROJECT")
                .build(cx),
            main_menu_btn: IconButton::builder(&style.top_panel_icon_btn)
                .icon(AppIcon::Menu)
                .tooltip_message("Main Menu", tooltip_align)
                .build(cx),
            project_seperator_1: Separator::builder(&style.top_panel_seperator)
                .vertical(true)
                .build(cx),
            import_btn: IconButton::builder(&style.top_panel_icon_btn)
                .icon(AppIcon::Open)
                .tooltip_message("Import", tooltip_align)
                .build(cx),
            save_btn: IconButton::builder(&style.top_panel_icon_btn)
                .icon(AppIcon::Save)
                .tooltip_message("Save", tooltip_align)
                .build(cx),
            save_as_btn: IconButton::builder(&style.top_panel_icon_btn)
                .icon(AppIcon::SaveAs)
                .tooltip_message("Save As", tooltip_align)
                .build(cx),
            project_seperator_2: Separator::builder(&style.top_panel_seperator)
                .vertical(true)
                .build(cx),
            undo_btn: IconButton::builder(&style.top_panel_icon_btn)
                .icon(AppIcon::Undo)
                .tooltip_message("Undo", tooltip_align)
                .build(cx),
            redo_btn: IconButton::builder(&style.top_panel_icon_btn)
                .icon(AppIcon::Redo)
                .tooltip_message("Redo", tooltip_align)
                .build(cx),

            section_seperator_1: Separator::builder(&style.top_panel_seperator)
                .vertical(true)
                .build(cx),

            view_section_label: Label::builder(&style.top_panel_section_title_label)
                .text("VIEW")
                .build(cx),
            browser_panel_toggle_btn: IconToggleButton::builder(&style.top_panel_toggle_btn)
                .icon(AppIcon::Browser)
                .tooltip_message("Browser", tooltip_align)
                .build(cx),
            tracks_panel_toggle_btn: IconToggleButton::builder(&style.top_panel_toggle_btn)
                .icon(AppIcon::Tracks)
                .tooltip_message("Tracks", tooltip_align)
                .build(cx),
            clips_panel_toggle_btn: IconToggleButton::builder(&style.top_panel_toggle_btn)
                .icon(AppIcon::ClipsPanel)
                .tooltip_message("Clips", tooltip_align)
                .build(cx),
            fx_rack_toggle_btn: IconToggleButton::builder(&style.top_panel_toggle_btn)
                .icon(AppIcon::FXRack)
                .tooltip_message("FX Rack", tooltip_align)
                .build(cx),
            piano_roll_toggle_btn: IconToggleButton::builder(&style.top_panel_toggle_btn)
                .icon(AppIcon::PianoKeys)
                .tooltip_message("Piano Roll", tooltip_align)
                .build(cx),
            mixer_toggle_btn: IconToggleButton::builder(&style.top_panel_toggle_btn)
                .icon(AppIcon::Mixer)
                .tooltip_message("Mixer", tooltip_align)
                .build(cx),
            properties_panel_toggle_btn: IconToggleButton::builder(&style.top_panel_toggle_btn)
                .icon(AppIcon::Properties)
                .tooltip_message("Properties", tooltip_align)
                .build(cx),
            command_palette_toggle_btn: IconToggleButton::builder(&style.top_panel_toggle_btn)
                .icon(AppIcon::CommandPalette)
                .tooltip_message("Command Palette", tooltip_align)
                .build(cx),

            section_seperator_2: Separator::builder(&style.top_panel_seperator)
                .vertical(true)
                .build(cx),

            transport_section_label: Label::builder(&style.top_panel_section_title_label)
                .text("TRANSPORT")
                .build(cx),
            skip_back_btn: IconButton::builder(&style.top_panel_icon_btn)
                .icon(AppIcon::SkipBack)
                .tooltip_message("Skip Back", tooltip_align)
                .build(cx),
            play_pause_btn: IconToggleButton::builder(&style.top_panel_play_pause_btn)
                .dual_icons(AppIcon::Play, AppIcon::Pause)
                .tooltip_message("Play / Pause", tooltip_align)
                .build(cx),
            stop_btn: IconButton::builder(&style.top_panel_icon_btn)
                .icon(AppIcon::Stop)
                .tooltip_message("Stop", tooltip_align)
                .build(cx),
            skip_forward_btn: IconButton::builder(&style.top_panel_icon_btn)
                .icon(AppIcon::SkipForward)
                .tooltip_message("Skip Forward", tooltip_align)
                .build(cx),
            transport_seperator_1: Separator::builder(&style.top_panel_seperator)
                .vertical(true)
                .build(cx),
            loop_toggle_btn: IconToggleButton::builder(&style.top_panel_toggle_btn)
                .icon(AppIcon::Loop)
                .tooltip_message("Loop", tooltip_align)
                .build(cx),
            auto_return_toggle_btn: IconToggleButton::builder(&style.top_panel_toggle_btn)
                .icon(AppIcon::AutoReturn)
                .tooltip_message("Auto Return Playhead", tooltip_align)
                .build(cx),
            transport_seperator_2: Separator::builder(&style.top_panel_seperator)
                .vertical(true)
                .build(cx),
            transport_menu_btn: IconButton::builder(&style.top_panel_icon_btn)
                .icon(AppIcon::Menu)
                .tooltip_message("Transport Menu", tooltip_align)
                .z_index(2)
                .build(cx),
            bpm_label: Label::builder(&style.top_panel_label).text("bpm").build(cx),
            time_signature_label: Label::builder(&style.top_panel_label).text("sig").build(cx),
            bmp_numeric_input: TextInput::builder(&style.top_panel_numeric_text_input)
                .text("130.00")
                .tooltip_message("Beats per Minute", tooltip_align)
                .build(cx),
            time_signature_numeric_input: TextInput::builder(&style.top_panel_numeric_text_input)
                .text("4/4")
                .tooltip_message("Time Signature", tooltip_align)
                .build(cx),
            transport_box_seperator: Separator::builder(&style.top_panel_box_separator)
                .vertical(true)
                .build(cx),
            mbs_label: Label::builder(&style.top_panel_label)
                .text("m/b/s")
                .build(cx),
            hmsm_label: Label::builder(&style.top_panel_label)
                .text("h/m/s/m")
                .build(cx),
            mbs_numeric_input: TextInput::builder(&style.top_panel_numeric_text_input)
                .text("5.2.3")
                .tooltip_message("Measures / Bars / Beats", tooltip_align)
                .build(cx),
            hmsm_numeric_input: TextInput::builder(&style.top_panel_numeric_text_input)
                .text("00:00:12:202")
                .tooltip_message("Hours / Minutes / Seconds / Milliseconds", tooltip_align)
                .build(cx),

            section_seperator_3: Separator::builder(&style.top_panel_seperator)
                .vertical(true)
                .build(cx),

            record_section_label: Label::builder(&style.top_panel_section_title_label)
                .text("RECORD")
                .build(cx),
            metronome_toggle_btn: IconToggleButton::builder(&style.top_panel_toggle_btn)
                .icon(AppIcon::Metronome)
                .tooltip_message("Metronome", tooltip_align)
                .build(cx),
            record_seperator_1: Separator::builder(&style.top_panel_seperator)
                .vertical(true)
                .build(cx),
            record_btn: IconToggleButton::builder(&style.top_panel_record_btn)
                .icon(AppIcon::Record)
                .tooltip_message("Start Recording", tooltip_align)
                .build(cx),
            record_seperator_2: Separator::builder(&style.top_panel_seperator)
                .vertical(true)
                .build(cx),
            record_mic_toggle_btn: IconToggleButton::builder(&style.top_panel_toggle_btn)
                .icon(AppIcon::Mic)
                .tooltip_message("Record Microphone", tooltip_align)
                .build(cx),
            record_notes_toggle_btn: IconToggleButton::builder(&style.top_panel_toggle_btn)
                .icon(AppIcon::PianoKeys)
                .tooltip_message("Record Notes", tooltip_align)
                .build(cx),
            record_automation_toggle_btn: IconToggleButton::builder(&style.top_panel_toggle_btn)
                .icon(AppIcon::Automation)
                .tooltip_message("Record Automation", tooltip_align)
                .build(cx),
            record_seperator_3: Separator::builder(&style.top_panel_seperator)
                .vertical(true)
                .build(cx),
            record_mode_dropdown_btn: IconLabelButton::builder(&style.top_panel_dropdown_btn)
                .text(Some("Overwrite"))
                .icon(Some(AppIcon::DropdownArrow))
                .tooltip_message("Loop Recording Mode", tooltip_align)
                .build(cx),

            section_seperator_4: Separator::builder(&style.top_panel_seperator)
                .vertical(true)
                .build(cx),

            monitor_section_label: Label::builder(&style.top_panel_section_title_label)
                .text("MONITOR")
                .build(cx),
            cpu_icon: Icon::builder(&style.top_panel_icon)
                .icon(AppIcon::CPU)
                .build(cx),

            panel_bg,
            transport_box,

            prev_window_width: -1.0,
        })
    }

    pub fn layout(&mut self, window_size: Size, style: &AppStyle) {
        if window_size.width == self.prev_window_width {
            // No need to perform layout if window width is the same.
            return;
        }
        self.prev_window_width = window_size.width;

        let btn_y = style.top_panel_height - 4.0;

        let section_padding = 12.0;
        let element_spacing = 4.0;

        self.panel_bg
            .el
            .set_rect(rect(0.0, 0.0, window_size.width, style.top_panel_height));

        self.project_section_label
            .layout_aligned(point(0.0, 0.0), Align2::TOP_LEFT);
        self.main_menu_btn
            .layout_aligned(point(section_padding, btn_y), Align2::BOTTOM_LEFT);
        self.project_seperator_1.el.set_rect(rect(
            self.main_menu_btn.el.rect().max_x() + element_spacing,
            self.main_menu_btn.el.rect().min_y(),
            style.top_panel_seperator_width,
            self.main_menu_btn.el.rect().size.height,
        ));
        self.import_btn.layout_aligned(
            point(
                self.project_seperator_1.el.rect().max_x() + element_spacing,
                btn_y,
            ),
            Align2::BOTTOM_LEFT,
        );
        self.save_btn.layout_aligned(
            point(self.import_btn.el.rect().max_x() + element_spacing, btn_y),
            Align2::BOTTOM_LEFT,
        );
        self.save_as_btn.layout_aligned(
            point(self.save_btn.el.rect().max_x() + element_spacing, btn_y),
            Align2::BOTTOM_LEFT,
        );
        self.project_seperator_2.el.set_rect(rect(
            self.save_as_btn.el.rect().max_x() + element_spacing,
            self.save_btn.el.rect().min_y(),
            style.top_panel_seperator_width,
            self.save_btn.el.rect().size.height,
        ));
        self.undo_btn.layout_aligned(
            point(
                self.project_seperator_2.el.rect().max_x() + element_spacing,
                btn_y,
            ),
            Align2::BOTTOM_LEFT,
        );
        self.redo_btn.layout_aligned(
            point(self.undo_btn.el.rect().max_x() + element_spacing, btn_y),
            Align2::BOTTOM_LEFT,
        );

        self.section_seperator_1.el.set_rect(rect(
            self.redo_btn.el.rect().max_x() + section_padding,
            0.0,
            style.top_panel_section_seperator_width,
            style.top_panel_height,
        ));

        self.view_section_label.layout_aligned(
            point(self.section_seperator_1.el.rect().max_x(), 0.0),
            Align2::TOP_LEFT,
        );
        self.browser_panel_toggle_btn.layout_aligned(
            point(
                self.section_seperator_1.el.rect().max_x() + section_padding,
                btn_y,
            ),
            Align2::BOTTOM_LEFT,
        );
        self.tracks_panel_toggle_btn.layout_aligned(
            point(
                self.browser_panel_toggle_btn.el.rect().max_x() + element_spacing,
                btn_y,
            ),
            Align2::BOTTOM_LEFT,
        );
        self.clips_panel_toggle_btn.layout_aligned(
            point(
                self.tracks_panel_toggle_btn.el.rect().max_x() + element_spacing,
                btn_y,
            ),
            Align2::BOTTOM_LEFT,
        );
        self.fx_rack_toggle_btn.layout_aligned(
            point(
                self.clips_panel_toggle_btn.el.rect().max_x() + element_spacing,
                btn_y,
            ),
            Align2::BOTTOM_LEFT,
        );
        self.piano_roll_toggle_btn.layout_aligned(
            point(
                self.fx_rack_toggle_btn.el.rect().max_x() + element_spacing,
                btn_y,
            ),
            Align2::BOTTOM_LEFT,
        );
        self.mixer_toggle_btn.layout_aligned(
            point(
                self.piano_roll_toggle_btn.el.rect().max_x() + element_spacing,
                btn_y,
            ),
            Align2::BOTTOM_LEFT,
        );
        self.properties_panel_toggle_btn.layout_aligned(
            point(
                self.mixer_toggle_btn.el.rect().max_x() + element_spacing,
                btn_y,
            ),
            Align2::BOTTOM_LEFT,
        );
        self.command_palette_toggle_btn.layout_aligned(
            point(
                self.properties_panel_toggle_btn.el.rect().max_x() + element_spacing,
                btn_y,
            ),
            Align2::BOTTOM_LEFT,
        );

        self.section_seperator_2.el.set_rect(rect(
            self.command_palette_toggle_btn.el.rect().max_x() + section_padding,
            0.0,
            style.top_panel_section_seperator_width,
            style.top_panel_height,
        ));

        self.transport_section_label.layout_aligned(
            point(self.section_seperator_2.el.rect().max_x(), 0.0),
            Align2::TOP_LEFT,
        );
        self.skip_back_btn.layout_aligned(
            point(
                self.section_seperator_2.el.rect().max_x() + section_padding,
                btn_y,
            ),
            Align2::BOTTOM_LEFT,
        );
        self.play_pause_btn.layout_aligned(
            point(
                self.skip_back_btn.el.rect().max_x() + element_spacing,
                btn_y,
            ),
            Align2::BOTTOM_LEFT,
        );
        self.stop_btn.layout_aligned(
            point(
                self.play_pause_btn.el.rect().max_x() + element_spacing,
                btn_y,
            ),
            Align2::BOTTOM_LEFT,
        );
        self.skip_forward_btn.layout_aligned(
            point(self.stop_btn.el.rect().max_x() + element_spacing, btn_y),
            Align2::BOTTOM_LEFT,
        );
        self.transport_seperator_1.el.set_rect(rect(
            self.skip_forward_btn.el.rect().max_x() + element_spacing,
            self.main_menu_btn.el.rect().min_y(),
            style.top_panel_seperator_width,
            self.main_menu_btn.el.rect().size.height,
        ));
        self.loop_toggle_btn.layout_aligned(
            point(
                self.transport_seperator_1.el.rect().max_x() + element_spacing,
                btn_y,
            ),
            Align2::BOTTOM_LEFT,
        );
        self.auto_return_toggle_btn.layout_aligned(
            point(
                self.loop_toggle_btn.el.rect().max_x() + element_spacing,
                btn_y,
            ),
            Align2::BOTTOM_LEFT,
        );
        self.transport_seperator_2.el.set_rect(rect(
            self.auto_return_toggle_btn.el.rect().max_x() + element_spacing,
            self.main_menu_btn.el.rect().min_y(),
            style.top_panel_seperator_width,
            self.main_menu_btn.el.rect().size.height,
        ));
        self.transport_box.el.set_rect(rect(
            self.transport_seperator_2.el.rect().max_x() + section_padding,
            4.0,
            310.0,
            style.top_panel_height - (4.0 * 2.0),
        ));
        self.transport_menu_btn.layout_aligned(
            point(
                self.transport_box.el.rect().min_x() + element_spacing,
                style.top_panel_height * 0.5,
            ),
            Align2::CENTER_LEFT,
        );
        self.bpm_label.layout_aligned(
            point(
                self.transport_menu_btn.el.rect().max_x() + element_spacing,
                style.top_panel_height * 0.33,
            ),
            Align2::CENTER_LEFT,
        );
        self.time_signature_label.layout_aligned(
            point(
                self.transport_menu_btn.el.rect().max_x() + element_spacing,
                style.top_panel_height * 0.67,
            ),
            Align2::CENTER_LEFT,
        );
        self.time_signature_label
            .el
            .set_width(self.bpm_label.el.rect().width());
        self.bmp_numeric_input.el.set_rect(rect(
            self.bpm_label.el.rect().max_x() + element_spacing,
            4.0,
            60.0,
            24.0,
        ));
        self.time_signature_numeric_input.el.set_rect(rect(
            self.time_signature_label.el.rect().max_x() + element_spacing,
            21.0,
            60.0,
            24.0,
        ));
        self.transport_box_seperator.el.set_rect(rect(
            self.bmp_numeric_input.el.rect().max_x() + element_spacing,
            self.transport_box.el.rect().min_y(),
            style.top_panel_seperator_width,
            self.transport_box.el.rect().height(),
        ));
        self.mbs_label.layout_aligned(
            point(
                self.transport_box_seperator.el.rect().max_x() + element_spacing,
                style.top_panel_height * 0.33,
            ),
            Align2::CENTER_LEFT,
        );
        self.hmsm_label.layout_aligned(
            point(
                self.transport_box_seperator.el.rect().max_x() + element_spacing,
                style.top_panel_height * 0.67,
            ),
            Align2::CENTER_LEFT,
        );
        self.mbs_label
            .el
            .set_width(self.hmsm_label.el.rect().width());
        self.mbs_numeric_input.el.set_rect(rect(
            self.hmsm_label.el.rect().max_x() + element_spacing,
            4.0,
            100.0,
            24.0,
        ));
        self.hmsm_numeric_input.el.set_rect(rect(
            self.hmsm_label.el.rect().max_x() + element_spacing,
            21.0,
            100.0,
            24.0,
        ));
        self.transport_box.el.set_width(
            self.hmsm_numeric_input.el.rect().max_x() + element_spacing
                - self.transport_box.el.rect().min_x(),
        );

        self.section_seperator_3.el.set_rect(rect(
            self.transport_box.el.rect().max_x() + section_padding,
            0.0,
            style.top_panel_section_seperator_width,
            style.top_panel_height,
        ));

        self.record_section_label.layout_aligned(
            point(self.section_seperator_3.el.rect().max_x(), 0.0),
            Align2::TOP_LEFT,
        );
        self.metronome_toggle_btn.layout_aligned(
            point(
                self.section_seperator_3.el.rect().max_x() + section_padding,
                btn_y,
            ),
            Align2::BOTTOM_LEFT,
        );
        self.record_seperator_1.el.set_rect(rect(
            self.metronome_toggle_btn.el.rect().max_x() + element_spacing,
            self.main_menu_btn.el.rect().min_y(),
            style.top_panel_seperator_width,
            self.main_menu_btn.el.rect().size.height,
        ));
        self.record_btn.layout_aligned(
            point(
                self.record_seperator_1.el.rect().max_x() + element_spacing,
                btn_y,
            ),
            Align2::BOTTOM_LEFT,
        );
        self.record_seperator_2.el.set_rect(rect(
            self.record_btn.el.rect().max_x() + element_spacing,
            self.main_menu_btn.el.rect().min_y(),
            style.top_panel_seperator_width,
            self.main_menu_btn.el.rect().size.height,
        ));
        self.record_mic_toggle_btn.layout_aligned(
            point(
                self.record_seperator_2.el.rect().max_x() + element_spacing,
                btn_y,
            ),
            Align2::BOTTOM_LEFT,
        );
        self.record_notes_toggle_btn.layout_aligned(
            point(
                self.record_mic_toggle_btn.el.rect().max_x() + element_spacing,
                btn_y,
            ),
            Align2::BOTTOM_LEFT,
        );
        self.record_automation_toggle_btn.layout_aligned(
            point(
                self.record_notes_toggle_btn.el.rect().max_x() + element_spacing,
                btn_y,
            ),
            Align2::BOTTOM_LEFT,
        );
        self.record_seperator_3.el.set_rect(rect(
            self.record_automation_toggle_btn.el.rect().max_x() + element_spacing,
            self.main_menu_btn.el.rect().min_y(),
            style.top_panel_seperator_width,
            self.main_menu_btn.el.rect().size.height,
        ));
        self.record_mode_dropdown_btn.layout_aligned(
            point(
                self.record_seperator_3.el.rect().max_x() + element_spacing,
                btn_y,
            ),
            Align2::BOTTOM_LEFT,
        );

        self.section_seperator_4.el.set_rect(rect(
            self.record_mode_dropdown_btn.el.rect().max_x() + section_padding,
            0.0,
            style.top_panel_section_seperator_width,
            style.top_panel_height,
        ));

        self.monitor_section_label.layout_aligned(
            point(self.section_seperator_4.el.rect().max_x(), 0.0),
            Align2::TOP_LEFT,
        );
        self.cpu_icon.layout_aligned(
            point(
                self.section_seperator_4.el.rect().max_x() + section_padding,
                btn_y,
            ),
            Align2::BOTTOM_LEFT,
        );

        // TODO: Spread out sections to fill window width.

        let end_x = self.monitor_section_label.el.rect().max_x() + section_padding;

        if window_size.width < end_x {
            #[cfg(debug_assertions)]
            self.dev_mode_icon.el.set_hidden(true);
            return;
        } else {
            #[cfg(debug_assertions)]
            {
                self.dev_mode_icon.el.set_hidden(false);
                self.dev_mode_icon.layout_aligned(
                    point(window_size.width - element_spacing, btn_y),
                    Align2::BOTTOM_RIGHT,
                );
            }
        }

        let offset = point((window_size.width - end_x) * 0.5, 0.0);

        self.project_section_label.el.offset_pos(offset);
        self.main_menu_btn.el.offset_pos(offset);
        self.project_seperator_1.el.offset_pos(offset);
        self.import_btn.el.offset_pos(offset);
        self.save_btn.el.offset_pos(offset);
        self.save_as_btn.el.offset_pos(offset);
        self.project_seperator_2.el.offset_pos(offset);
        self.undo_btn.el.offset_pos(offset);
        self.redo_btn.el.offset_pos(offset);

        self.section_seperator_1.el.offset_pos(offset);

        self.view_section_label.el.offset_pos(offset);
        self.browser_panel_toggle_btn.el.offset_pos(offset);
        self.tracks_panel_toggle_btn.el.offset_pos(offset);
        self.clips_panel_toggle_btn.el.offset_pos(offset);
        self.fx_rack_toggle_btn.el.offset_pos(offset);
        self.piano_roll_toggle_btn.el.offset_pos(offset);
        self.mixer_toggle_btn.el.offset_pos(offset);
        self.properties_panel_toggle_btn.el.offset_pos(offset);
        self.command_palette_toggle_btn.el.offset_pos(offset);

        self.section_seperator_2.el.offset_pos(offset);

        self.transport_section_label.el.offset_pos(offset);
        self.skip_back_btn.el.offset_pos(offset);
        self.transport_seperator_1.el.offset_pos(offset);
        self.play_pause_btn.el.offset_pos(offset);
        self.stop_btn.el.offset_pos(offset);
        self.skip_forward_btn.el.offset_pos(offset);
        self.loop_toggle_btn.el.offset_pos(offset);
        self.auto_return_toggle_btn.el.offset_pos(offset);
        self.transport_seperator_2.el.offset_pos(offset);
        self.transport_box.el.offset_pos(offset);
        self.transport_menu_btn.el.offset_pos(offset);
        self.bpm_label.el.offset_pos(offset);
        self.time_signature_label.el.offset_pos(offset);
        self.bmp_numeric_input.el.offset_pos(offset);
        self.time_signature_numeric_input.el.offset_pos(offset);
        self.transport_box_seperator.el.offset_pos(offset);
        self.mbs_label.el.offset_pos(offset);
        self.hmsm_label.el.offset_pos(offset);
        self.mbs_numeric_input.el.offset_pos(offset);
        self.hmsm_numeric_input.el.offset_pos(offset);

        self.section_seperator_3.el.offset_pos(offset);

        self.record_section_label.el.offset_pos(offset);
        self.metronome_toggle_btn.el.offset_pos(offset);
        self.record_seperator_1.el.offset_pos(offset);
        self.record_btn.el.offset_pos(offset);
        self.record_seperator_2.el.offset_pos(offset);
        self.record_mic_toggle_btn.el.offset_pos(offset);
        self.record_notes_toggle_btn.el.offset_pos(offset);
        self.record_automation_toggle_btn.el.offset_pos(offset);
        self.record_seperator_3.el.offset_pos(offset);
        self.record_mode_dropdown_btn.el.offset_pos(offset);

        self.section_seperator_4.el.offset_pos(offset);
        self.monitor_section_label.el.offset_pos(offset);
        self.cpu_icon.el.offset_pos(offset);
    }
}
