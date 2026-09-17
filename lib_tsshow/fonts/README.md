# Fonts

These fonts are rendered into the SVGs created by this crate.
Each SVG embeds them as `@font-face` rules, reduced to the characters it actually renders,
so that it looks the same everywhere, even where the fonts are not installed.

| File | Rendered with it | Licence |
| --- | --- | --- |
| `LiberationMono-Regular.ttf` | the sequences of the template switch arrangement | [SIL Open Font License 1.1](LICENSE-LiberationMono.txt) |
| `DejaVuSansMono.ttf` | labels and numbers | [Bitstream Vera Fonts Copyright](LICENSE-DejaVu.txt) |
| `DejaVuSans.ttf` | free text, such as the names of the sequences | [Bitstream Vera Fonts Copyright](LICENSE-DejaVu.txt) |

Liberation Mono is metric-compatible with Courier New,
which was used to draw the characters before they were rendered as text.

The copyright and licence notices declared by the font files themselves
are copied into every SVG next to the font that they belong to.
