use crate::font_source::{FontFile, FontSource, TEMPDIR_PREFIX, path_is_font};
use anyhow::Result;
use cfg_if::cfg_if;
use log::{debug, info, warn};
#[cfg(feature = "parallel")]
use rayon::iter::{IntoParallelRefIterator, ParallelExtend, ParallelIterator};
use std::{
    fs::{self, File},
    io,
    path::Path,
    time::Instant,
};
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
        let mut extracted = vec![];
        let start = Instant::now();
        for name in file_names {
            let extract = self.extract.path().join(Path::new(&name));
            debug!(
                "Found font \"{}\" from zip \"{}\" and extract to \"{}\"",
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
            extracted.push((name, extract.to_str().unwrap().to_string()));
        }
        debug!(
            "Extracted fonts from zip \"{}\" in {}s",
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
