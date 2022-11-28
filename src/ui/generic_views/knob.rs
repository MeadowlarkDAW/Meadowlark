use vizia::prelude::*;

use super::virtual_slider::{
    BoundVirtualSliderState, VirtualSlider, VirtualSliderDirection, VirtualSliderEvent,
    VirtualSliderMode, VirtualSliderScalars,
};

pub struct KnobView<L: Lens<Target = BoundVirtualSliderState>> {
    virtual_slider: VirtualSlider<L>,

    on_changing: Option<Box<dyn Fn(&mut EventContext, VirtualSliderEvent)>>,
}

impl<L: Lens<Target = BoundVirtualSliderState>> KnobView<L> {
    pub fn new(
        cx: &mut Context,
        lens: L,
        mode: VirtualSliderMode,
        direction: VirtualSliderDirection,
        scalars: VirtualSliderScalars,
        centered: bool,
        on_changing: Option<Box<dyn Fn(&mut EventContext, VirtualSliderEvent)>>,
    ) -> Handle<Self> {
        Self {
            virtual_slider: VirtualSlider::new(cx, lens.clone(), mode, direction, scalars),
            on_changing,
        }
        .build(cx, |cx| Binding::new(cx, lens, move |cx, state| {}))
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
        if let Some(vs_event) = self.virtual_slider.on_event(cx, event) {
            if let Some(on_changing) = &mut self.on_changing {
                (on_changing)(cx, vs_event);
            }
        }
    }
}

struct KnobViewRenderer {}
