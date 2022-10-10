use gtk::gio::Menu;
use gtk::{prelude::*, ListView, Paned, SingleSelection};
use gtk::{
    Align, Box, Button, CenterBox, Image, Label, Notebook, Orientation, Overflow, PolicyType,
    PopoverMenuBar, ScrolledWindow, SearchEntry, Separator, SignalListItemFactory, Stack,
    ToggleButton,
};

use crate::state::browser_panel::BrowserPanelListItem;
use crate::state::AppState;

pub struct BrowserPanelWidgets {
    box_: Box,
}

impl BrowserPanelWidgets {
    pub fn new(app_state: &AppState) -> Self {
        let box_ = Box::builder().name("browser_panel").orientation(Orientation::Vertical).build();

        // --- Tabs ------------------------------------------------------

        let title_text = Label::builder()
            .label("BROWSER")
            .css_classes(vec!["panel_title".into()])
            .halign(Align::Start)
            .build();

        box_.append(&title_text);

        let tabs_group = Box::builder()
            .orientation(Orientation::Vertical)
            .margin_top(4)
            .margin_start(3)
            .margin_end(3)
            .build();

        let samples_tab_btn_contents =
            Box::builder().orientation(Orientation::Horizontal).spacing(4).build();
        samples_tab_btn_contents.append(&Image::from_icon_name("mdk-audio-symbolic"));
        samples_tab_btn_contents.append(&Label::new(Some("Samples")));
        let samples_tab_btn = ToggleButton::builder()
            .child(&samples_tab_btn_contents)
            .css_classes(vec!["category_tab".into()])
            .build();

        let multisamples_tab_btn_contents =
            Box::builder().orientation(Orientation::Horizontal).spacing(4).build();
        multisamples_tab_btn_contents.append(&Image::from_icon_name("mdk-instrument-symbolic"));
        multisamples_tab_btn_contents.append(&Label::new(Some("Multisamples")));
        let multisamples_tab_btn = ToggleButton::builder()
            .child(&multisamples_tab_btn_contents)
            .css_classes(vec!["category_tab".into()])
            .group(&samples_tab_btn)
            .build();

        let synths_tab_btn_contents =
            Box::builder().orientation(Orientation::Horizontal).spacing(4).build();
        synths_tab_btn_contents.append(&Image::from_icon_name("mdk-oscilloscope-symbolic"));
        synths_tab_btn_contents.append(&Label::new(Some("Synths")));
        let synths_tab_btn = ToggleButton::builder()
            .child(&synths_tab_btn_contents)
            .css_classes(vec!["category_tab".into()])
            .group(&samples_tab_btn)
            .build();

        let fx_tab_btn_contents =
            Box::builder().orientation(Orientation::Horizontal).spacing(4).build();
        fx_tab_btn_contents.append(&Image::from_icon_name("mdk-fx-symbolic"));
        fx_tab_btn_contents.append(&Label::new(Some("Effects")));
        let fx_tab_btn = ToggleButton::builder()
            .child(&fx_tab_btn_contents)
            .css_classes(vec!["category_tab".into()])
            .group(&samples_tab_btn)
            .build();

        let midi_tab_btn_contents =
            Box::builder().orientation(Orientation::Horizontal).spacing(4).build();
        midi_tab_btn_contents.append(&Image::from_icon_name("mdk-midi-symbolic"));
        midi_tab_btn_contents.append(&Label::new(Some("Piano Roll Clips")));
        let midi_tab_btn = ToggleButton::builder()
            .child(&midi_tab_btn_contents)
            .css_classes(vec!["category_tab".into()])
            .group(&samples_tab_btn)
            .build();

        let automation_tab_btn_contents =
            Box::builder().orientation(Orientation::Horizontal).spacing(4).build();
        automation_tab_btn_contents.append(&Image::from_icon_name("mdk-automation-symbolic"));
        automation_tab_btn_contents.append(&Label::new(Some("Automation Clips")));
        let automation_tab_btn = ToggleButton::builder()
            .child(&automation_tab_btn_contents)
            .css_classes(vec!["category_tab".into()])
            .group(&samples_tab_btn)
            .build();

        let projects_tab_btn_contents =
            Box::builder().orientation(Orientation::Horizontal).spacing(4).build();
        projects_tab_btn_contents.append(&Image::from_icon_name("mdk-music-symbolic"));
        projects_tab_btn_contents.append(&Label::new(Some("Projects")));
        let projects_tab_btn = ToggleButton::builder()
            .child(&projects_tab_btn_contents)
            .css_classes(vec!["category_tab".into()])
            .group(&samples_tab_btn)
            .build();

        let files_tab_btn_contents =
            Box::builder().orientation(Orientation::Horizontal).spacing(4).build();
        files_tab_btn_contents.append(&Image::from_icon_name("mdk-folder-symbolic"));
        files_tab_btn_contents.append(&Label::new(Some("Files")));
        let files_tab_btn = ToggleButton::builder()
            .child(&files_tab_btn_contents)
            .css_classes(vec!["category_tab".into()])
            .group(&samples_tab_btn)
            .build();

        tabs_group.append(&samples_tab_btn);
        tabs_group.append(&multisamples_tab_btn);
        tabs_group.append(&synths_tab_btn);
        tabs_group.append(&fx_tab_btn);
        tabs_group.append(&midi_tab_btn);
        tabs_group.append(&automation_tab_btn);
        tabs_group.append(&projects_tab_btn);
        tabs_group.append(&files_tab_btn);

        box_.append(&tabs_group);

        // --- Search box ------------------------------------------------------

        let search_bar_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .margin_start(4)
            .margin_end(4)
            .build();

        let search_entry =
            SearchEntry::builder().placeholder_text("test").margin_top(4).hexpand(true).build();

        let toggle_favorites_filter_button = ToggleButton::builder()
            .icon_name("mdk-favorite-symbolic")
            .css_classes(vec!["small-image-toggle".into()])
            .margin_top(5)
            .margin_start(4)
            .build();

        search_bar_box.append(&search_entry);
        search_bar_box.append(&toggle_favorites_filter_button);

        box_.append(&search_bar_box);

        // --- List panels ------------------------------------------------------

        let top_panel_list = ScrolledWindow::builder()
            .vscrollbar_policy(PolicyType::Automatic)
            .hscrollbar_policy(PolicyType::Automatic)
            .kinetic_scrolling(true)
            .min_content_height(100)
            .css_classes(vec!["browser_list_pane".into()])
            .margin_bottom(5)
            .build();
        let top_panel_list_factory = SignalListItemFactory::new();
        top_panel_list_factory.connect_setup(move |_, list_item| {
            let contents = Box::builder()
                .orientation(Orientation::Horizontal)
                .css_classes(vec!["browser_list_item".into()])
                .spacing(6)
                .build();

            contents.append(&Image::from_icon_name("mdk-folder-symbolic"));
            contents.append(&Label::new(None));
            list_item.set_child(Some(&contents));
        });
        top_panel_list_factory.connect_bind(move |_, list_item| {
            let list_object = list_item.item().unwrap().downcast::<BrowserPanelListItem>().unwrap();

            // Get `i32` from `IntegerObject`
            let number = list_object.property::<i32>("number");

            let contents = list_item.child().unwrap().downcast::<Box>().unwrap();
            let label = contents.last_child().unwrap().downcast::<Label>().unwrap();

            // Set "label" to "number"
            label.set_label(&number.to_string());
        });
        let top_panel_list_selection_model =
            SingleSelection::new(Some(&app_state.browser_panel.top_panel_list_model));
        let top_panel_list_view =
            ListView::new(Some(&top_panel_list_selection_model), Some(&top_panel_list_factory));
        top_panel_list.set_child(Some(&top_panel_list_view));

        let bottom_panel_list = ScrolledWindow::builder()
            .vscrollbar_policy(PolicyType::Automatic)
            .hscrollbar_policy(PolicyType::Automatic)
            .kinetic_scrolling(true)
            .min_content_height(100)
            .css_classes(vec!["browser_list_pane".into()])
            .build();
        let bottom_panel_list_factory = SignalListItemFactory::new();
        bottom_panel_list_factory.connect_setup(move |_, list_item| {
            let contents = Box::builder()
                .orientation(Orientation::Horizontal)
                .css_classes(vec!["browser_list_item".into()])
                .spacing(6)
                .build();

            contents.append(&Image::from_icon_name("mdk-audio-symbolic"));
            contents.append(&Label::new(None));
            list_item.set_child(Some(&contents));
        });
        bottom_panel_list_factory.connect_bind(move |_, list_item| {
            let list_object = list_item.item().unwrap().downcast::<BrowserPanelListItem>().unwrap();

            // Get `i32` from `IntegerObject`
            let number = list_object.property::<i32>("number");

            let contents = list_item.child().unwrap().downcast::<Box>().unwrap();
            let label = contents.last_child().unwrap().downcast::<Label>().unwrap();

            // Set "label" to "number"
            label.set_label(&number.to_string());
        });
        let bottom_panel_list_selection_model =
            SingleSelection::new(Some(&app_state.browser_panel.bottom_panel_list_model));
        let bottom_panel_list_view = ListView::new(
            Some(&bottom_panel_list_selection_model),
            Some(&bottom_panel_list_factory),
        );
        bottom_panel_list.set_child(Some(&bottom_panel_list_view));

        let browser_list_panes = Paned::builder()
            .orientation(Orientation::Vertical)
            .resize_start_child(true)
            .resize_end_child(true)
            .shrink_start_child(false)
            .shrink_end_child(false)
            .position(200)
            .start_child(&top_panel_list)
            .end_child(&bottom_panel_list)
            .vexpand(true)
            .hexpand(true)
            .margin_top(4)
            .margin_start(3)
            .margin_end(3)
            .build();

        box_.append(&browser_list_panes);

        // --- Browser playback controls ------------------------------------------------------

        let browser_playback_controls_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .margin_top(3)
            .margin_bottom(3)
            .margin_start(3)
            .margin_end(3)
            .spacing(3)
            .build();

        let toggle_playback_btn = ToggleButton::builder()
            .icon_name("mdk-sound-high-symbolic")
            .css_classes(vec!["small-image-toggle".into()])
            .build();

        let play_pause_btn = ToggleButton::builder()
            .icon_name("mdk-play-symbolic")
            .css_classes(vec!["small-image-toggle".into()])
            .build();

        let stop_btn = ToggleButton::builder()
            .icon_name("mdk-stop-symbolic")
            .css_classes(vec!["small-image-toggle".into()])
            .build();

        let toggle_loop_btn = ToggleButton::builder()
            .icon_name("mdk-loop-symbolic")
            .css_classes(vec!["small-image-toggle".into()])
            .build();

        browser_playback_controls_box.append(&toggle_playback_btn);
        browser_playback_controls_box.append(&play_pause_btn);
        browser_playback_controls_box.append(&stop_btn);
        browser_playback_controls_box.append(&toggle_loop_btn);

        box_.append(&browser_playback_controls_box);

        box_.set_visible(app_state.browser_panel.shown);

        Self { box_ }
    }

    pub fn container_widget(&self) -> &Box {
        &self.box_
    }

    pub fn toggle_shown(&self, shown: bool) {
        self.box_.set_visible(shown);
    }
}