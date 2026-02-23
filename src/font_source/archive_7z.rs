use crate::font_source::{FontFile, FontSource, TEMPDIR_PREFIX, path_is_font};
use anyhow::Result;
use log::{debug, info, warn};
use sevenz_rust2;
use std::{fs, fs::File, io, path::Path};
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
                    "Found font file \"{}\" from 7z \"{}\" and extract to \"{}\"",
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
                let mut f = FontFile::new(extract.to_str().unwrap().to_string());
                match f.load() {
                    Ok(_) => {
                        self.loaded.push((entry.name.clone(), f));
                        info!(
                            "Extracted font \"{}\" from \"{}\" and loaded",
                            entry.name, self.path
                        );
                    }
                    Err(err) => {
                        warn!(
                            "Skipped font \"{}\" from \"{}\" failed to load: {}",
                            entry.name, self.path, err
                        );
                    }
                }

                Ok(true)
            })
            .map(|_| ())
            .map_err(|err| err.into())
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
