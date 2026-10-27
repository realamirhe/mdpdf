# A note worth keeping

Markdown keeps notes easy to edit. A PDF makes them easy to share or print.

## From note to document

1. Export a note from Bear as Markdown.
2. Keep any image attachments beside the Markdown file.
3. Run `./mdpdf note.md` to create a PDF in the same folder.

> Keep the source simple. Let the template handle the layout.

## A small example

```rust
fn main() {
    println!("A note, ready to share.");
}
```

| Theme | Appearance |
| --- | --- |
| Editorial | Purple accents and an underlined title |
| Classic | Blue-gray accents and a plain title |

The source file stays unchanged. Existing PDFs are preserved unless you pass
`--force`.
