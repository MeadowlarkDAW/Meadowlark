use vizia::prelude::*;

pub mod icons;
pub mod top_bar;

pub fn run() -> anyhow::Result<()> {
    Application::new(|cx| {
        add_resources(cx);

        top_bar::build(cx);
    })
    .title(if cfg!(debug_assertions) {
        "Meadowlark [DEBUG]"
    } else {
        "Meadowlark"
    })
    .inner_size((1920, 1000))
    // TODO: App icon doesn't seem to work, at least on Wayland.
    .icon(256, 256, {
        image::load_from_memory_with_format(
            include_bytes!("resources/icons/logo-meadowlark-256.png"),
            image::ImageFormat::Png,
        )
        .unwrap()
        .into_rgba8()
        .into_raw()
    })
    .ignore_default_theme()
    .run()?;

    Ok(())
}

fn add_resources(cx: &mut Context) {
    cx.add_font_mem(include_bytes!("resources/fonts/inter/Inter-Regular.otf"));
    cx.add_font_mem(include_bytes!(
        "resources/fonts/fira-mono/FiraMono-Regular.otf"
    ));

    cx.add_stylesheet(include_style!("src/ui/default_dark.css"))
        .unwrap();

    cx.add_translation(
        langid!("en-US"),
        include_str!("resources/localization/en-US.ftl").to_owned(),
    );
}
