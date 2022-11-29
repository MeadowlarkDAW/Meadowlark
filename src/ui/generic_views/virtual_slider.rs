use vizia::prelude::*;

#[derive(Debug, Lens, Clone, Copy, Data)]
pub struct BoundVirtualSliderState {
    pub value_normalized: f32,
    pub default_normalized: f32,
    pub modulation_amount: f32,
    pub modulation_visible: bool,
}

impl BoundVirtualSliderState {
    pub fn from_value_only(value_normalized: f32, default_normalized: f32) -> Self {
        Self {
            value_normalized,
            default_normalized,
            modulation_amount: 0.0,
            modulation_visible: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VirtualSliderEvent {
    Changed(f32),
    GestureStarted,
    GestureFinished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualSliderMode {
    Continuous,
    Discrete { steps: u16 },
}

impl VirtualSliderMode {
    pub fn snap_normal(&self, normal: f32) -> f32 {
        let normal = normal.clamp(0.0, 1.0);
        match self {
            VirtualSliderMode::Continuous => normal,
            VirtualSliderMode::Discrete { steps } => {
                if *steps < 2 {
                    0.0
                } else {
                    let nearest_step = (normal * f32::from(*steps - 1)).round() as u16;

                    // Make sure that the values at 0.0 and 1.0 are snapped to those exactly.
                    if nearest_step == 0 {
                        0.0
                    } else if nearest_step == (*steps - 1) {
                        1.0
                    } else {
                        f32::from(nearest_step) / f32::from(*steps - 1)
                    }
                }
            }
        }
    }
}

impl Default for VirtualSliderMode {
    fn default() -> Self {
        VirtualSliderMode::Continuous
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VirtualSliderScalars {
    /// The scalar applied when dragging the mouse up/down.
    ///
    /// The default value is `0.0044`.
    pub drag: f32,

    /// The scalar applied when scrolling the mouse wheel.
    ///
    /// The default value is `0.005`.
    pub wheel: f32,

    /// The scalar applied when pressing the up/down keys on the keyboard.
    ///
    /// The default value is `0.01`.
    pub arrow: f32,

    /// The additional scalar applied when a modifier key is pressed.
    ///
    /// The default value is `0.04`.
    pub modifier: f32,
}

impl Default for VirtualSliderScalars {
    fn default() -> Self {
        Self { drag: 0.0044, wheel: 0.005, arrow: 0.01, modifier: 0.04 }
    }
}

/// The reusable logic for a view which modifies a parameter by dragging
/// up/down with the mouse, as well as mouse wheel scrolling and keyboard
/// input.
#[derive(Debug, Clone, Copy)]
pub struct VirtualSlider<L: Lens<Target = BoundVirtualSliderState>> {
    lens: L,

    mode: VirtualSliderMode,
    direction: VirtualSliderDirection,
    scalars: VirtualSliderScalars,

    is_dragging: bool,
    drag_start_pos: f32,
    drag_start_normal: f32,
}

impl<L: Lens<Target = BoundVirtualSliderState>> VirtualSlider<L> {
    pub fn new(
        cx: &mut Context,
        lens: L,
        mode: VirtualSliderMode,
        direction: VirtualSliderDirection,
        scalars: VirtualSliderScalars,
    ) -> Self {
        Self {
            lens,
            mode,
            direction,
            scalars,
            is_dragging: false,
            drag_start_pos: 0.0,
            drag_start_normal: 0.0,
        }
    }

    pub fn on_event(
        &mut self,
        cx: &mut EventContext,
        event: &mut Event,
    ) -> (Option<VirtualSliderEvent>, bool) {
        // TODO: Use pointer-locking once that feature becomes available in Vizia.

        let mut status = None;
        let mut did_reset = false;

        let move_virtual_slider_by_amount = |self_ref: &mut Self,
                                             mut delta_normal: f32,
                                             cx: &mut EventContext|
         -> Option<VirtualSliderEvent> {
            if cx.modifiers.contains(Modifiers::SHIFT) {
                delta_normal *= self_ref.scalars.modifier;
            }

            let current_normal = self_ref.lens.get(cx).value_normalized;

            let new_normal = (current_normal + delta_normal).clamp(0.0, 1.0);

            if current_normal != new_normal {
                Some(VirtualSliderEvent::Changed(new_normal))
            } else {
                None
            }
        };

        event.map(|window_event, _| match window_event {
            WindowEvent::MouseDown(button) if *button == MouseButton::Left => {
                self.is_dragging = true;

                self.drag_start_normal = self.lens.get(cx).value_normalized;

                self.drag_start_pos = match self.direction {
                    VirtualSliderDirection::Vertical => cx.mouse.left.pos_down.1,
                    VirtualSliderDirection::Horizontal => cx.mouse.left.pos_down.0,
                };

                cx.capture();
                cx.focus_with_visibility(false);

                status = Some(VirtualSliderEvent::GestureStarted);
            }
            WindowEvent::MouseUp(button) if *button == MouseButton::Left => {
                self.is_dragging = false;

                cx.release();

                status = Some(VirtualSliderEvent::GestureFinished);
            }
            WindowEvent::MouseMove(x, y) => {
                if self.is_dragging {
                    let mut delta_normal = match self.direction {
                        VirtualSliderDirection::Vertical => {
                            (self.drag_start_pos - *y) * self.scalars.drag
                        }
                        VirtualSliderDirection::Horizontal => {
                            (*x - self.drag_start_pos) * self.scalars.drag
                        }
                    };

                    if cx.modifiers.contains(Modifiers::SHIFT) {
                        delta_normal *= self.scalars.modifier;
                    }

                    let new_normal = (self.drag_start_normal + delta_normal).clamp(0.0, 1.0);

                    let current_normal = self.lens.get(cx).value_normalized;

                    if current_normal != new_normal {
                        status = Some(VirtualSliderEvent::Changed(new_normal))
                    }
                }
            }
            WindowEvent::MouseScroll(x, y) => match self.direction {
                VirtualSliderDirection::Vertical => {
                    if *y != 0.0 {
                        let delta_normal = *y * self.scalars.wheel;

                        status = move_virtual_slider_by_amount(self, delta_normal, cx);
                    }
                }
                VirtualSliderDirection::Horizontal => {
                    if *x != 0.0 {
                        let delta_normal = *x * self.scalars.wheel;

                        status = move_virtual_slider_by_amount(self, delta_normal, cx);
                    }
                }
            },
            WindowEvent::MouseDoubleClick(button) if *button == MouseButton::Left => {
                self.is_dragging = false;

                cx.release();

                let state = self.lens.get(cx);

                if state.value_normalized != state.default_normalized {
                    status = Some(VirtualSliderEvent::Changed(state.default_normalized));
                }

                did_reset = true;
            }
            WindowEvent::KeyDown(Code::ArrowUp, _) => {
                if self.direction == VirtualSliderDirection::Vertical {
                    status = move_virtual_slider_by_amount(self, self.scalars.arrow, cx);
                }
            }
            WindowEvent::KeyDown(Code::ArrowDown, _) => {
                if self.direction == VirtualSliderDirection::Vertical {
                    status = move_virtual_slider_by_amount(self, -self.scalars.arrow, cx);
                }
            }
            WindowEvent::KeyDown(Code::ArrowLeft, _) => {
                if self.direction == VirtualSliderDirection::Horizontal {
                    status = move_virtual_slider_by_amount(self, -self.scalars.arrow, cx);
                }
            }
            WindowEvent::KeyDown(Code::ArrowRight, _) => {
                if self.direction == VirtualSliderDirection::Horizontal {
                    status = move_virtual_slider_by_amount(self, self.scalars.arrow, cx);
                }
            }
            _ => {}
        });

        (status, did_reset)
    }

    pub fn snap_normal(&self, normal: f32) -> f32 {
        self.mode.snap_normal(normal)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualSliderDirection {
    Vertical,
    Horizontal,
}

impl Default for VirtualSliderDirection {
    fn default() -> Self {
        VirtualSliderDirection::Vertical
    }
}
