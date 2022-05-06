use vizia::prelude::*;

use crate::ui::{icons::IconCode, Icon, ResizableStack, ResizableStackHandle};

#[derive(Lens)]
pub struct LeftBar {
    hide_browser: bool,

    current_tab: u16,

    browser_width: f32,
}

impl LeftBar {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self { hide_browser: true, current_tab: 0, browser_width: 320.0 }.build(cx, |cx| {
            HStack::new(cx, |cx| {
                VStack::new(cx, |cx| {
                    Icon::new(cx, IconCode::FileHierarchy, 32.0, 16.0)
                        .toggle_class(
                            "active_tab",
                            LeftBar::current_tab.map(|current_tab| *current_tab == 1),
                        )
                        .on_press(|cx| cx.emit(LeftBarEvent::Tab(1)));
                    Icon::new(cx, IconCode::Search, 32.0, 16.0)
                        .toggle_class(
                            "active_tab",
                            LeftBar::current_tab.map(|current_tab| *current_tab == 2),
                        )
                        .on_press(|cx| cx.emit(LeftBarEvent::Tab(2)));
                    Icon::new(cx, IconCode::Sample, 32.0, 16.0)
                        .toggle_class(
                            "active_tab",
                            LeftBar::current_tab.map(|current_tab| *current_tab == 3),
                        )
                        .on_press(|cx| cx.emit(LeftBarEvent::Tab(3)));
                    Icon::new(cx, IconCode::Piano, 32.0, 16.0)
                        .toggle_class(
                            "active_tab",
                            LeftBar::current_tab.map(|current_tab| *current_tab == 4),
                        )
                        .on_press(|cx| cx.emit(LeftBarEvent::Tab(4)));
                    Icon::new(cx, IconCode::Plug, 32.0, 16.0)
                        .toggle_class(
                            "active_tab",
                            LeftBar::current_tab.map(|current_tab| *current_tab == 5),
                        )
                        .on_press(|cx| cx.emit(LeftBarEvent::Tab(5)));
                    Icon::new(cx, IconCode::Tools, 32.0, 16.0)
                        .toggle_class(
                            "active_tab",
                            LeftBar::current_tab.map(|current_tab| *current_tab == 6),
                        )
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

                            // TODO - remove this when Vizia#120 is merged
                            cx.style.needs_restyle = true;
                        });
                    });
                })
                .class("browser")
                .bind(LeftBar::hide_browser, |h, hide| {
                    let hide = hide.get(h.cx);

                    let new_width = if hide {
                        0.0
                    } else {
                        // Safe to unwrap because I know it exists
                        let browser_width = h.cx.data::<LeftBar>().unwrap().browser_width;
                        browser_width
                    };

                    let width = h.cx.cache.get_width(h.entity);
                    // This is bad because it will create a new animation every time the panel is hidden/unhidden
                    // Need to either clean up unused animations or have a way to re-use animations
                    // This might require a rethink of the current animations API
                    let animation =
                        h.cx.add_animation(std::time::Duration::from_millis(100))
                            .add_keyframe(0.0, |keyframe| keyframe.set_width(Pixels(width)))
                            .add_keyframe(1.0, |keyframe| keyframe.set_width(Pixels(new_width)))
                            .build();
                    h.entity.play_animation(h.cx, animation);
                    h.width(Pixels(new_width));
                })
                .width(Pixels(0.0))
                .on_drag(|cx, width| cx.emit(LeftBarEvent::UpdateWidth(width)));
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
    fn element(&self) -> Option<String> {
        Some(String::from("left_bar"))
    }

    fn event(&mut self, _: &mut Context, event: &mut Event) {
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
                }
                LeftBarEvent::UpdateWidth(wid) => {
                    if !self.hide_browser {
                        self.browser_width = *wid;
                    }
                }
            }
        }
    }
}
