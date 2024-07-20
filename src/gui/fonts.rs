use std::sync::Arc;
use yarrow::{prelude::ResourceCtx, vg::text::glyphon::fontdb};

static FIRA_SANS_REGULAR: &[u8] =
    include_bytes!("../../assets/fonts/fira-sans/FiraSans-Regular.ttf");
static FIRA_SANS_CONDENSED_REGULAR: &[u8] =
    include_bytes!("../../assets/fonts/fira-sans-condensed/firasanscondensed-regular.otf");
static FIRA_MONO_REGULAR: &[u8] =
    include_bytes!("../../assets/fonts/fira-mono/FiraMono-Regular.otf");

pub fn font_sources() -> Vec<fontdb::Source> {
    vec![
        fontdb::Source::Binary(Arc::new(FIRA_SANS_REGULAR)),
        fontdb::Source::Binary(Arc::new(FIRA_SANS_CONDENSED_REGULAR)),
        fontdb::Source::Binary(Arc::new(FIRA_MONO_REGULAR)),
    ]
}

pub fn load_fonts(res: &mut ResourceCtx) {
    let db = res.font_system.db_mut();

    db.set_sans_serif_family("Fira Sans");
    db.set_monospace_family("Fira Mono");
}
