use vizia::*;

use super::timeline_view::TimelineViewState;

use femtovg::{Align, Baseline, Paint, Path};
pub struct TimelineGrid {}

impl TimelineGrid {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self {}.build2(cx, |_cx| {})
    }
}

impl View for TimelineGrid {
    fn draw(&self, cx: &mut Context, canvas: &mut Canvas) {
        if let Some(timeline_view) = cx.data::<TimelineViewState>() {
            let start_time = timeline_view.start_time.as_beats_f64();
            let end_time = timeline_view.end_time.as_beats_f64();
            //let timeline_start = timeline_view.timeline_start.as_beats_f64();
            //let timeline_end = timeline_view.timeline_end.as_beats_f64();
            let timeline_width = timeline_view.width;

            let bounds = cx.cache.get_bounds(cx.current);

            let pixels_per_beat = timeline_width / (end_time - start_time) as f32;

            //println!("Pixels per beat: {}", pixels_per_beat);

            let font = cx.style.font.get(cx.current).cloned().unwrap_or_default();

            let default_font = cx
                .resource_manager
                .fonts
                .get(&cx.style.default_font)
                .and_then(|font| match font {
                    FontOrId::Id(id) => Some(id),
                    _ => None,
                })
                .expect("Failed to find default font");

            let font_id = cx
                .resource_manager
                .fonts
                .get(&font)
                .and_then(|font| match font {
                    FontOrId::Id(id) => Some(id),
                    _ => None,
                })
                .unwrap_or(default_font);

            canvas.save();

            canvas.scissor(bounds.x, bounds.y, bounds.w, bounds.h);

            for i in (start_time.floor() as usize)..(end_time.ceil() as usize) {
                let ratio = (i as f64 - start_time) / (end_time - start_time);
                let mut path = Path::new();
                path.move_to(bounds.x + (ratio as f32 * timeline_width).floor(), bounds.y);
                path.line_to(
                    bounds.x + (ratio as f32 * timeline_width).floor(),
                    bounds.y + bounds.h,
                );
                canvas.stroke_path(&mut path, Paint::color(femtovg::Color::rgb(36, 36, 36)));
                let mut text_paint = Paint::color(femtovg::Color::rgb(255, 255, 255));
                text_paint.set_font(&[font_id.clone()]);
                text_paint.set_text_align(Align::Left);
                text_paint.set_text_baseline(Baseline::Top);
                canvas.fill_text(
                    bounds.x + (ratio as f32 * timeline_width).floor() + 2.0,
                    bounds.y,
                    &(i + 1).to_string(),
                    text_paint,
                );

                if pixels_per_beat >= 100.0 && pixels_per_beat < 400.0 {
                    for j in 1..4 {
                        let ratio =
                            (i as f64 + j as f64 * 0.25 - start_time) / (end_time - start_time);
                        let mut path = Path::new();
                        path.move_to(bounds.x + (ratio as f32 * timeline_width).floor(), bounds.y);
                        path.line_to(
                            bounds.x + (ratio as f32 * timeline_width).floor(),
                            bounds.y + bounds.h,
                        );
                        canvas
                            .stroke_path(&mut path, Paint::color(femtovg::Color::rgb(46, 46, 46)));
                    }
                }

                if pixels_per_beat >= 300.0 {
                    for j in 1..4 {
                        let ratio =
                            (i as f64 + j as f64 * 0.25 - start_time) / (end_time - start_time);
                        let mut text_paint = Paint::color(femtovg::Color::rgb(255, 255, 255));
                        text_paint.set_font(&[font_id.clone()]);
                        text_paint.set_text_align(Align::Left);
                        text_paint.set_text_baseline(Baseline::Top);
                        canvas.fill_text(
                            bounds.x + (ratio as f32 * timeline_width).floor() + 2.0,
                            bounds.y,
                            &format!("{}.{}", i + 1, j + 1),
                            text_paint,
                        );
                    }
                }

                if pixels_per_beat >= 400.0 {
                    for j in 1..16 {
                        let ratio =
                            (i as f64 + j as f64 * 0.0625 - start_time) / (end_time - start_time);
                        let mut path = Path::new();
                        path.move_to(bounds.x + (ratio as f32 * timeline_width).floor(), bounds.y);
                        path.line_to(
                            bounds.x + (ratio as f32 * timeline_width).floor(),
                            bounds.y + bounds.h,
                        );
                        if j % 4 == 0 {
                            canvas.stroke_path(
                                &mut path,
                                Paint::color(femtovg::Color::rgb(46, 46, 46)),
                            );
                        } else {
                            canvas.stroke_path(
                                &mut path,
                                Paint::color(femtovg::Color::rgb(56, 56, 56)),
                            );
                        }
                    }
                }
            }

            canvas.restore();
        }
    }
}
