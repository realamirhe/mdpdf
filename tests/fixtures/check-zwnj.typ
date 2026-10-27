#import "template.typ": space-persian-zwnj
#import "vendor/cmarker/lib.typ": render
#set text(font: "Vazirmatn", lang: "fa", size: 20pt)
#let plain(value) = {
  if type(value) == str { value }
  else if type(value) == content {
    if value.func() == [ ].func() { " " }
    else if value.has("text") { value.text }
    else if value.has("body") { plain(value.body) }
    else if value.has("children") { value.children.map(plain).join("") }
    else { "" }
  } else { "" }
}
#for word in json("cases.json") {
  assert.eq(plain(space-persian-zwnj(text(word))), word)
  assert.eq(plain(space-persian-zwnj(text(word), gap: 0)), word)
}
#let rendered = render(read("fixture.md"), raw-typst: false, smart-punctuation: false)
#assert.eq(plain(rendered), read("expected.txt"))
#context {
  let token = text("ب‌ب‌ب")
  let original = measure(token).width
  let spaced = measure(space-persian-zwnj(token, gap: .10)).width
  assert(calc.abs((spaced - original) / 1pt - 4) < .01)
  assert(calc.abs((measure(space-persian-zwnj(token, gap: 0)).width - original) / 1pt) < .01)
}
