use std::sync::LazyLock;

use super::Font;

/// The font used to render labels and numbers.
pub static FONT: LazyLock<Font> = LazyLock::new(|| {
    Font::new_monospace(
        include_bytes!("../../../fonts/DejaVuSansMono.ttf"),
        2.0,
        3.175,
        0.75,
    )
});
