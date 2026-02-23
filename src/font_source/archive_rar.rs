use crate::font_source::{FontFile, FontSource, TEMPDIR_PREFIX, path_is_font};
use anyhow::Result;
use log::{debug, info, warn};
use std::{fs, path::Path};
use tempdir::TempDir;
use unrar;

/// 包含字体的 RAR 压缩包
pub struct FontArchiveRar {
    path: String,
    extract: TempDir,
    loaded: Vec<(String, FontFile)>,
}

impl FontArchiveRar {
    pub fn new(path: String) -> Self {
        Self {
            path,
            extract: TempDir::new(TEMPDIR_PREFIX).unwrap(),
            loaded: Vec::new(),
        }
    }
}

impl FontSource for FontArchiveRar {
    fn load(&mut self) -> Result<()> {
        debug!("Walking rar \"{}\"", self.path);
        let mut archive = unrar::Archive::new(&self.path).open_for_processing()?;
        while let Some(header) = archive.read_header()? {
            let path = Path::new(&header.entry().filename);
            let path_str = path.to_str().unwrap().to_string();
            if !path_is_font(&path) {
                archive = header.skip()?;
                continue;
            }
            let extract = self.extract.path().join(path);
            debug!(
                "Found font file \"{}\" from rar \"{}\" and extract to \"{}\"",
                path_str,
                self.path,
                extract.to_str().unwrap(),
            );
            if let Some(parent) = &extract.parent()
                && !parent.exists()
            {
                fs::create_dir_all(parent)?;
            }
            archive = header.extract_to(&extract)?;
            let mut f = FontFile::new(extract.to_str().unwrap().to_string());
            match f.load() {
                Ok(_) => {
                    self.loaded.push((path_str.clone(), f));
                    info!(
                        "Extracted font \"{}\" from \"{}\" and loaded",
                        path_str, self.path
                    );
                }
                Err(err) => {
                    warn!(
                        "Skipped font \"{}\" from \"{}\" failed to load: {}",
                        path_str, self.path, err
                    );
                }
            }
        }
        Ok(())
    }

    fn unload(&self) {
        self.loaded.iter().for_each(|(name, f)| {
            debug!(
                "Unload font file \"{}\" (\"{}\") from 7z \"{}\"",
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
