use vizia::*;

use crate::ui::{icons::IconCode, Icon, ResizableStack};

#[derive(Lens)]
pub struct LeftBar {
    hide_browser: bool,

    current_tab: u16,

    current_width: f32,
    browser_width: f32,
}

impl LeftBar {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self { hide_browser: false, current_tab: 0, current_width: 320.0, browser_width: 320.0 }.build(cx, |cx| {
            HStack::new(cx, |cx| {
                VStack::new(cx, |cx| {
                    Icon::new(cx, IconCode::FileHierarchy, 32.0, 16.0)
                        .on_press(|cx| cx.emit(LeftBarEvent::Tab(1)));
                    Icon::new(cx, IconCode::Search, 32.0, 16.0)
                        .on_press(|cx| cx.emit(LeftBarEvent::Tab(2)));
                    Icon::new(cx, IconCode::Sample, 32.0, 16.0)
                        .on_press(|cx| cx.emit(LeftBarEvent::Tab(3)));
                    Icon::new(cx, IconCode::Piano, 32.0, 16.0)
                        .on_press(|cx| cx.emit(LeftBarEvent::Tab(4)));
                    Icon::new(cx, IconCode::Plug, 32.0, 16.0)
                        .on_press(|cx| cx.emit(LeftBarEvent::Tab(5)));
                    Icon::new(cx, IconCode::Tools, 32.0, 16.0)
                        .on_press(|cx| cx.emit(LeftBarEvent::Tab(6)));
                })
                .class("left_bar");

                ResizableStack::new(cx, |cx| {
                    VStack::new(cx, |cx| {
                        Binding::new(cx, LeftBar::current_tab, |cx, current_tab| {
                            let tab_value: u16 = current_tab.get(cx);

                            match tab_value {
                                1 => {
                                    Label::new(cx, "Page 0");
                                }
                                2 => {
                                    Label::new(cx, "Page 1");
                                }
                                3 => {
                                    Label::new(cx, "Page 2");
                                }
                                4 => {
                                    Label::new(cx, "Page 3");
                                }
                                5 => {
                                    Label::new(cx, "Page 4");
                                }
                                6 => {
                                    Label::new(cx, "Page 5");
                                }
                                _ => (),
                            }
                        });
                    });
                })
                .class("browser")
                .bind(LeftBar::current_width, |h,w| {
                    let v: f32 = w.get(h.cx);
                    h.width(Pixels(v));
                })
                .on_geo_changed(|cx, geo| {
                    if geo.contains(GeometryChanged::WIDTH_CHANGED) {
                        cx.emit(LeftBarEvent::UpdateWidth(cx.cache.get_width(cx.current)));
                    }
                });
            })
            .class("left_bar_wrapper");
        })
    }
}

pub enum LeftBarEvent {
    Tab(u16),
    UpdateWidth(f32),
}

impl View for LeftBar {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        if let Some(left_bar_event) = event.message.downcast() {
            match left_bar_event {
                LeftBarEvent::Tab(tab) => {
                    if self.current_tab == *tab {
                        self.hide_browser = !self.hide_browser;
                        self.current_tab = 0;
                    } else {
                        self.current_tab = *tab;
                        self.hide_browser = false;
                    }

                    if self.hide_browser {
                        self.current_width = 0.0;
                    } else {
                        self.current_width = self.browser_width;
                    }
                    
                }
                LeftBarEvent::UpdateWidth(wid) => {
                    if !self.hide_browser {
                        self.browser_width = *wid;

                        if self.current_width != 0.0 {
                            self.current_width = self.browser_width;
                        }
                    }
                }
            }
        }
    }
}
