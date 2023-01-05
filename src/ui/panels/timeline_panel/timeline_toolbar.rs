use vizia::prelude::*;

use crate::state_system::source_state::{SnapMode, TimelineTool};
use crate::state_system::{AppAction, StateSystem, TimelineAction, WorkingState};
use crate::ui::generic_views::{Icon, IconCode};

pub fn timeline_toolbar(cx: &mut Context) {
    const TOOLBAR_HEIGHT: f32 = 36.0;
    const TOOLBAR_CHILD_SPACE: f32 = 2.0;

    const TOOLBAR_GROUP_HEIGHT: f32 = 28.0;
    const SEPARATOR_PADDING: f32 = 9.0;
    const LABEL_LR_PADDING: f32 = 5.0;

    const ICON_FRAME_SIZE: f32 = 26.0;
    const ICON_SIZE: f32 = 25.0;
    const SMALL_ICON_FRAME_SIZE: f32 = 20.0;
    const SMALL_ICON_SIZE: f32 = 18.0;

    HStack::new(cx, |cx| {
        HStack::new(cx, |cx| {
            Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Menu, ICON_FRAME_SIZE, ICON_SIZE))
                .class("icon_btn");
        })
        .class("toolbar_group")
        .left(Pixels(SEPARATOR_PADDING))
        .height(Pixels(TOOLBAR_GROUP_HEIGHT))
        .width(Auto);

        Label::new(cx, "TIMELINE")
            .class("small_text")
            .left(Pixels(SEPARATOR_PADDING))
            .right(Pixels(SEPARATOR_PADDING))
            .top(Stretch(1.0))
            .bottom(Stretch(1.0));

        HStack::new(cx, |cx| {
            Button::new(
                cx,
                |cx| {
                    cx.emit(AppAction::Timeline(TimelineAction::SelectTool(TimelineTool::Pointer)))
                },
                |cx| Icon::new(cx, IconCode::Cursor, ICON_FRAME_SIZE, ICON_SIZE),
            )
            .class("icon_btn")
            .toggle_class(
                "icon_btn_accent_toggled",
                StateSystem::working_state
                    .then(WorkingState::selected_timeline_tool)
                    .map(|t| *t == TimelineTool::Pointer),
            );

            Element::new(cx).class("toolbar_group_separator");

            Button::new(
                cx,
                |cx| cx.emit(AppAction::Timeline(TimelineAction::SelectTool(TimelineTool::Pencil))),
                // TODO: Fix icon size
                |cx| Icon::new(cx, IconCode::Pencil, ICON_FRAME_SIZE, 16.0),
            )
            .class("icon_btn")
            .toggle_class(
                "icon_btn_accent_toggled",
                StateSystem::working_state
                    .then(WorkingState::selected_timeline_tool)
                    .map(|t| *t == TimelineTool::Pencil),
            );

            Element::new(cx).class("toolbar_group_separator");

            Button::new(
                cx,
                |cx| cx.emit(AppAction::Timeline(TimelineAction::SelectTool(TimelineTool::Slicer))),
                // TODO: Fix icon size
                |cx| Icon::new(cx, IconCode::Slice, ICON_FRAME_SIZE, 18.0),
            )
            .class("icon_btn")
            .toggle_class(
                "icon_btn_accent_toggled",
                StateSystem::working_state
                    .then(WorkingState::selected_timeline_tool)
                    .map(|t| *t == TimelineTool::Slicer),
            );

            Element::new(cx).class("toolbar_group_separator");

            Button::new(
                cx,
                |cx| cx.emit(AppAction::Timeline(TimelineAction::SelectTool(TimelineTool::Eraser))),
                // TODO: Fix icon size
                |cx| Icon::new(cx, IconCode::Eraser, ICON_FRAME_SIZE, 20.0),
            )
            .class("icon_btn")
            .toggle_class(
                "icon_btn_accent_toggled",
                StateSystem::working_state
                    .then(WorkingState::selected_timeline_tool)
                    .map(|t| *t == TimelineTool::Eraser),
            );
        })
        .class("toolbar_group")
        .left(Pixels(SEPARATOR_PADDING * 2.0))
        .height(Pixels(TOOLBAR_GROUP_HEIGHT))
        .width(Auto);

        HStack::new(cx, |cx| {
            Button::new(
                cx,
                |cx| {
                    cx.emit(AppAction::Timeline(TimelineAction::SetSnapActive(
                        !StateSystem::working_state
                            .then(WorkingState::timeline_snap_active)
                            .get(cx),
                    )))
                },
                // TODO: Fix icon size
                |cx| Icon::new(cx, IconCode::Magnet, ICON_FRAME_SIZE, 16.0),
            )
            .class("icon_btn")
            .toggle_class(
                "icon_btn_accent_toggled",
                StateSystem::working_state.then(WorkingState::timeline_snap_active),
            );

            Element::new(cx).class("toolbar_group_separator");

            MenuController::new(cx, false, |cx| {
                MenuStack::new_horizontal(cx, |cx| {
                    Menu::new(
                        cx,
                        |cx| {
                            HStack::new(cx, |cx| {
                                Label::new(
                                    cx,
                                    StateSystem::working_state
                                        .then(WorkingState::timeline_snap_mode)
                                        .map(|m| m.to_text()),
                                )
                                .top(Stretch(1.0))
                                .bottom(Stretch(1.0))
                                .left(Pixels(LABEL_LR_PADDING));
                                Icon::new(
                                    cx,
                                    IconCode::DropdownArrow,
                                    SMALL_ICON_FRAME_SIZE,
                                    SMALL_ICON_SIZE,
                                )
                                .top(Stretch(0.55))
                                .bottom(Stretch(0.45));
                            })
                            .child_left(Pixels(LABEL_LR_PADDING))
                            .child_right(Pixels(LABEL_LR_PADDING))
                        },
                        |cx| {
                            MenuButton::new_simple(cx, "Line", |cx| {
                                cx.emit(AppAction::Timeline(TimelineAction::SetSnapMode(
                                    SnapMode::Line,
                                )))
                            });
                            MenuButton::new_simple(cx, "Beat", |cx| {
                                cx.emit(AppAction::Timeline(TimelineAction::SetSnapMode(
                                    SnapMode::Beat,
                                )))
                            });
                            MenuButton::new_simple(cx, "1/2 Beat", |cx| {
                                cx.emit(AppAction::Timeline(TimelineAction::SetSnapMode(
                                    SnapMode::HalfBeat,
                                )))
                            });
                            MenuButton::new_simple(cx, "1/3 Beat", |cx| {
                                cx.emit(AppAction::Timeline(TimelineAction::SetSnapMode(
                                    SnapMode::ThirdBeat,
                                )))
                            });
                            MenuButton::new_simple(cx, "1/4 Beat", |cx| {
                                cx.emit(AppAction::Timeline(TimelineAction::SetSnapMode(
                                    SnapMode::QuarterBeat,
                                )))
                            });
                            MenuButton::new_simple(cx, "1/8 Beat", |cx| {
                                cx.emit(AppAction::Timeline(TimelineAction::SetSnapMode(
                                    SnapMode::EigthBeat,
                                )))
                            });
                            MenuButton::new_simple(cx, "1/16 Beat", |cx| {
                                cx.emit(AppAction::Timeline(TimelineAction::SetSnapMode(
                                    SnapMode::SixteenthBeat,
                                )))
                            });
                            MenuButton::new_simple(cx, "1/32 Beat", |cx| {
                                cx.emit(AppAction::Timeline(TimelineAction::SetSnapMode(
                                    SnapMode::_32ndBeat,
                                )))
                            });
                        },
                    );
                });
            });
        })
        .left(Pixels(SEPARATOR_PADDING))
        .height(Pixels(TOOLBAR_GROUP_HEIGHT))
        .class("toolbar_group")
        .width(Auto);

        HStack::new(cx, |cx| {
            Button::new(
                cx,
                |cx| cx.emit(AppAction::Timeline(TimelineAction::ZoomIn)),
                // TODO: Fix icon size
                |cx| Icon::new(cx, IconCode::ZoomIn, ICON_FRAME_SIZE, 16.0),
            )
            .class("icon_btn");

            Element::new(cx).class("toolbar_group_separator");

            Button::new(
                cx,
                |cx| cx.emit(AppAction::Timeline(TimelineAction::ZoomOut)),
                // TODO: Fix icon size
                |cx| Icon::new(cx, IconCode::ZoomOut, ICON_FRAME_SIZE, 16.0),
            )
            .class("icon_btn");

            Element::new(cx).class("toolbar_group_separator");

            Button::new(
                cx,
                |cx| cx.emit(AppAction::Timeline(TimelineAction::ZoomReset)),
                // TODO: Fix icon size
                |cx| Icon::new(cx, IconCode::ZoomReset, ICON_FRAME_SIZE, 16.0),
            )
            .class("icon_btn");
        })
        .class("toolbar_group")
        .left(Pixels(SEPARATOR_PADDING))
        .height(Pixels(TOOLBAR_GROUP_HEIGHT))
        .width(Auto);
    })
    .height(Pixels(TOOLBAR_HEIGHT))
    .child_space(Pixels(TOOLBAR_CHILD_SPACE))
    .class("timeline_toolbar");
}
