#import "vendor/cmarker/lib.typ": render
#import "vendor/codly/codly.typ" as codly
#import "vendor/auto-bidi/lib.typ": auto-dir
#let cfg = json("settings.json")
#let assets = json("assets.json")
#let zwnj-gap = cfg.at("zwnj_spacing_em", default: .10)
#assert(type(zwnj-gap) in (int, float) and zwnj-gap >= 0 and zwnj-gap <= .5, message: "zwnj_spacing_em must be between 0 and 0.5")
#let persian-zwnj = regex("[\\p{Arabic}\\p{M}\\x{200c}\\x{200d}]+")
#let space-persian-zwnj(token, gap: .10) = {
  if gap == 0 or not token.text.contains("\u{200c}") { token } else {
    // Keep the compound word unbroken, as with the original zero-width non-joiner.
    box(token.text.split("\u{200c}").map(part => text(part)).join(text("\u{200c}") + h(gap * 1em)))
  }
}
#let literal-text = state("mdpdf-literal-text", false)
#let editorial = cfg.theme == "editorial"
#let accent = if editorial { rgb("5945c6") } else { rgb("28596a") }
#let soft = if editorial { rgb("f5f5f6") } else { rgb("f2f5f7") }
#let border = if editorial { rgb("d7d7dc") } else { rgb("d3dce1") }
// Use a subtle hatch around square image and code panels.
#let hatch = tiling(size: (6pt, 6pt))[
  #line(start: (0pt, 6pt), end: (6pt, 0pt), stroke: .35pt + rgb("e5e5e8"))
]
#let linear-panel(body, inset: 12pt) = block(
  width: 100%, above: .8em, below: .8em,
  radius: 0pt, fill: hatch, stroke: .45pt + border,
  inset: (x: 8pt, y: 7pt),
  block(width: 100%, radius: 0pt, fill: soft, stroke: .45pt + border, inset: inset, body),
)
#let syntax-theme = bytes(```xml
<?xml version="1.0"?><plist version="1.0"><dict><key>settings</key><array>
<dict><key>settings</key><dict><key>foreground</key><string>#303044</string></dict></dict>
<dict><key>scope</key><string>comment</string><key>settings</key><dict><key>foreground</key><string>#858592</string></dict></dict>
<dict><key>scope</key><string>keyword, storage</string><key>settings</key><dict><key>foreground</key><string>#b215da</string></dict></dict>
<dict><key>scope</key><string>string</string><key>settings</key><dict><key>foreground</key><string>#0081e8</string></dict></dict>
<dict><key>scope</key><string>constant, support, entity.name</string><key>settings</key><dict><key>foreground</key><string>#6445ff</string></dict></dict>
</array></dict></plist>
```.text)
#set document(author: "", title: "")
#set text(size: cfg.font_size_pt * 1pt, lang: "fa", fill: rgb("292625"))
#set page(
  paper: "a4",
  margin: cfg.page_margin_mm * 1mm,
  footer: context align(center, text(size: 8pt, fill: rgb("88817a"), counter(page).display("1"))),
  footer-descent: 7mm,
)
#set par(justify: false, leading: (cfg.line_height - 1) * 1em, spacing: .85em)
#set heading(numbering: none)
#set list(tight: false, indent: 1.1em, body-indent: .5em, spacing: .35em)
#set enum(tight: false, indent: 1.1em, body-indent: .5em, spacing: .35em)
#set table(inset: 7pt, stroke: (top: none, left: none, right: none, bottom: .5pt + border), fill: (x, y) => if y == 0 { soft } else { none })
#show table.cell.where(y: 0): strong
#show table: t => {
  if t.columns.all(c => c == auto) {
    table(columns: (1fr,) * t.columns.len(), align: t.align, ..t.children)
  } else { t }
}
#show link: set text(fill: accent)
#show heading: it => {
  set text(fill: if it.level == 1 { rgb("292625") } else { accent })
  let size = if it.level == 1 { 1.95em } else if it.level == 2 { 1.35em } else { 1.12em }
  block(above: if it.level == 1 { .5em } else { 1.1em }, below: .6em, sticky: true)[
    #text(size: size, weight: "bold", it.body)
    #if editorial and it.level == 1 {
      v(.3em)
      line(length: 100%, stroke: 1pt + accent)
    }
  ]
}
#show quote.where(block: true): it => block(
  width: 100%, inset: 10pt, fill: soft, radius: 0pt,
  stroke: (right: 2pt + accent), above: .75em, below: .75em,
  it.body,
)
#show: codly.line-set_(fill: none, stroke: none, radius: 0pt, inset: (x: 0pt, y: 3pt))
#show raw: it => {
  // Scope the prose rule out of literal code, including inline raw text.
  literal-text.update(true)
  set text(font: ((name: cfg.persian_font, covers: regex("[\\p{Arabic}\\x{200C}\\x{200D}]")),) + cfg.mono_fonts + (cfg.persian_font,), lang: "en", dir: ltr, size: .93em, ligatures: false)
  if it.block {
    set text(size: 1.1em / .93)
    set par(leading: .55em, justify: false)
    linear-panel(layout(size => {
      // Compensate for the nested raw element’s default 0.8em scaling.
      set text(size: 1em / .8)
      let glyph-width = measure(text(font: cfg.mono_fonts, size: cfg.font_size_pt * 1.1 * .8 * 1pt, "M")).width
      let columns = calc.max(12, calc.floor((size.width - 30pt) / glyph-width))
      let wrapped = it.text.split("\n").map(line => {
        let chars = line.clusters()
        if chars.len() <= columns { line } else {
          range(0, chars.len(), step: columns).map(start => chars.slice(start, calc.min(start + columns, chars.len())).join("")).join("\n")
        }
      }).join("\n")
      // Reflow only the rendered copy; preserve the original Markdown.
      show raw: x => x
      codly.new(raw(wrapped, lang: it.lang, block: true, theme: syntax-theme), gutters: (), padding: 0pt, radius: 0pt, lang-position: none)
    }))
  } else {
    box(fill: soft, radius: 0pt, inset: (x: 3pt, y: 1pt), it)
  }
  literal-text.update(false)
}
#let resolved-image(path, alt: none, ..args) = {
  let data = if type(path) == bytes { path } else { assets.at(path) }
  linear-panel(inset: 8pt, layout(size => {
    let named = args.named()
    if "width" in named { named.remove("width") }
    let full = image(data, alt: alt, width: size.width, ..named)
    let dimensions = measure(full)
    if dimensions.height > 110mm {
      image(data, alt: alt, width: size.width, height: 110mm, fit: "contain")
    } else { full }
  }))
}
#show: auto-dir.with(
  arab-font: cfg.persian_font,
  hebrew-font: cfg.persian_font,
  english-font: cfg.latin_fonts,
  // Neutral characters, especially ZWNJ/ZWJ, must share the Persian font.
  base-font: cfg.persian_font,
  arabic-script-lang: "fa",
  detect-by: "auto",
  default-lang: "fa",
)
// This changes layout only; U+200C and surrounding Unicode scalars stay intact.
#show persian-zwnj: token => context {
  if literal-text.get() { token } else { space-persian-zwnj(token, gap: zwnj-gap) }
}
#render(
  read("source.md"),
  raw-typst: false,
  smart-punctuation: false,
  scope: (
    image: resolved-image,
  ),
)
