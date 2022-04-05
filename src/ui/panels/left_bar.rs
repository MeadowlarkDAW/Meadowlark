use vizia::*;

use crate::ui::{icons::IconCode, Icon};

#[derive(Lens)]
pub struct LeftBar {
    hide_browser: bool,
    current_tab: u16,
}

impl LeftBar {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self { hide_browser: false, current_tab: 0 }.build2(cx, |cx| {
            HStack::new(cx, |cx| {
                VStack::new(cx, |cx| {
                    Icon::new(cx, IconCode::FileHierarchy, 32.0, 16.0)
                        .on_press(|cx| cx.emit(LeftBarEvent::SwitchTab(0)));
                    Icon::new(cx, IconCode::Search, 32.0, 16.0)
                        .on_press(|cx| cx.emit(LeftBarEvent::SwitchTab(1)));
                    Icon::new(cx, IconCode::Sample, 32.0, 16.0)
                        .on_press(|cx| cx.emit(LeftBarEvent::SwitchTab(2)));
                    Icon::new(cx, IconCode::Piano, 32.0, 16.0)
                        .on_press(|cx| cx.emit(LeftBarEvent::SwitchTab(3)));
                    Icon::new(cx, IconCode::Plug, 32.0, 16.0)
                        .on_press(|cx| cx.emit(LeftBarEvent::SwitchTab(4)));
                    Icon::new(cx, IconCode::Tools, 32.0, 16.0)
                        .on_press(|cx| cx.emit(LeftBarEvent::SwitchTab(5)));
                })
                .class("left_bar");

                VStack::new(cx, |cx| {

                    Binding::new(cx, LeftBar::current_tab, |cx, current_tab| {

                        let tab_value: u16 = current_tab.get(cx);

                        match tab_value {
                            0 => {Label::new(cx, "Page 0");},
                            1 => {Label::new(cx, "Page 1");},
                            2 => {Label::new(cx, "Page 2");},
                            3 => {Label::new(cx, "Page 3");},
                            4 => {Label::new(cx, "Page 4");},
                            5 => {Label::new(cx, "Page 5");},
                            _ => ()
                        }
                    });
                    
                })
                .class("browser")
                .toggle_class("hide_browser", LeftBar::hide_browser);
            })
            .class("left_bar_wrapper");
        })
    }
}

pub enum LeftBarEvent {
    SwitchTab(u16),
}

impl View for LeftBar {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        if let Some(left_bar_event) = event.message.downcast() {
            match left_bar_event {
                LeftBarEvent::SwitchTab(tab) => {
                    if self.current_tab == *tab {
                        self.hide_browser = !self.hide_browser;

                        if self.hide_browser {
                            self.current_tab = 20;
                        }
                    } else {
                        self.current_tab = *tab;
                    }

                    event.consume()
                }
            }

            println!("status: \ntab: {}\nhide?: {}", self.current_tab, self.hide_browser);
        }
    }
}
