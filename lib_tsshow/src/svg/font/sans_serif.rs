use std::sync::LazyLock;

use super::Font;

/// The font used to render free text, such as the names of the sequences.
pub static FONT: LazyLock<Font> =
    LazyLock::new(|| Font::new_proportional(include_bytes!("../../../fonts/DejaVuSans.ttf"), 3.0));
