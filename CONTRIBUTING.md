# Contributing

Keep changes focused and describe the user-visible behavior they affect. For a
rendering bug, include a minimal Markdown file, the theme, font settings, platform,
and the error message or a screenshot with private content removed.

## Local checks

```sh
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
```

The default suite checks image parsing, path handling, and output protection
without invoking Typst. PDF integration tests are ignored
by default because they need the bundled macOS Apple Silicon Typst binary and
installed fonts. With the prerequisites in the README installed, run:

```sh
cargo test --locked -- --ignored
```

Enable the local pre-push hook once per clone:

```sh
git config --local core.hooksPath .githooks
```

The hook runs all tests, including PDF integration tests, before each push and
blocks the push if they fail. It requires the same Typst and font prerequisites.
There is no GitHub CI workflow.

These tests cover both themes, local and embedded images, Unicode paths,
overwrite protection, source preservation, and Persian non-joiner spacing. They
stage the Cargo-built binary with its resources in temporary directories; no
manual copy to the project root is needed. Persian fixtures are intentional test
data, while project documentation and code comments use English.

For template changes, also inspect a generated PDF for clipping, line breaks,
font fallback, and page flow. Automated assertions do not replace visual review.

Keep `Cargo.lock` in version control. Preserve upstream license files and notices
when changing bundled dependencies, and record any version changes in
`THIRD_PARTY.md`. Do not include generated PDFs or local font files in a patch.
