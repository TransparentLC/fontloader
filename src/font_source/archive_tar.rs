use crate::font_source::{FontFile, FontSource, TEMPDIR_PREFIX, path_is_font};
use anyhow::Result;
use bzip2::read::BzDecoder;
use cfg_if::cfg_if;
use flate2::read::GzDecoder;
use log::{debug, info, warn};
#[cfg(feature = "parallel")]
use rayon::iter::{IntoParallelRefIterator, ParallelExtend, ParallelIterator};
use std::{
    fs::{self, File},
    io::{self, Read},
    time::Instant,
};
use tempdir::TempDir;
use xz2::read::XzDecoder;

#[derive(Debug)]
pub enum TarCompression {
    None,
    GZ,
    BZ2,
    XZ,
    Zstd,
}

/// 包含字体的 tarball，可以是原始文件或 tar.{gz,bz2,xz,zstd} 压缩
pub struct FontArchiveTar {
    path: String,
    compression: TarCompression,
    extract: TempDir,
    loaded: Vec<(String, FontFile)>,
}

impl FontArchiveTar {
    pub fn new(path: String, compression: TarCompression) -> Self {
        Self {
            path,
            compression,
            extract: TempDir::new(TEMPDIR_PREFIX).unwrap(),
            loaded: Vec::new(),
        }
    }
}

impl FontSource for FontArchiveTar {
    fn load(&mut self) -> Result<()> {
        debug!("Walking tar \"{}\"", self.path);
        let archive = File::open(&self.path)?;
        let archive: Box<dyn Read> = match self.compression {
            TarCompression::None => Box::new(archive),
            TarCompression::GZ => Box::new(GzDecoder::new(archive)),
            TarCompression::BZ2 => Box::new(BzDecoder::new(archive)),
            TarCompression::XZ => Box::new(XzDecoder::new(archive)),
            TarCompression::Zstd => Box::new(zstd::Decoder::new(archive)?),
        };
        let mut archive = tar::Archive::new(archive);
        let mut extracted = vec![];
        let start = Instant::now();
        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;
            if !path_is_font(&path) {
                continue;
            }

            let extract = self.extract.path().join(&path);
            let path_str = path.to_str().unwrap().to_string();
            debug!(
                "Found font \"{}\" from tar \"{}\" and extract to \"{}\"",
                path_str,
                self.path,
                extract.to_str().unwrap(),
            );
            if let Some(parent) = &extract.parent()
                && !parent.exists()
            {
                fs::create_dir_all(parent)?;
            }
            File::create(&extract).and_then(|mut outfile| io::copy(&mut entry, &mut outfile))?;
            extracted.push((path_str, extract.to_str().unwrap().to_string()));
        }
        debug!(
            "Extracted fonts from tar \"{}\" in {}s",
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
                "Unload font \"{}\" (\"{}\") from tar \"{}\"",
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
