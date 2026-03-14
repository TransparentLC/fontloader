use crate::font_source::{FontFile, FontSource, path_is_font};
use anyhow::Result;
use cfg_if::cfg_if;
use log::{debug, info, warn};
#[cfg(feature = "parallel")]
use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelExtend, ParallelIterator};
use walkdir::{DirEntry, WalkDir};

/// 包含字体的文件夹
pub struct FontDir {
    path: String,
    loaded: Vec<FontFile>,
}

impl FontDir {
    pub fn new(path: String) -> Self {
        Self {
            path,
            loaded: Vec::new(),
        }
    }
}

impl FontSource for FontDir {
    fn load(&mut self) -> Result<()> {
        debug!("Walking dir \"{}\"", self.path);

        let entry_op = |entry: DirEntry| -> Option<FontFile> {
            let path = entry.path();
            if path_is_font(path) {
                let path_str = path.to_str().unwrap().to_string();
                debug!("Found font \"{path_str}\" from dir \"{}\"", self.path);
                let mut f = FontFile::new(path_str.clone());
                match f.load() {
                    Ok(_) => {
                        info!(
                            "Found font \"{path_str}\" from \"{}\" and loaded",
                            self.path
                        );
                        return Some(f);
                    }
                    Err(err) => {
                        warn!(
                            "Skipped font \"{}\" from dir \"{}\" failed to load: {}",
                            path_str, self.path, err
                        );
                    }
                }
            }
            None
        };

        let iter = WalkDir::new(&self.path).into_iter().filter_map(|e| e.ok());
        cfg_if! {
            if #[cfg(feature = "parallel")] {
                let iter = iter.par_bridge();
                self.loaded.par_extend(iter.filter_map(entry_op));
            } else {
                self.loaded.extend(iter.filter_map(entry_op));
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
        iter.for_each(|f| {
            f.unload();
            info!("Unloaded font \"{}\" from dir \"{}\"", f.path, self.path);
        });
    }

    fn get_path(&self) -> &String {
        &self.path
    }
}
