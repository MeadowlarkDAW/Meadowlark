use yarrow::prelude::*;

use crate::gui::icons::AppIcon;
use crate::gui::styling::AppStyle;

pub struct TopPanel {
    panel_bg: QuadElement,

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
    //record_source_label: Label,
    record_mic_toggle_btn: IconToggleButton,
    record_notes_toggle_btn: IconToggleButton,
    record_automation_toggle_btn: IconToggleButton,
    record_seperator_3: Separator,
    //record_mode_label: Label,
    record_mode_dropdown_btn: IconLabelButton,

    section_seperator_4: Separator,
    monitor_section_label: Label,
    cpu_icon: Icon,
}

impl TopPanel {
    pub fn new(style: &AppStyle, cx: &mut WindowContext<'_, crate::Action>) -> Self {
        Self {
            panel_bg: QuadElement::builder(&style.top_panel_bg).build(cx),

            project_section_label: Label::builder(&style.top_panel_section_title_label)
                .text("PROJECT")
                .z_index(1)
                .build(cx),
            main_menu_btn: IconButton::builder(&style.top_panel_icon_btn)
                .icon(AppIcon::Menu)
                .z_index(1)
                .build(cx),
            project_seperator_1: Separator::builder(&style.top_panel_seperator)
                .z_index(1)
                .vertical(true)
                .build(cx),
            import_btn: IconButton::builder(&style.top_panel_icon_btn)
                .icon(AppIcon::Open)
                .z_index(1)
                .build(cx),
            save_btn: IconButton::builder(&style.top_panel_icon_btn)
                .icon(AppIcon::Save)
                .z_index(1)
                .build(cx),
            save_as_btn: IconButton::builder(&style.top_panel_icon_btn)
                .icon(AppIcon::SaveAs)
                .z_index(1)
                .build(cx),
            project_seperator_2: Separator::builder(&style.top_panel_seperator)
                .z_index(1)
                .vertical(true)
                .build(cx),
            undo_btn: IconButton::builder(&style.top_panel_icon_btn)
                .icon(AppIcon::Undo)
                .z_index(1)
                .build(cx),
            redo_btn: IconButton::builder(&style.top_panel_icon_btn)
                .icon(AppIcon::Redo)
                .z_index(1)
                .build(cx),

            section_seperator_1: Separator::builder(&style.top_panel_seperator)
                .z_index(1)
                .vertical(true)
                .build(cx),

            view_section_label: Label::builder(&style.top_panel_section_title_label)
                .text("VIEW")
                .z_index(1)
                .build(cx),
            browser_panel_toggle_btn: IconToggleButton::builder(&style.top_panel_toggle_btn)
                .icon(AppIcon::Browser)
                .z_index(1)
                .build(cx),
            tracks_panel_toggle_btn: IconToggleButton::builder(&style.top_panel_toggle_btn)
                .icon(AppIcon::Tracks)
                .z_index(1)
                .build(cx),
            clips_panel_toggle_btn: IconToggleButton::builder(&style.top_panel_toggle_btn)
                .icon(AppIcon::ClipsPanel)
                .z_index(1)
                .build(cx),
            fx_rack_toggle_btn: IconToggleButton::builder(&style.top_panel_toggle_btn)
                .icon(AppIcon::FXRack)
                .z_index(1)
                .build(cx),
            piano_roll_toggle_btn: IconToggleButton::builder(&style.top_panel_toggle_btn)
                .icon(AppIcon::PianoKeys)
                .z_index(1)
                .build(cx),
            mixer_toggle_btn: IconToggleButton::builder(&style.top_panel_toggle_btn)
                .icon(AppIcon::Mixer)
                .z_index(1)
                .build(cx),
            properties_panel_toggle_btn: IconToggleButton::builder(&style.top_panel_toggle_btn)
                .icon(AppIcon::Properties)
                .z_index(1)
                .build(cx),
            command_palette_toggle_btn: IconToggleButton::builder(&style.top_panel_toggle_btn)
                .icon(AppIcon::CommandPalette)
                .z_index(1)
                .build(cx),

            section_seperator_2: Separator::builder(&style.top_panel_seperator)
                .z_index(1)
                .vertical(true)
                .build(cx),

            transport_section_label: Label::builder(&style.top_panel_section_title_label)
                .text("TRANSPORT")
                .z_index(1)
                .build(cx),
            skip_back_btn: IconButton::builder(&style.top_panel_icon_btn)
                .icon(AppIcon::SkipBack)
                .z_index(1)
                .build(cx),
            play_pause_btn: IconToggleButton::builder(&style.top_panel_play_pause_btn)
                .dual_icons(AppIcon::Play, AppIcon::Pause)
                .z_index(1)
                .build(cx),
            stop_btn: IconButton::builder(&style.top_panel_icon_btn)
                .icon(AppIcon::Stop)
                .z_index(1)
                .build(cx),
            skip_forward_btn: IconButton::builder(&style.top_panel_icon_btn)
                .icon(AppIcon::SkipForward)
                .z_index(1)
                .build(cx),
            transport_seperator_1: Separator::builder(&style.top_panel_seperator)
                .z_index(1)
                .vertical(true)
                .build(cx),
            loop_toggle_btn: IconToggleButton::builder(&style.top_panel_toggle_btn)
                .icon(AppIcon::Loop)
                .z_index(1)
                .build(cx),
            auto_return_toggle_btn: IconToggleButton::builder(&style.top_panel_toggle_btn)
                .icon(AppIcon::AutoReturn)
                .z_index(1)
                .build(cx),
            transport_seperator_2: Separator::builder(&style.top_panel_seperator)
                .z_index(1)
                .vertical(true)
                .build(cx),
            transport_box: QuadElement::builder(&style.top_panel_transport_box).build(cx),
            transport_menu_btn: IconButton::builder(&style.top_panel_icon_btn)
                .icon(AppIcon::Menu)
                .z_index(2)
                .build(cx),
            bpm_label: Label::builder(&style.top_panel_label)
                .text("bpm")
                .z_index(1)
                .build(cx),
            time_signature_label: Label::builder(&style.top_panel_label)
                .text("sig")
                .z_index(1)
                .build(cx),
            bmp_numeric_input: TextInput::builder(&style.top_panel_numeric_text_input)
                .text("130.00")
                .z_index(1)
                .build(cx),
            time_signature_numeric_input: TextInput::builder(&style.top_panel_numeric_text_input)
                .text("4/4")
                .z_index(1)
                .build(cx),
            transport_box_seperator: Separator::builder(&style.top_panel_box_separator)
                .z_index(1)
                .vertical(true)
                .build(cx),
            mbs_label: Label::builder(&style.top_panel_label)
                .text("m/b/s")
                .z_index(1)
                .build(cx),
            hmsm_label: Label::builder(&style.top_panel_label)
                .text("h/m/s/m")
                .z_index(1)
                .build(cx),
            mbs_numeric_input: TextInput::builder(&style.top_panel_numeric_text_input)
                .text("5.2.3")
                .z_index(1)
                .build(cx),
            hmsm_numeric_input: TextInput::builder(&style.top_panel_numeric_text_input)
                .text("00:00:12:202")
                .z_index(1)
                .build(cx),

            section_seperator_3: Separator::builder(&style.top_panel_seperator)
                .z_index(1)
                .vertical(true)
                .build(cx),

            record_section_label: Label::builder(&style.top_panel_section_title_label)
                .text("RECORD")
                .z_index(1)
                .build(cx),
            metronome_toggle_btn: IconToggleButton::builder(&style.top_panel_toggle_btn)
                .icon(AppIcon::Metronome)
                .z_index(1)
                .build(cx),
            record_seperator_1: Separator::builder(&style.top_panel_seperator)
                .z_index(1)
                .vertical(true)
                .build(cx),
            record_btn: IconToggleButton::builder(&style.top_panel_record_btn)
                .icon(AppIcon::Record)
                .z_index(1)
                .build(cx),
            record_seperator_2: Separator::builder(&style.top_panel_seperator)
                .z_index(1)
                .vertical(true)
                .build(cx),
            //record_source_label: Label::builder(&style.top_panel_label)
            //    .text("sources")
            //    .z_index(1)
            //    .build(cx),
            record_mic_toggle_btn: IconToggleButton::builder(&style.top_panel_toggle_btn)
                .icon(AppIcon::Mic)
                .z_index(1)
                .build(cx),
            record_notes_toggle_btn: IconToggleButton::builder(&style.top_panel_toggle_btn)
                .icon(AppIcon::PianoKeys)
                .z_index(1)
                .build(cx),
            record_automation_toggle_btn: IconToggleButton::builder(&style.top_panel_toggle_btn)
                .icon(AppIcon::Automation)
                .z_index(1)
                .build(cx),
            record_seperator_3: Separator::builder(&style.top_panel_seperator)
                .z_index(1)
                .vertical(true)
                .build(cx),
            //record_mode_label: Label::builder(&style.top_panel_label)
            //    .text("mode")
            //    .z_index(1)
            //    .build(cx),
            record_mode_dropdown_btn: IconLabelButton::builder(&style.top_panel_dropdown_btn)
                .text(Some("Overwrite"))
                .icon(Some(AppIcon::DropdownArrow))
                .z_index(1)
                .build(cx),

            section_seperator_4: Separator::builder(&style.top_panel_seperator)
                .z_index(1)
                .vertical(true)
                .build(cx),

            monitor_section_label: Label::builder(&style.top_panel_section_title_label)
                .text("MONITOR")
                .z_index(1)
                .build(cx),
            cpu_icon: Icon::builder(&style.top_panel_icon)
                .icon(AppIcon::CPU)
                .z_index(1)
                .build(cx),
        }
    }

    pub fn layout(&mut self, window_size: Size, style: &AppStyle) {
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
        //self.record_source_label.layout_aligned(
        //    point(
        //        self.record_seperator_2.el.rect().max_x() + element_spacing,
        //        btn_y,
        //    ),
        //    Align2::BOTTOM_LEFT,
        //);
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
        //self.record_mode_label.layout_aligned(
        //    point(
        //        self.record_seperator_3.el.rect().max_x() + element_spacing,
        //        btn_y,
        //    ),
        //    Align2::BOTTOM_LEFT,
        //);
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
    }
}
