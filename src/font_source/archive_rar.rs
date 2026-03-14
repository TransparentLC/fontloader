use crate::font_source::{FontFile, FontSource, TEMPDIR_PREFIX, path_is_font};
use anyhow::Result;
use cfg_if::cfg_if;
use log::{debug, info, warn};
#[cfg(feature = "parallel")]
use rayon::iter::{IntoParallelRefIterator, ParallelExtend, ParallelIterator};
use std::{fs, path::Path, time::Instant};
use tempdir::TempDir;

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
        let mut extracted = vec![];
        let start = Instant::now();
        while let Some(header) = archive.read_header()? {
            let path = Path::new(&header.entry().filename);
            let path_str = path.to_str().unwrap().to_string();
            if !path_is_font(path) {
                archive = header.skip()?;
                continue;
            }
            let extract = self.extract.path().join(path);
            debug!(
                "Found font \"{}\" from rar \"{}\" and extract to \"{}\"",
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
            extracted.push((path_str, extract.to_str().unwrap().to_string()));
        }
        debug!(
            "Extracted fonts from rar \"{}\" in {}s",
            self.path,
            start.elapsed().as_secs_f64()
        );

        let extracted_op = |name_extract: &(String, String)| -> Option<(String, FontFile)> {
            let (name, extract) = name_extract;
            let mut f = FontFile::new(extract.clone());
            match f.load() {
                Ok(_) => {
                    info!(
                        "Extracted font \"{}\" from \"{}\" and loaded",
                        name, self.path
                    );
                    Some((name.clone(), f))
                }
                Err(err) => {
                    warn!(
                        "Skipped font \"{}\" from \"{}\" failed to load: {}",
                        name, self.path, err
                    );
                    None
                }
            }
        };
        cfg_if! {
            if #[cfg(feature = "parallel")] {
                let iter = extracted.par_iter();
                self.loaded.par_extend(iter.filter_map(extracted_op));
            } else {
                let iter = extracted.iter();
                self.loaded.extend(iter.filter_map(extracted_op));
            }
        }

        Ok(())
    }

    fn unload(&self) {
        cfg_if! {
            if #[cfg(feature = "parallel")] {
                let iter = self.loaded.par_iter();
            } else {
                let iter = self.loaded.iter();
            }
        }
        iter.for_each(|(name, f)| {
            debug!(
                "Unload font \"{}\" (\"{}\") from rar \"{}\"",
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
