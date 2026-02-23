use crate::font_source::{FontFile, FontSource, TEMPDIR_PREFIX, path_is_font};
use anyhow::Result;
use log::{debug, info, warn};
use std::{fs, fs::File, io, path::Path};
use tempdir::TempDir;
use zip::ZipArchive;

/// 包含字体的 ZIP 压缩包
pub struct FontArchiveZip {
    path: String,
    extract: TempDir,
    loaded: Vec<(String, FontFile)>,
}

impl FontArchiveZip {
    pub fn new(path: String) -> Self {
        Self {
            path,
            extract: TempDir::new(TEMPDIR_PREFIX).unwrap(),
            loaded: Vec::new(),
        }
    }
}

impl FontSource for FontArchiveZip {
    fn load(&mut self) -> Result<()> {
        debug!("Walking zip \"{}\"", self.path);
        let mut archive = ZipArchive::new(File::open(&self.path)?)?;
        let file_names: Vec<String> = archive
            .file_names()
            .filter(|name| path_is_font(Path::new(name)))
            .map(String::from)
            .collect();
        for name in file_names {
            let extract = self.extract.path().join(Path::new(&name));
            debug!(
                "Found font file \"{}\" from zip \"{}\" and extract to \"{}\"",
                name,
                self.path,
                extract.to_str().unwrap(),
            );
            let mut file = archive.by_name(&name)?;
            if let Some(parent) = &extract.parent()
                && !parent.exists()
            {
                fs::create_dir_all(parent)?;
            }
            File::create(&extract).and_then(|mut outfile| io::copy(&mut file, &mut outfile))?;
            let mut f = FontFile::new(extract.to_str().unwrap().to_string());
            match f.load() {
                Ok(_) => {
                    self.loaded.push((name.clone(), f));
                    info!(
                        "Extracted font \"{}\" from \"{}\" and loaded",
                        name, self.path
                    );
                }
                Err(err) => {
                    warn!(
                        "Skipped font \"{}\" from \"{}\" failed to load: {}",
                        name, self.path, err
                    );
                }
            }
        }
        Ok(())
    }

    fn unload(&self) {
        self.loaded.iter().for_each(|(name, f)| {
            debug!(
                "Unload font \"{}\" (\"{}\") from zip \"{}\"",
                f.path, name, self.path
            );
            f.unload();
            info!(
                "Unloaded extracted font \"{}\" from \"{}\"",
                name, self.path
            );
        });
    }

    fn get_path(&self) -> &String {
        &self.path
    }
}
