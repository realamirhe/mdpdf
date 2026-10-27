mod common;

use base64::Engine as _;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
#[ignore = "requires bundled macOS ARM64 Typst and configured fonts"]
fn cli_handles_assets_themes_and_safe_output() {
    let bundle = common::Bundle::new();
    let dir = tempdir().unwrap();
    let working_dir = tempdir().unwrap();
    let image = dir.path().join("عکس محلی.png");
    // Reuse the bundled PNG for local, embedded, and HTML image coverage.
    let png = fs::read(common::root().join("vendor/codly/src/typst-small.png")).unwrap();
    fs::write(&image, &png).unwrap();
    let data_uri = format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&png)
    );
    let source = dir.path().join("یادداشت من.md");
    let content = format!(
        r#"# Persian and English

یادداشت‌های من؛ می‌خواهم نرم‌افزار، خانه‌ها و نیم‌فاصله درست نمایش داده شوند.

![relative](عکس%20محلی.png)

![absolute](<{image_path}>)

![file URI](file://{image_path})

![base64]({data_uri})

<img data-src="ignored.png" src="{data_uri}" alt="HTML image">

```md
![code sample](missing.png)
```

## Long code

```rust
let value = "{long_token}";
```

## Mixed list

1. متن فارسی اول
2. PDF کنار فایل ساخته می‌شود.
3. مورد آخر با English

## Pagination

{paragraphs}"#,
        image_path = image.display(),
        long_token = "very_long_unbroken_token_".repeat(15),
        paragraphs =
            "این یک پاراگراف فارسی طولانی برای بررسی صفحه‌بندی و خوانایی متن در چند صفحه است.\n\n"
                .repeat(40),
    );
    fs::write(&source, content).unwrap();
    let original = fs::read(&source).unwrap();

    for theme in ["editorial", "classic"] {
        let output = dir.path().join(format!("{theme}.pdf"));
        let args = [
            source.as_path(),
            Path::new("--theme"),
            Path::new(theme),
            Path::new("-o"),
            output.as_path(),
        ];
        let result = bundle.invoke(working_dir.path(), &args);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(fs::read(&output).unwrap().starts_with(b"%PDF"));
    }
    let result = bundle.invoke(working_dir.path(), &[source.as_path()]);
    assert!(result.status.success());
    let old = fs::read(source.with_extension("pdf")).unwrap();
    assert!(!bundle
        .invoke(working_dir.path(), &[source.as_path()])
        .status
        .success());
    assert_eq!(fs::read(source.with_extension("pdf")).unwrap(), old);
    assert!(bundle
        .invoke(
            working_dir.path(),
            &[source.as_path(), Path::new("--force")]
        )
        .status
        .success());
    assert_eq!(fs::read(&source).unwrap(), original);

    fs::create_dir(dir.path().join("child")).unwrap();
    let alias = dir.path().join("child/../یادداشت من.md");
    assert!(!bundle
        .invoke(
            working_dir.path(),
            &[
                source.as_path(),
                Path::new("-o"),
                alias.as_path(),
                Path::new("--force")
            ]
        )
        .status
        .success());
    assert_eq!(fs::read(&source).unwrap(), original);

    let bad = dir.path().join("bad.md");
    fs::write(&bad, "![broken](data:image/png;base64,invalid!)").unwrap();
    assert!(!bundle
        .invoke(working_dir.path(), &[bad.as_path()])
        .status
        .success());
    assert!(!bad.with_extension("pdf").exists());
    fs::write(&bad, "![missing](absent.png)").unwrap();
    assert!(!bundle
        .invoke(working_dir.path(), &[bad.as_path()])
        .status
        .success());
    assert!(!bad.with_extension("pdf").exists());
}
