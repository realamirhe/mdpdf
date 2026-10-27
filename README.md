# mdpdf


Convert Markdown notes into PDFs locally, with Persian and English typography.

Built for notes exported from Bear: a small tool for anyone who does not want to
buy Bear Pro just to generate PDFs. It also works with standalone Markdown files.
Conversion runs offline, without a browser or a background service.

<img width="574" height="474" alt="image" src="https://github.com/user-attachments/assets/1dfb8a7e-2e75-4093-a40f-590baa31a04a" />

## Getting started

The bundled Typst executable supports **macOS on Apple Silicon**. Other platforms
need a compatible Typst executable and have not been verified with this bundle.

You need Rust and Cargo to build the CLI. Install Vazirmatn for Persian text and
Cascadia Code for Latin text and code, or choose installed fonts in
[`settings.json`](settings.json). Fonts are not included. The configured Persian
font is required even when the input is entirely English.

From the project directory:

```sh
cargo build --release --locked
cp target/release/mdpdf ./mdpdf
./mdpdf examples/sample.md
```

This creates `examples/sample.pdf`. Keep `mdpdf`, `settings.json`, `template.typ`,
and `vendor/` together: the CLI reads its resources beside the executable.
Running the binary directly from `target/release/` will not find those resources.
Building may download Rust dependencies; conversion itself needs no network.

For Bear notes, export Markdown and keep the exported attachments in their
original relative locations. On macOS, you can then double-click
`mdpdf.command` to choose a note and a theme. The launcher opens the resulting PDF.

## Usage

```sh
./mdpdf "/path/to/note.md"
./mdpdf "/path/to/note.md" --theme classic
./mdpdf "/path/to/note.md" -o "/path/to/output.pdf"
./mdpdf "/path/to/note.md" --force --open
./mdpdf "/path/to/note.md" --config "/path/to/settings.json"
./mdpdf --help
```

The default theme is `editorial`. Output goes beside the Markdown file unless
`-o` is supplied. Existing output is preserved unless you pass `--force`.
The Markdown source is never rewritten.

## Typography and layout

Both themes use A4 pages, syntax-highlighted code, and framed images. Paragraph
direction is detected from the text; code stays left-to-right. Persian
zero-width non-joiners are preserved, with optional visual spacing in prose.

Edit [`settings.json`](settings.json) to change the defaults:

| Setting | Purpose | Default |
| --- | --- | --- |
| `persian_font` | Required Persian font family | `Vazirmatn` |
| `latin_fonts`, `mono_fonts` | Font families in preference order | Cascadia Code, Fira Code, DejaVu Sans Mono |
| `font_paths` | Additional font directories | `~/Library/Fonts` |
| `font_size_pt` | Body text size | `11.5` |
| `line_height` | Relative line spacing | `1.7` |
| `page_margin_mm` | Page margins | `18` |
| `zwnj_spacing_em` | Extra visual spacing at Persian non-joiners | `0.1` |

Unavailable Latin and monospace fonts are filtered out; at least one configured
font in each list must be available. Set `zwnj_spacing_em` to `0` to disable the
extra spacing. It never applies to inline or fenced code. Fonts are embedded in
the PDF, so readers do not need to install them.

For layout changes, edit [`template.typ`](template.typ).

## Images and supported Markdown

Headings, emphasis, lists, tables, quotes, links, and fenced code are supported.
Images can use relative paths, absolute paths, local `file://` URLs, or base64
data URIs. Basic HTML `<img src="...">` tags are also supported.

```md
![Photo](assets/photo.png)
![Filename with spaces](assets/my%20photo.png)
![Absolute path](</path/to/my photo.png>)
![Local file](file:///path/to/photo.png)
```

Relative image paths resolve from the Markdown file's directory. Missing images
and malformed base64 data fail conversion without publishing a partial PDF.
Images fit within the page width and are capped at 110 mm high.

HTTP/HTTPS images must be downloaded first. Mermaid, LaTeX math, Bear-specific
internal links, and full HTML/CSS rendering are not implemented. Direction
detection can be ambiguous in mixed text. Long code lines wrap in the PDF, and
those visual line breaks may appear when copied from it.

## Development

Tests use Rust and Cargo; Python is not required.

Enable local tests before each push:

```sh
git config --local core.hooksPath .githooks
```

The pre-push hook runs the full test suite, including PDF integration tests, and
blocks the push on failure. It requires the bundled Typst executable and fonts
described above. No GitHub CI is configured.

To run individual checks manually:

```sh
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
```

PDF integration tests require the bundled Typst executable and the configured
fonts. Run them explicitly on a supported machine:

```sh
cargo test --locked -- --ignored
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for test coverage and contribution guidance.

## Dependencies

The rendering bundle includes Typst 0.15.1, cmarker 0.1.10, auto-bidi 0.1.0,
Codly 2.0.0, and Elembic 1.1.1. Upstream license files and notices remain in
`vendor/`. See [THIRD_PARTY.md](THIRD_PARTY.md) for their locations.

## License

The project's original code and documentation are licensed under the
[MIT License](LICENSE). Bundled dependencies retain their own licenses, listed in
[THIRD_PARTY.md](THIRD_PARTY.md).
