use base64::{engine::general_purpose::STANDARD, Engine as _};
use percent_encoding::percent_decode_str;
use pulldown_cmark::{Event, Options, Parser, Tag};
use regex::Regex;
use serde_json::{json, Map, Value};
use std::collections::{HashMap, HashSet};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::Builder;

#[derive(Debug)]
struct Args {
    input: PathBuf,
    output: Option<PathBuf>,
    theme: String,
    force: bool,
    config: Option<PathBuf>,
    open: bool,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("mdpdf: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args(env::args_os().skip(1))?;
    let input = fs::canonicalize(&args.input)?;
    let source = fs::read_to_string(&input).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("could not read UTF-8 Markdown {}: {error}", input.display()),
        )
    })?;
    let root = env::current_exe()?
        .parent()
        .ok_or("could not determine mdpdf installation directory")?
        .to_path_buf();
    let template = root.join("template.typ");
    let typst = root.join("vendor/typst");
    for required in [&template, &typst] {
        if !required.is_file() {
            return Err(format!("required bundled file is missing: {}", required.display()).into());
        }
    }

    let output = args.output.unwrap_or_else(|| input.with_extension("pdf"));
    let output = absolute_from_cwd(output)?;
    if same_file_path(&output, &input)? {
        return Err("output path must not overwrite the Markdown input".into());
    }
    if output.exists() && !args.force {
        return Err(format!(
            "output already exists: {} (use --force to replace it)",
            output.display()
        )
        .into());
    }
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }

    let config_was_explicit = args.config.is_some();
    let config_path = args.config.unwrap_or_else(|| root.join("settings.json"));
    if config_was_explicit && !config_path.is_file() {
        return Err(format!(
            "configuration file does not exist: {}",
            config_path.display()
        )
        .into());
    }
    let mut config = if config_path.exists() {
        serde_json::from_slice::<Value>(&fs::read(&config_path)?)?
    } else {
        json!({})
    };
    if !config.is_object() {
        return Err(format!(
            "configuration must be a JSON object: {}",
            config_path.display()
        )
        .into());
    }
    for key in ["font_size_pt", "line_height", "page_margin_mm"] {
        let value = config
            .get(key)
            .and_then(Value::as_f64)
            .ok_or_else(|| format!("{key} must be a number in the configuration"))?;
        if !value.is_finite()
            || value <= 0.0
            || (key == "line_height" && value < 1.0)
            || (key == "page_margin_mm" && value >= 80.0)
        {
            return Err(format!("invalid configuration value for {key}: {value}").into());
        }
    }
    config["theme"] = json!(args.theme);

    let stage = Builder::new().prefix("mdpdf-").tempdir()?;
    stage_layout(&root, &template, stage.path())?;
    let mut staged_config = config.clone();
    filter_config_fonts(&mut staged_config, &typst, stage.path())?;
    let image_roots = collect_images(
        &source,
        input.parent().unwrap_or(Path::new(".")),
        stage.path(),
    )?;
    fs::write(stage.path().join("source.md"), source)?;
    fs::write(
        stage.path().join("settings.json"),
        serde_json::to_vec_pretty(&staged_config)?,
    )?;
    let assets_json = image_roots
        .iter()
        .map(|(source, staged)| (source.clone(), json!(staged)))
        .collect::<Map<String, Value>>();
    fs::write(
        stage.path().join("assets.json"),
        serde_json::to_vec_pretty(&assets_json)?,
    )?;

    let mut command = Command::new(&typst);
    command
        .arg("compile")
        .arg("--package-path")
        .arg(stage.path().join("packages"))
        .arg("--root")
        .arg(stage.path())
        .arg("--font-path")
        .arg(stage.path().join("fonts"));
    if let Some(paths) = config.get("font_paths") {
        let paths = paths
            .as_array()
            .ok_or("configuration field font_paths must be an array of paths")?;
        for path in paths {
            let path = path
                .as_str()
                .ok_or("configuration field font_paths contains a non-string path")?;
            command.arg("--font-path").arg(expand_home(path));
        }
    }
    let staged_pdf = stage.path().join("output.pdf");
    let status = command
        .arg(stage.path().join("template.typ"))
        .arg(&staged_pdf)
        .stdin(Stdio::null())
        .status()?;
    if !status.success() {
        return Err(format!("Typst compilation failed with status {status}").into());
    }
    if !staged_pdf.is_file() {
        return Err("Typst reported success but did not produce a PDF".into());
    }

    persist_output(&staged_pdf, &output, args.force)?;
    println!("Created {}", output.display());
    if args.open {
        open_file(&output)?;
    }
    drop(stage);
    Ok(())
}

fn parse_args<I>(iter: I) -> Result<Args, Box<dyn std::error::Error>>
where
    I: IntoIterator<Item = std::ffi::OsString>,
{
    let mut values = iter.into_iter();
    let mut input = None;
    let mut output = None;
    let mut theme = "editorial".to_owned();
    let mut force = false;
    let mut config = None;
    let mut open = false;
    while let Some(arg) = values.next() {
        if arg == OsStr::new("-h") || arg == OsStr::new("--help") {
            println!("Usage: mdpdf <input.md> [--theme classic|editorial] [-o|--output <path>] [--force] [--config <path>] [--open]");
            std::process::exit(0);
        } else if arg == OsStr::new("-o") || arg == OsStr::new("--output") {
            output = Some(PathBuf::from(
                values.next().ok_or("--output requires a path")?,
            ));
        } else if arg == OsStr::new("--config") {
            config = Some(PathBuf::from(
                values.next().ok_or("--config requires a path")?,
            ));
        } else if arg == OsStr::new("--theme") {
            theme = values
                .next()
                .ok_or("--theme requires classic or editorial")?
                .to_string_lossy()
                .into_owned();
            if theme != "classic" && theme != "editorial" {
                return Err("--theme must be classic or editorial".into());
            }
        } else if arg == OsStr::new("--force") {
            force = true;
        } else if arg == OsStr::new("--open") {
            open = true;
        } else if arg.to_string_lossy().starts_with('-') {
            return Err(format!("unknown option: {}", arg.to_string_lossy()).into());
        } else if input.replace(PathBuf::from(arg)).is_some() {
            return Err("only one input Markdown file is supported".into());
        }
    }
    Ok(Args {
        input: input.ok_or("missing input Markdown file (try --help)")?,
        output,
        theme,
        force,
        config,
        open,
    })
}

fn absolute_from_cwd(path: PathBuf) -> Result<PathBuf, io::Error> {
    if path.is_absolute() {
        Ok(path)
    } else {
        Ok(env::current_dir()?.join(path))
    }
}

fn expand_home(path: &str) -> PathBuf {
    if path == "~" {
        return env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(path));
    }
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(path)
}

fn filter_config_fonts(
    config: &mut Value,
    typst: &Path,
    stage: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut command = Command::new(typst);
    command
        .arg("fonts")
        .arg("--font-path")
        .arg(stage.join("fonts"));
    if let Some(paths) = config.get("font_paths") {
        let paths = paths
            .as_array()
            .ok_or("configuration field font_paths must be an array of paths")?;
        for path in paths {
            let path = path
                .as_str()
                .ok_or("configuration field font_paths contains a non-string path")?;
            command.arg("--font-path").arg(expand_home(path));
        }
    }
    let output = command.output()?;
    if !output.status.success() {
        return Err(format!("Typst font discovery failed with status {}", output.status).into());
    }
    let available = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|family| family.trim().to_lowercase())
        .filter(|family| !family.is_empty())
        .collect::<HashSet<_>>();

    let persian = config
        .get("persian_font")
        .and_then(Value::as_str)
        .filter(|name| !name.trim().is_empty())
        .ok_or("configuration must specify a non-empty persian_font")?;
    if !available.contains(&persian.to_lowercase()) {
        return Err(
            format!("requested Persian font is not installed or discoverable: {persian}").into(),
        );
    }
    for key in ["latin_fonts", "mono_fonts"] {
        let Some(value) = config.get(key) else {
            continue;
        };
        let requested = value.as_array().ok_or_else(|| {
            format!("configuration field {key} must be an array of font family names")
        })?;
        let names = requested
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .filter(|name| !name.trim().is_empty())
                    .ok_or_else(|| {
                        format!(
                            "configuration field {key} contains a non-string or empty font name"
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        if names.is_empty() {
            continue;
        }
        if !available.contains(&names[0].to_lowercase()) {
            eprintln!(
                "mdpdf: preferred {key} font {:?} is unavailable; checking fallback fonts",
                names[0]
            );
        }
        let filtered = names
            .into_iter()
            .filter(|name| available.contains(&name.to_lowercase()))
            .map(|name| json!(name))
            .collect::<Vec<_>>();
        if filtered.is_empty() {
            return Err(
                format!("none of the requested {key} fonts are installed or discoverable").into(),
            );
        }
        config[key] = Value::Array(filtered);
    }
    Ok(())
}

fn stage_layout(root: &Path, template: &Path, stage: &Path) -> Result<(), io::Error> {
    fs::copy(template, stage.join("template.typ"))?;
    for package in ["cmarker", "auto-bidi"] {
        let src = root.join("vendor").join(package);
        let dst = stage.join("vendor").join(package);
        fs::create_dir_all(&dst)?;
        for name in if package == "cmarker" {
            vec!["lib.typ", "plugin.wasm", "typst.toml"]
        } else {
            vec!["lib.typ", "typst.toml"]
        } {
            let from = src.join(name);
            if from.exists() {
                fs::copy(from, dst.join(name))?;
            }
        }
    }
    copy_tree(&root.join("vendor/codly"), &stage.join("vendor/codly"))?;
    copy_tree(
        &root.join("vendor/preview"),
        &stage.join("packages/preview"),
    )?;
    fs::create_dir_all(stage.join("assets"))?;
    fs::create_dir_all(stage.join("fonts"))?;
    Ok(())
}

fn collect_images(
    markdown: &str,
    source_dir: &Path,
    stage: &Path,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let mut sources = Vec::<String>::new();
    let parser = Parser::new_ext(markdown, Options::all());
    for event in parser {
        match event {
            Event::Start(Tag::Image { dest_url, .. }) => sources.push(dest_url.to_string()),
            Event::Html(html) | Event::InlineHtml(html) => {
                sources.extend(html_image_sources(&html))
            }
            _ => {}
        }
    }
    sources.sort();
    sources.dedup();

    let mut roots = HashMap::new();
    for (index, source) in sources.iter().enumerate() {
        if source
            .get(..7)
            .is_some_and(|scheme| scheme.eq_ignore_ascii_case("http://"))
            || source
                .get(..8)
                .is_some_and(|scheme| scheme.eq_ignore_ascii_case("https://"))
        {
            return Err(format!("remote image URLs are unsupported: {source}").into());
        }
        let relative = format!("assets/asset-{index}{}", image_extension(source));
        let target = stage.join(&relative);
        if source
            .get(..11)
            .is_some_and(|scheme| scheme.eq_ignore_ascii_case("data:image/"))
        {
            let (bytes, extension) = decode_data_image(source)?;
            let relative = format!("assets/asset-{index}.{extension}");
            fs::write(stage.join(&relative), bytes)?;
            roots.insert(source.clone(), relative);
        } else {
            let path = resolve_local_image(source, source_dir)?;
            fs::copy(&path, &target).map_err(|error| {
                io::Error::new(
                    error.kind(),
                    format!("could not stage image {source:?}: {error}"),
                )
            })?;
            roots.insert(source.clone(), relative);
        }
    }
    Ok(roots)
}

fn html_image_sources(html: &str) -> Vec<String> {
    let tags = Regex::new(r#"(?is)<img\b(?:[^>"']|"[^"]*"|'[^']*')*>"#).expect("valid image regex");
    let attributes =
        Regex::new(r#"(?is)\s+([a-z_:][a-z0-9_.:-]*)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))"#)
            .expect("valid attribute regex");
    tags.find_iter(html)
        .filter_map(|tag| {
            attributes.captures_iter(tag.as_str()).find_map(|capture| {
                if !capture[1].eq_ignore_ascii_case("src") {
                    return None;
                }
                (2..=4).find_map(|i| {
                    capture
                        .get(i)
                        .map(|m| html_escape::decode_html_entities(m.as_str()).into_owned())
                })
            })
        })
        .collect()
}

fn copy_tree(source: &Path, target: &Path) -> Result<(), io::Error> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target.join(entry.file_name()))?;
        } else if entry.file_type()?.is_file() {
            fs::copy(entry.path(), target.join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn same_file_path(output: &Path, input: &Path) -> Result<bool, io::Error> {
    if output.exists() {
        return Ok(fs::canonicalize(output)? == input);
    }
    Ok(false)
}

fn resolve_local_image(
    source: &str,
    source_dir: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let decoded = if let Some(path) = source.strip_prefix("file://") {
        let uri_path = if let Some(rest) = path.strip_prefix("localhost/") {
            format!("/{rest}")
        } else if path.starts_with('/') {
            path.to_owned()
        } else {
            return Err(format!("non-local file URI is unsupported: {source}").into());
        };
        percent_decode_str(&uri_path).decode_utf8()?.into_owned()
    } else {
        percent_decode_str(source).decode_utf8()?.into_owned()
    };
    #[cfg(windows)]
    let decoded = if decoded.starts_with('/') && decoded.as_bytes().get(2) == Some(&b':') {
        decoded[1..].to_owned()
    } else {
        decoded
    };
    let path = PathBuf::from(decoded);
    let path = if path.is_absolute() {
        path
    } else {
        source_dir.join(path)
    };
    let canonical = fs::canonicalize(&path)?;
    if !canonical.is_file() {
        return Err(format!("image path is not a file: {}", canonical.display()).into());
    }
    Ok(canonical)
}

fn decode_data_image(source: &str) -> Result<(Vec<u8>, &'static str), Box<dyn std::error::Error>> {
    let (metadata, payload) = source.split_once(',').ok_or("invalid image data URI")?;
    let mime = metadata
        .get(..5)
        .filter(|scheme| scheme.eq_ignore_ascii_case("data:"))
        .and_then(|_| metadata.get(5..))
        .and_then(|v| v.split(';').next())
        .ok_or("invalid image data URI")?;
    if !metadata.ends_with(";base64") {
        return Err("only base64-encoded image data URIs are supported".into());
    }
    let extension = match mime.to_ascii_lowercase().as_str() {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "image/svg+xml" => "svg",
        "image/webp" => "webp",
        "image/bmp" => "bmp",
        _ => return Err(format!("unsupported image MIME type: {mime}").into()),
    };
    Ok((STANDARD.decode(payload)?, extension))
}

fn image_extension(source: &str) -> &'static str {
    let path = source.split(['?', '#']).next().unwrap_or(source);
    match Path::new(path)
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => ".png",
        "jpg" | "jpeg" => ".jpg",
        "gif" => ".gif",
        "svg" => ".svg",
        "webp" => ".webp",
        "bmp" => ".bmp",
        _ => ".img",
    }
}

fn persist_output(
    staged: &Path,
    output: &Path,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let temp = Builder::new()
        .prefix(".mdpdf-output-")
        .tempfile_in(output.parent().unwrap_or(Path::new(".")))?;
    fs::copy(staged, temp.path())?;
    if force {
        temp.persist(output)?;
    } else {
        temp.persist_noclobber(output)?;
    }
    Ok(())
}

fn open_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    let mut command = Command::new("open");
    #[cfg(target_os = "windows")]
    let mut command = Command::new("cmd");
    #[cfg(target_os = "windows")]
    command.arg("/C").arg("start").arg("");
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = Command::new("xdg-open");
    command.arg(path).status()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn resolves_percent_encoded_relative_path_from_source_directory() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("my image.png"), b"image").unwrap();
        let resolved = resolve_local_image("my%20image.png", temp.path()).unwrap();
        assert_eq!(fs::read(resolved).unwrap(), b"image");
    }

    #[test]
    fn rejects_invalid_base64_image_data() {
        assert!(decode_data_image("data:image/png;base64,not-base64!").is_err());
    }

    #[test]
    fn extracts_quoted_and_unquoted_html_img_src_values() {
        let sources =
            html_image_sources(r#"<img src="a&amp;b.png"><IMG SRC='two.png'><img src=three.png>"#);
        assert_eq!(sources, vec!["a&b.png", "two.png", "three.png"]);
    }

    #[test]
    fn html_src_is_not_confused_with_data_src_or_alt_text() {
        let input = r#"<img alt="src='fake.png'" data-src="lazy.png" src="real.png">"#;
        assert_eq!(html_image_sources(input), vec!["real.png"]);
    }

    #[test]
    fn detects_input_path_alias_through_parent() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir(temp.path().join("child")).unwrap();
        let input = temp.path().join("input.md");
        fs::write(&input, "text").unwrap();
        assert!(same_file_path(
            &temp.path().join("child/../input.md"),
            &fs::canonicalize(input).unwrap()
        )
        .unwrap());
    }

    #[test]
    fn ignores_images_in_fenced_code() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir(temp.path().join("assets")).unwrap();
        assert!(collect_images(
            "```md\n![not real](missing.png)\n```",
            temp.path(),
            temp.path()
        )
        .unwrap()
        .is_empty());
    }

    #[test]
    fn refuses_to_overwrite_without_force() {
        let temp = tempfile::tempdir().unwrap();
        let staged = temp.path().join("new.pdf");
        let output = temp.path().join("existing.pdf");
        fs::write(&staged, b"new").unwrap();
        fs::write(&output, b"old").unwrap();
        assert!(persist_output(&staged, &output, false).is_err());
        assert_eq!(fs::read(output).unwrap(), b"old");
    }
}
