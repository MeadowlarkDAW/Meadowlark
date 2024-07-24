mod action;
mod action_handler;

use yarrow::action_queue::{ActionReceiver, ActionSender};
use yarrow::event::{AppWindowEvent, KeyboardEvent};
use yarrow::math::Size;
use yarrow::window::{ScaleFactorConfig, WindowCloseRequest, WindowConfig, WindowID};
use yarrow::AppContext;

use crate::config::AppConfig;
use crate::gui::main_window::MainWindow;
use crate::gui::styling::{AppStyle, AppTheme};

pub use action::AppAction;

pub struct App {
    action_sender: ActionSender<AppAction>,
    action_receiver: ActionReceiver<AppAction>,

    main_window: Option<MainWindow>,
    config: AppConfig,
    style: AppStyle,

    queued_action_for_main_window: Vec<AppAction>,
}

impl App {
    pub fn new(
        action_sender: ActionSender<AppAction>,
        action_receiver: ActionReceiver<AppAction>,
    ) -> Self {
        Self {
            action_sender,
            action_receiver,
            main_window: None,
            config: AppConfig::load(),
            style: AppStyle::new(AppTheme::default()),
            queued_action_for_main_window: Vec::new(),
        }
    }
}

impl yarrow::Application for App {
    type Action = AppAction;

    fn init(&mut self) -> Result<yarrow::AppConfig, Box<dyn std::error::Error>> {
        Ok(yarrow::AppConfig {
            pointer_locking_enabled: self.config.pointer_locking_enabled,
            ..Default::default()
        })
    }

    fn main_window_config(&self) -> WindowConfig {
        #[cfg(debug_assertions)]
        let title = if crate::IS_NIGHTLY {
            String::from("Meadowlark (Nightly) [DEV BUILD]")
        } else {
            String::from("Meadowlark [DEV BUILD]")
        };
        #[cfg(not(debug_assertions))]
        let title = if crate::IS_NIGHTLY {
            String::from("Meadowlark (Nightly)")
        } else {
            String::from("Meadowlark")
        };

        WindowConfig {
            title,
            size: Size::new(1920.0, 1000.0),
            scale_factor: ScaleFactorConfig::Custom(1.0.into()),
            ..Default::default()
        }
    }

    fn on_window_event(
        &mut self,
        event: AppWindowEvent,
        window_id: WindowID,
        cx: &mut AppContext<Self::Action>,
    ) {
        if let AppWindowEvent::WindowOpened = event {
            if window_id == yarrow::MAIN_WINDOW {
                crate::gui::fonts::load_fonts(&mut cx.res);
                crate::gui::icons::load_icons(&mut cx.res);

                let mut main_window_cx = cx.window_context(yarrow::MAIN_WINDOW).unwrap();
                main_window_cx.view.clear_color = self.style.clear_color.into();

                self.main_window = Some(MainWindow::new(&self.style, &mut main_window_cx));
            } else {
                // TODO
            }
        } else {
            if window_id == yarrow::MAIN_WINDOW {
                if let Some(main_window) = &mut self.main_window {
                    main_window.on_window_event(event, cx, &self.style);
                }
            } else {
                // TODO
            }
        }
    }

    fn on_keyboard_event(
        &mut self,
        _event: KeyboardEvent,
        _window_id: WindowID,
        _cx: &mut AppContext<Self::Action>,
    ) {
        // TODO
    }

    fn on_action_emitted(&mut self, cx: &mut AppContext<Self::Action>) {
        if self.main_window.is_none() {
            // Process the actions later when the main window is ready.
            while let Ok(action) = self.action_receiver.try_recv() {
                self.queued_action_for_main_window.push(action);
            }
            return;
        }

        if !self.queued_action_for_main_window.is_empty() {
            let mut actions = Vec::new();
            std::mem::swap(&mut actions, &mut self.queued_action_for_main_window);

            for action in actions.drain(..) {
                if let Err(fatal_error) = self::action_handler::handle_action(action, self, cx) {
                    log::error!("fatal error: {}", &fatal_error);

                    // TODO: Handle fatal errors more gracefully.
                    panic!("fatal error: {}", fatal_error);
                }
            }
        }

        while let Ok(action) = self.action_receiver.try_recv() {
            if let Err(fatal_error) = self::action_handler::handle_action(action, self, cx) {
                log::error!("fatal error: {}", &fatal_error);

                // TODO: Handle fatal errors more gracefully.
                panic!("fatal error: {}", fatal_error);
            }
        }
    }

    fn on_tick(&mut self, _dt: f64, _cx: &mut AppContext<Self::Action>) {
        // TODO
    }

    fn on_request_to_close_window(
        &mut self,
        window_id: WindowID,
        // This is only relevant for audio plugins that run inside a host.
        _host_will_force_close: bool,
        _cx: &mut AppContext<Self::Action>,
    ) -> WindowCloseRequest {
        if window_id == yarrow::MAIN_WINDOW {
            // TODO: Show prompt asking if user wants to save their project first.
        }

        WindowCloseRequest::CloseImmediately
    }
}
