use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub struct Bundle {
    _directory: tempfile::TempDir,
    pub binary: PathBuf,
}

impl Bundle {
    pub fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let binary = directory.path().join("mdpdf");
        fs::copy(env!("CARGO_BIN_EXE_mdpdf"), &binary).unwrap();
        for name in ["template.typ", "settings.json"] {
            fs::copy(root().join(name), directory.path().join(name)).unwrap();
        }
        copy_tree(&root().join("vendor"), &directory.path().join("vendor")).unwrap();
        Self {
            _directory: directory,
            binary,
        }
    }

    pub fn invoke(&self, cwd: &Path, args: &[&Path]) -> Output {
        Command::new(&self.binary)
            .current_dir(cwd)
            .args(args)
            .output()
            .unwrap()
    }
}

pub fn copy_tree(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
