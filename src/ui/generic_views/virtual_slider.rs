use vizia::prelude::*;

#[derive(Debug, Lens, Clone, Copy, Data)]
pub struct BoundVirtualSliderState {
    pub current_normal: f32,
    pub modulation_amount: f32,
    pub modulation_visible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VirtualSliderEvent {
    Changed(f32),
    GestureStarted,
    GestureFinished,
    ResetToDefault,
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
    /// The default value is `0.0042`.
    pub drag: f32,

    /// The scalar applied when scrolling the mouse wheel.
    ///
    /// The default value is `0.005`.
    pub wheel: f32,

    /// The scalar applied when pressing the up/down keys on the keyboard.
    ///
    /// The default value is `0.1`.
    pub arrow: f32,

    /// The additional scalar applied when a modifier key is pressed.
    ///
    /// The default value is `0.04`.
    pub modifier: f32,
}

impl Default for VirtualSliderScalars {
    fn default() -> Self {
        Self { drag: 0.0042, wheel: 0.005, arrow: 0.1, modifier: 0.04 }
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
    prev_drag_pos: f32,
}

impl<L: Lens<Target = BoundVirtualSliderState>> VirtualSlider<L> {
    pub fn new(
        cx: &mut Context,
        lens: L,
        mode: VirtualSliderMode,
        direction: VirtualSliderDirection,
        scalars: VirtualSliderScalars,
    ) -> Self {
        Self { lens, mode, direction, scalars, is_dragging: false, prev_drag_pos: 0.0 }
    }

    pub fn on_event(
        &mut self,
        cx: &mut EventContext,
        event: &mut Event,
    ) -> Option<VirtualSliderEvent> {
        // TODO: Use pointer-locking once that feature becomes available in Vizia.

        let mut status = None;

        let move_virtual_slider = |self_ref: &mut Self,
                                   mut delta_normal: f32,
                                   cx: &mut EventContext|
         -> Option<VirtualSliderEvent> {
            if cx.modifiers.contains(Modifiers::SHIFT) {
                delta_normal *= self_ref.scalars.modifier;
            }

            let current_normal = self_ref.lens.get(cx).current_normal;

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

                self.prev_drag_pos = match self.direction {
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
                    let (new_pos, delta_normal) = match self.direction {
                        VirtualSliderDirection::Vertical => {
                            (*y, (self.prev_drag_pos - *y) * self.scalars.drag)
                        }
                        VirtualSliderDirection::Horizontal => {
                            (*x, (*x - self.prev_drag_pos) * self.scalars.drag)
                        }
                    };

                    self.prev_drag_pos = new_pos;

                    status = move_virtual_slider(self, delta_normal, cx);
                }
            }
            WindowEvent::MouseScroll(x, y) => match self.direction {
                VirtualSliderDirection::Vertical => {
                    if *y != 0.0 {
                        let delta_normal = -*y * self.scalars.wheel;

                        status = move_virtual_slider(self, delta_normal, cx);
                    }
                }
                VirtualSliderDirection::Horizontal => {
                    if *x != 0.0 {
                        let delta_normal = *x * self.scalars.wheel;

                        status = move_virtual_slider(self, delta_normal, cx);
                    }
                }
            },
            WindowEvent::MouseDoubleClick(button) if *button == MouseButton::Left => {
                self.is_dragging = false;

                cx.release();

                status = Some(VirtualSliderEvent::ResetToDefault);
            }
            WindowEvent::KeyDown(Code::ArrowUp, _) => {
                if self.direction == VirtualSliderDirection::Vertical {
                    status = move_virtual_slider(self, self.scalars.arrow, cx);
                }
            }
            WindowEvent::KeyDown(Code::ArrowDown, _) => {
                if self.direction == VirtualSliderDirection::Vertical {
                    status = move_virtual_slider(self, -self.scalars.arrow, cx);
                }
            }
            WindowEvent::KeyDown(Code::ArrowLeft, _) => {
                if self.direction == VirtualSliderDirection::Horizontal {
                    status = move_virtual_slider(self, -self.scalars.arrow, cx);
                }
            }
            WindowEvent::KeyDown(Code::ArrowRight, _) => {
                if self.direction == VirtualSliderDirection::Horizontal {
                    status = move_virtual_slider(self, self.scalars.arrow, cx);
                }
            }
            _ => {}
        });

        status
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
