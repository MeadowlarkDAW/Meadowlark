mod top_panel;

use yarrow::prelude::*;

use super::styling::AppStyle;

pub struct MainWindow {
    top_panel: top_panel::TopPanel,
}

impl MainWindow {
    pub fn new(style: &AppStyle, cx: &mut WindowContext<'_, crate::Action>) -> Self {
        let mut new_self = Self {
            top_panel: top_panel::TopPanel::new(style, cx),
        };

        new_self.full_layout(cx.logical_size(), style);

        new_self
    }

    fn full_layout(&mut self, window_size: Size, style: &AppStyle) {
        self.top_panel.layout(window_size, style);
    }

    pub fn on_window_event(
        &mut self,
        event: AppWindowEvent,
        cx: &mut AppContext<crate::Action>,
        style: &AppStyle,
    ) {
        match event {
            AppWindowEvent::WindowResized => {
                let size = cx.window_context(MAIN_WINDOW).unwrap().logical_size();
                self.full_layout(size, style);
            }
            _ => {}
        }
    }
}
