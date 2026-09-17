use std::sync::LazyLock;

use super::Font;

/// The font used to render the sequences of the template switch arrangement.
///
/// [Liberation Mono](https://github.com/liberationfonts) is metric-compatible with Courier New,
/// which was used to draw the characters before they were rendered as text.
pub static FONT: LazyLock<Font> = LazyLock::new(|| {
    Font::new_monospace(
        include_bytes!("../../../fonts/LiberationMono-Regular.ttf"),
        2.0,
        3.175,
        1.0,
    )
});
