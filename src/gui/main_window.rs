mod top_panel;

use yarrow::prelude::*;

use crate::AppAction;

use super::{styling::AppStyle, OVERLAY_Z_INDEX};

pub struct MainWindow {
    top_panel: top_panel::TopPanel,

    tooltip: Tooltip,
}

impl MainWindow {
    pub fn new(style: &AppStyle, cx: &mut WindowContext<'_, crate::AppAction>) -> Self {
        cx.view.set_tooltip_actions(
            |info| AppAction::ShowTooltip {
                window_id: MAIN_WINDOW,
                info,
            },
            || AppAction::HideTooltip {
                window_id: MAIN_WINDOW,
            },
        );

        let mut new_self = Self {
            top_panel: top_panel::TopPanel::new(style, cx),
            tooltip: Tooltip::builder(&style.tooltip)
                .z_index(OVERLAY_Z_INDEX)
                .build(cx),
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
        cx: &mut AppContext<crate::AppAction>,
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

    pub fn show_tooltip(&mut self, info: TooltipInfo, cx: &mut AppContext<crate::AppAction>) {
        self.tooltip
            .show(&info.message, info.element_bounds, info.align, &mut cx.res);
    }

    pub fn hide_tooltip(&mut self) {
        self.tooltip.hide();
    }
}
