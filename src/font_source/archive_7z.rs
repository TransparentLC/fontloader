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

/// 包含字体的 7z 压缩包
pub struct FontArchive7z {
    path: String,
    extract: TempDir,
    loaded: Vec<(String, FontFile)>,
}

impl FontArchive7z {
    pub fn new(path: String) -> Self {
        Self {
            path,
            extract: TempDir::new(TEMPDIR_PREFIX).unwrap(),
            loaded: Vec::new(),
        }
    }
}

impl FontSource for FontArchive7z {
    fn load(&mut self) -> Result<()> {
        debug!("Walking 7z \"{}\"", self.path);
        let mut archive = sevenz_rust2::ArchiveReader::new(
            File::open(&self.path)?,
            sevenz_rust2::Password::empty(),
        )?;
        let mut extracted = vec![];
        let start = Instant::now();
        archive
            .for_each_entries(|entry, reader| {
                let path = Path::new(&entry.name);
                if !path_is_font(path) {
                    // 因为 7z 可以固实压缩，所以就算文件不需要解压到硬盘上也要解压一遍
                    io::copy(reader, &mut io::sink())?;
                    return Ok(true);
                }

                let extract = self.extract.path().join(Path::new(&entry.name));
                debug!(
                    "Found font \"{}\" from 7z \"{}\" and extract to \"{}\"",
                    entry.name,
                    self.path,
                    extract.to_str().unwrap(),
                );
                if let Some(parent) = &extract.parent()
                    && !parent.exists()
                {
                    fs::create_dir_all(parent)?;
                }
                File::create(&extract).and_then(|mut outfile| io::copy(reader, &mut outfile))?;
                extracted.push((entry.name.clone(), extract.to_str().unwrap().to_string()));

                Ok(true)
            })
            .map(|_| ())?;
        debug!(
            "Extracted fonts from 7z \"{}\" in {}s",
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
                "Unload font \"{}\" (\"{}\") from 7z \"{}\"",
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
