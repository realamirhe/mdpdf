mod common;

use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

#[test]
#[ignore = "requires bundled macOS ARM64 Typst and configured fonts"]
fn persian_fixture_preserves_unicode_and_builds_both_themes() {
    let bundle = common::Bundle::new();
    let fixture = common::root().join("tests/fixtures/persian-zwnj.md");
    let original = fs::read(&fixture).unwrap();
    let dir = tempdir().unwrap();
    for theme in ["editorial", "classic"] {
        let output = dir.path().join(format!("{theme}.pdf"));
        let args = [
            fixture.as_path(),
            Path::new("--theme"),
            Path::new(theme),
            Path::new("-o"),
            output.as_path(),
        ];
        let result = bundle.invoke(dir.path(), &args);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(fs::read(&output).unwrap().starts_with(b"%PDF"));
    }
    let text = String::from_utf8(original.clone()).unwrap();
    for word in ["دیده‌شدن", "کارکردن", "قابل‌استفاده‌شدن", "کلیک", "ادامهٔ"]
    {
        assert!(
            text.contains(word),
            "fixture lost expected ZWNJ word {word:?}"
        );
    }
    assert_eq!(fs::read(fixture).unwrap(), original);
}

#[test]
#[ignore = "requires bundled macOS ARM64 Typst and configured fonts"]
fn typst_preserves_zwnj_spacing_and_literal_code_context() {
    let root = common::root();
    let stage = tempdir().unwrap();
    fs::copy(root.join("template.typ"), stage.path().join("template.typ")).unwrap();
    common::copy_tree(&root.join("vendor"), &stage.path().join("vendor")).unwrap();
    let mut config: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("settings.json")).unwrap()).unwrap();
    config["theme"] = serde_json::json!("editorial");
    fs::write(
        stage.path().join("settings.json"),
        serde_json::to_vec(&config).unwrap(),
    )
    .unwrap();
    fs::write(stage.path().join("assets.json"), "{}").unwrap();
    let cases = [
        "ب‌ب‌ب",
        "قابل‌استفاده‌شدن",
        "نگه‌داشتن",
        "دیده‌شدن",
        "کنترل‌ها",
        "می‌خواهم",
        "ادامهٔ",
        "نرم‌افزار",
        "ی ي ک ك",
        "هٔ ۀ",
    ];
    fs::write(
        stage.path().join("cases.json"),
        serde_json::to_string(&cases).unwrap(),
    )
    .unwrap();
    fs::write(
        stage.path().join("fixture.md"),
        fs::read_to_string(root.join("tests/fixtures/persian-zwnj.md")).unwrap(),
    )
    .unwrap();
    let source = fs::read_to_string(root.join("tests/fixtures/persian-zwnj.md")).unwrap();
    let expected =
        source
            .replacen('[', "", 1)
            .replacen("](https://addyosmani.com/blog/usability/)", "", 1);
    fs::write(stage.path().join("expected.txt"), expected).unwrap();
    fs::write(
        stage.path().join("source.md"),
        "نگه‌داشتن\n\n`نگه‌داشتن`\n\n```text\nنگه‌داشتن\n```\n\nنگه‌داشتن",
    )
    .unwrap();
    fs::write(
        stage.path().join("check.typ"),
        include_str!("fixtures/check-zwnj.typ"),
    )
    .unwrap();
    let typst = root.join("vendor/typst");
    let command = |operation: &str| {
        let mut cmd = Command::new(&typst);
        cmd.arg(operation)
            .arg("--root")
            .arg(stage.path())
            .arg("--package-path")
            .arg(stage.path().join("vendor"));
        for path in config["font_paths"].as_array().unwrap() {
            let path = path.as_str().unwrap();
            let path = if let Some(rest) = path.strip_prefix("~/") {
                std::env::var_os("HOME")
                    .map(|h| Path::new(&h).join(rest))
                    .unwrap()
            } else {
                path.into()
            };
            cmd.arg("--font-path").arg(path);
        }
        cmd
    };
    let output = command("compile")
        .arg(stage.path().join("check.typ"))
        .arg(stage.path().join("check.pdf"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let template_path = stage.path().join("template.typ");
    let template = fs::read_to_string(&template_path).unwrap();
    let hook = "#show persian-zwnj: token => context {";
    assert!(
        template.contains(hook),
        "update the instrumentation for the new template"
    );
    fs::write(&template_path, template.replace(hook,
        "#show persian-zwnj: token => context {\n  [#metadata(literal-text.get()) <zwnj-context>]"
    )).unwrap();
    let output = command("eval")
        .arg("--in")
        .arg(&template_path)
        .arg("query(<zwnj-context>).map(it => it.value)")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let contexts: Vec<bool> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(contexts, [false, true, true, false]);
}
