use vizia::{
    prelude::*,
    vg::{Color, LineCap},
};

use super::virtual_slider::{
    BoundVirtualSliderState, VirtualSlider, VirtualSliderDirection, VirtualSliderEvent,
    VirtualSliderMode, VirtualSliderScalars,
};

pub struct KnobView<L: Lens<Target = BoundVirtualSliderState>> {
    virtual_slider: VirtualSlider<L>,

    on_event: Box<dyn Fn(&mut EventContext, VirtualSliderEvent)>,
}

impl<L: Lens<Target = BoundVirtualSliderState>> KnobView<L> {
    pub fn new(
        cx: &mut Context,
        lens: L,
        mode: VirtualSliderMode,
        direction: VirtualSliderDirection,
        scalars: VirtualSliderScalars,
        radius: Units,
        bipolar_mode: bool,
        style: KnobViewStyle,
        on_event: impl Fn(&mut EventContext, VirtualSliderEvent) + 'static,
    ) -> Handle<Self> {
        Self {
            virtual_slider: VirtualSlider::new(cx, lens.clone(), mode, direction, scalars),
            on_event: Box::new(on_event),
        }
        .build(cx, move |cx| {
            let state = lens.get(cx);

            let knob_renderer = KnobViewRenderer::new(
                cx,
                state.value_normalized,
                radius,
                bipolar_mode,
                style.clone(),
            )
            .width(Stretch(1.0))
            .height(Stretch(1.0));
            let knob_renderer_entity = knob_renderer.entity;

            Binding::new(knob_renderer.cx, lens, move |cx, state| {
                let state = state.get(cx);
                if let Some(view) = cx.views.get_mut(&knob_renderer_entity) {
                    if let Some(knob) = view.downcast_mut::<KnobViewRenderer>() {
                        knob.normalized_value = state.value_normalized;
                        cx.need_redraw();
                    }
                }
            })
        })
    }
}

impl<L> View for KnobView<L>
where
    L: Lens<Target = BoundVirtualSliderState>,
{
    fn element(&self) -> Option<&'static str> {
        Some("knobview")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        let (status, did_reset) = self.virtual_slider.on_event(cx, event);

        if let Some(vs_event) = status {
            (self.on_event)(cx, vs_event)
        }

        if did_reset {
            // Make sure that gesturing is stopped when the knob is reset
            // to its default value.
            (self.on_event)(cx, VirtualSliderEvent::GestureFinished);
        }
    }
}

#[derive(Debug, Clone)]
pub struct KnobViewStyle {
    pub angle_start: f32,
    pub angle_end: f32,

    pub background_color: Color,
    pub border_color: Color,
    pub border_width: f32,
    pub notch_color: Color,
    pub notch_width: f32,
    pub notch_line_cap: LineCap,
    pub notch_start: f32,
    pub notch_end: f32,

    pub arc_track_bg_color: Color,
    pub arc_track_filled_color: Color,
    pub arc_track_width: f32,
    pub arc_track_line_cap: LineCap,
    pub arc_track_offset: f32,
}

impl Default for KnobViewStyle {
    fn default() -> Self {
        Self {
            angle_start: -150.0,
            angle_end: 150.0,
            background_color: Color::rgb(0x44, 0x45, 0x45),
            border_color: Color::rgb(0x11, 0x11, 0x11),
            border_width: 1.5,
            notch_color: Color::rgb(0xff, 0xff, 0xff),
            notch_width: 2.0,
            notch_line_cap: LineCap::Butt,
            notch_start: 0.3,
            notch_end: 0.9,
            arc_track_bg_color: Color::rgb(0x33, 0x33, 0x33),
            arc_track_filled_color: Color::rgb(0xf0, 0x4f, 0x49),
            arc_track_width: 3.0,
            arc_track_line_cap: LineCap::Butt,
            arc_track_offset: 0.0,
        }
    }
}

struct KnobViewRenderer {
    normalized_value: f32,
    radius: Units,
    bipolar_mode: bool,
    style: KnobViewStyle,
}

impl KnobViewRenderer {
    pub fn new(
        cx: &mut Context,
        normalized_value: f32,
        radius: Units,
        bipolar_mode: bool,
        style: KnobViewStyle,
    ) -> Handle<Self> {
        Self { radius, normalized_value, bipolar_mode, style }.build(cx, |_| {})
    }
}

impl View for KnobViewRenderer {
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        use vizia::vg::{Paint, Path, Solidity};

        let bounds = cx.bounds();
        let scale_factor = cx.style.dpi_factor as f32;

        let center_x = bounds.x + (bounds.w / 2.0);
        let center_y = bounds.y + (bounds.h / 2.0);

        // Get the radius of the knob.
        let radius = self.radius.value_or(bounds.w / 2.0, 0.0) * scale_factor;

        // Calculate the angle of the notch/arc track.
        let notch_angle = (((self.style.angle_start - self.style.angle_end)
            * self.normalized_value)
            + self.style.angle_end
            + 180.0)
            .to_radians();
        let (notch_angle_x, notch_angle_y) = notch_angle.sin_cos();

        // -- Draw the arc track -------------------------------------------------------

        if self.style.arc_track_width != 0.0 {
            let arc_track_radius = radius
                + ((self.style.arc_track_offset + (self.style.arc_track_width / 2.0))
                    * scale_factor);

            let angle_start = (self.style.angle_start - 90.0).to_radians();
            let angle_end = (self.style.angle_end - 90.0).to_radians();

            let mut arc_bg_path = Path::new();
            arc_bg_path.arc(
                center_x,
                center_y,
                arc_track_radius,
                angle_start,
                angle_end,
                Solidity::Hole,
            );

            let mut arc_bg_paint = Paint::color(self.style.arc_track_bg_color);
            arc_bg_paint.set_line_width(self.style.arc_track_width * scale_factor);
            arc_bg_paint.set_line_cap(self.style.arc_track_line_cap);

            canvas.stroke_path(&mut arc_bg_path, &arc_bg_paint);

            if self.bipolar_mode {
                if self.normalized_value != 0.5 {
                    let mut arc_filled_paint = arc_bg_paint;
                    arc_filled_paint.set_color(self.style.arc_track_filled_color);

                    let arc_filled_center_angle = ((angle_end - angle_start) / 2.0) + angle_start;
                    let arc_filled_notch_angle =
                        ((angle_end - angle_start) * self.normalized_value) + angle_start;

                    let mut arc_filled_path = Path::new();

                    if self.normalized_value > 0.5 {
                        arc_filled_path.arc(
                            center_x,
                            center_y,
                            arc_track_radius,
                            arc_filled_center_angle,
                            arc_filled_notch_angle,
                            Solidity::Hole,
                        );
                    } else {
                        arc_filled_path.arc(
                            center_x,
                            center_y,
                            arc_track_radius,
                            arc_filled_notch_angle,
                            arc_filled_center_angle,
                            Solidity::Hole,
                        );
                    }

                    canvas.stroke_path(&mut arc_filled_path, &arc_filled_paint);
                }
            } else if self.normalized_value != 0.0 {
                let mut arc_filled_paint = arc_bg_paint;
                arc_filled_paint.set_color(self.style.arc_track_filled_color);

                let arc_filled_end_angle =
                    ((angle_end - angle_start) * self.normalized_value) + angle_start;

                let mut arc_filled_path = Path::new();
                arc_filled_path.arc(
                    center_x,
                    center_y,
                    arc_track_radius,
                    angle_start,
                    arc_filled_end_angle,
                    Solidity::Hole,
                );

                canvas.stroke_path(&mut arc_filled_path, &arc_filled_paint);
            }
        }

        // -- Draw the knob's background -----------------------------------------------

        // Subtract the border width from the background's radius.
        let bg_radius = radius - (self.style.border_width * scale_factor / 2.0);

        let mut bg_path = Path::new();
        bg_path.circle(center_x, center_y, bg_radius);

        let bg_fill_paint = Paint::color(self.style.background_color);
        canvas.fill_path(&mut bg_path, &bg_fill_paint);

        if self.style.border_width != 0.0 {
            let mut bg_stroke_paint = Paint::color(self.style.border_color);
            bg_stroke_paint.set_line_width(self.style.border_width * scale_factor);

            canvas.stroke_path(&mut bg_path, &bg_stroke_paint);
        }

        // -- Draw the knob's notch ----------------------------------------------------

        let notch_start_x = center_x + ((bg_radius * self.style.notch_start) * notch_angle_x);
        let notch_start_y = center_y + ((bg_radius * self.style.notch_start) * notch_angle_y);
        let notch_end_x = center_x + ((bg_radius * self.style.notch_end) * notch_angle_x);
        let notch_end_y = center_y + ((bg_radius * self.style.notch_end) * notch_angle_y);

        let mut notch_path = Path::new();
        notch_path.move_to(notch_start_x, notch_start_y);
        notch_path.line_to(notch_end_x, notch_end_y);

        let mut notch_paint = Paint::color(self.style.notch_color);
        notch_paint.set_line_width(self.style.notch_width * scale_factor);
        notch_paint.set_line_cap(self.style.notch_line_cap);

        canvas.stroke_path(&mut notch_path, &notch_paint);
    }
}
