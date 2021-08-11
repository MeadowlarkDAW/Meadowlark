use tuix::widgets::*;
use tuix::style::*;

use crate::ui::AppData;
use crate::ui::TempoEvent;

#[derive(Default)]
pub struct ControlBar {

}

impl Widget for ControlBar {
    type Ret = Entity;
    type Data = ();
    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        
        Label::new("TEMPO").build(state, entity, |builder| builder);

        let controls = Row::new().build(state, entity, |builder| builder.class("controls"));

        Textbox::new("130")
            .on_submit(|data, state, textbox|{
                if let Ok(bpm) = data.text.parse::<i32>() {
                    textbox.emit(state, TempoEvent::SetBPM(bpm));
                } else {
                    // TODO - need better error handling/ fallback here
                    data.text = "130".to_string();
                }
                
            })
            .bind(AppData::beats_per_minute, |value| value.to_string())
            .build(state, controls, |builder| builder);

        Button::with_label("TAP").build(state, controls, |builder| builder);
        Button::with_label("4/4").build(state, controls, |builder| builder);
        Button::with_label("GROOVE").build(state, controls, |builder| 
            builder
                .set_disabled(true)
        );

        
        entity.class(state, "control_bar")
    }
}