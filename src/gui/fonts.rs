use std::sync::Arc;
use yarrow::{prelude::ResourceCtx, vg::text::glyphon::fontdb};

static INTER_REGULAR: &[u8] = include_bytes!("../../assets/fonts/inter/Inter-Regular.otf");
static INTER_ITALIC: &[u8] = include_bytes!("../../assets/fonts/inter/Inter-Italic.otf");
static INTER_BOLD: &[u8] = include_bytes!("../../assets/fonts/inter/Inter-Bold.otf");
static FIRA_MONO: &[u8] = include_bytes!("../../assets/fonts/fira-mono/FiraMono-Regular.otf");

pub fn load_fonts(res: &mut ResourceCtx) {
    let db = res.font_system.db_mut();

    db.load_font_source(fontdb::Source::Binary(Arc::new(INTER_REGULAR)));
    db.load_font_source(fontdb::Source::Binary(Arc::new(INTER_ITALIC)));
    db.load_font_source(fontdb::Source::Binary(Arc::new(INTER_BOLD)));
    db.load_font_source(fontdb::Source::Binary(Arc::new(FIRA_MONO)));

    db.set_sans_serif_family("Inter");
    db.set_monospace_family("Fira Mono");
}
