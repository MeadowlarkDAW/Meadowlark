use vizia::prelude::*;

use super::icons;

pub fn build(cx: &mut Context) {
    HStack::new(cx, |cx| {
        VStack::new(cx, |cx| {
            Label::new(cx, Localized::new("project_section_label")).class("top_bar_section_label");

            Spacer::new(cx).height(Stretch(1.0));

            HStack::new(cx, |cx| {
                Button::new(cx, |cx| Svg::new(cx, icons::ICON_MENU));

                Divider::new(cx);

                Button::new(cx, |cx| Svg::new(cx, icons::ICON_SAVE));
                Button::new(cx, |cx| Svg::new(cx, icons::ICON_SAVE_AS));

                Divider::new(cx);

                Button::new(cx, |cx| Svg::new(cx, icons::ICON_UNDO)).disabled(true);
                Button::new(cx, |cx| Svg::new(cx, icons::ICON_REDO)).disabled(true);
            })
            .width(Auto)
            .height(Pixels(24.0))
            .gap(Pixels(6.0))
            .alignment(Alignment::Left);
        })
        .width(Auto)
        .class("top_bar_section");

        Spacer::new(cx).width(Stretch(1.0));

        Divider::new(cx);

        VStack::new(cx, |cx| {
            Label::new(cx, Localized::new("view_section_label")).class("top_bar_section_label");

            Spacer::new(cx).height(Stretch(1.0));

            HStack::new(cx, |cx| {
                Button::new(cx, |cx| Svg::new(cx, icons::ICON_BROWSER));
                Button::new(cx, |cx| Svg::new(cx, icons::ICON_TRACKS));
                Button::new(cx, |cx| Svg::new(cx, icons::ICON_CLIPS_PANEL));
                Button::new(cx, |cx| Svg::new(cx, icons::ICON_CLIP_LAUNCHER)).disabled(true);
                Button::new(cx, |cx| Svg::new(cx, icons::ICON_MIXER));
                Button::new(cx, |cx| Svg::new(cx, icons::ICON_PIANO_ROLL));
                Button::new(cx, |cx| Svg::new(cx, icons::ICON_AUTOMATION));
                Button::new(cx, |cx| Svg::new(cx, icons::ICON_AUDIO_CLIP_EDITOR));
                Button::new(cx, |cx| Svg::new(cx, icons::ICON_COMMAND_PALETTE));
            })
            .height(Pixels(24.0))
            .gap(Pixels(6.0))
            .alignment(Alignment::Left);
        })
        .class("top_bar_section");

        Spacer::new(cx).width(Stretch(1.0));

        Divider::new(cx);

        VStack::new(cx, |cx| {
            Label::new(cx, Localized::new("record_section_label")).class("top_bar_section_label");

            Spacer::new(cx).height(Stretch(1.0));

            HStack::new(cx, |cx| {
                Button::new(cx, |cx| Svg::new(cx, icons::ICON_RECORD));

                Divider::new(cx);

                Button::new(cx, |cx| Svg::new(cx, icons::ICON_METRONOME));

                Divider::new(cx);
            })
            .height(Pixels(24.0))
            .gap(Pixels(6.0))
            .alignment(Alignment::Left);
        })
        .class("top_bar_section");

        Spacer::new(cx).width(Stretch(1.0));

        Divider::new(cx);

        VStack::new(cx, |cx| {
            Label::new(cx, Localized::new("transport_section_label")).class("top_bar_section_label");

            Spacer::new(cx).height(Stretch(1.0));

            HStack::new(cx, |cx| {
            })
            .height(Pixels(24.0))
            .gap(Pixels(6.0))
            .alignment(Alignment::Left);
        })
        .class("top_bar_section");

        Spacer::new(cx).width(Stretch(1.0));

        Divider::new(cx);

        VStack::new(cx, |cx| {
            Label::new(cx, Localized::new("tempo_section_label")).class("top_bar_section_label");

            Spacer::new(cx).height(Stretch(1.0));

            HStack::new(cx, |cx| {
            })
            .height(Pixels(24.0))
            .gap(Pixels(6.0))
            .alignment(Alignment::Left);
        })
        .class("top_bar_section");

        Spacer::new(cx).width(Stretch(1.0));

        Divider::new(cx);

        VStack::new(cx, |cx| {
            Label::new(cx, Localized::new("monitor_section_label")).class("top_bar_section_label");

            Spacer::new(cx).height(Stretch(1.0));

            HStack::new(cx, |cx| {
            })
            .height(Pixels(24.0))
            .gap(Pixels(6.0))
            .alignment(Alignment::Left);
        })
        .class("top_bar_section");
    })
    .min_gap(Pixels(2.0))
    .alignment(Alignment::Left)
    .width(Percentage(100.0))
    .height(Pixels(50.0))
    .class("top_bar");
}
