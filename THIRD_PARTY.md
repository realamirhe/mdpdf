# Third-party components

Bundled components retain their upstream licenses. Fonts are supplied by the
user and are not distributed in this repository.

| Component | Version | License files |
| --- | --- | --- |
| Typst | 0.15.1 | [License](vendor/LICENSE), [notices](vendor/NOTICE) |
| cmarker | 0.1.10 | [MIT](vendor/cmarker/LICENSE) |
| auto-bidi | 0.1.0 | [MIT](vendor/auto-bidi/LICENSE) |
| Codly | 2.0.0 | [MIT](vendor/codly/LICENSE) |
| Elembic | 1.1.1 | [MIT](vendor/preview/elembic/1.1.1/LICENSE-MIT) or [Apache-2.0](vendor/preview/elembic/1.1.1/LICENSE-APACHE) |

The bundled `vendor/typst` executable reports `typst 0.15.1 (9dfd3a08)` and is a
macOS ARM64 binary. Its SHA-256 is:

```text
7c4a136b377f3689400afe37b4f0fe3528d50faaa55cbb1106ac8a128f86ba1a
```

The upstream project is `https://github.com/typst/typst`. The original download
URL and build recipe for this copy were not recorded. The version and checksum
identify the checked-in artifact; they do not establish a reproducible build.
Other package source locations are recorded in their `typst.toml` files.

Rust dependencies and their exact versions are recorded in `Cargo.lock`; their
licenses are declared in their respective crate metadata.
