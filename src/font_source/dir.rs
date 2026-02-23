use crate::font_source::{FontFile, FontSource, path_is_font};
use anyhow::Result;
use log::{debug, info, warn};
use walkdir::WalkDir;

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
        for entry in WalkDir::new(&self.path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path_is_font(path) {
                let path_str = path.to_str().unwrap().to_string();
                debug!("Found font \"{path_str}\" from dir \"{}\"", self.path);
                let mut f = FontFile::new(path_str.clone());
                match f.load() {
                    Ok(_) => {
                        self.loaded.push(f);
                    }
                    Err(err) => {
                        warn!(
                            "Skipped font \"{}\" from dir \"{}\" failed to load: {}",
                            path_str, self.path, err
                        );
                    }
                }
            }
        }
        Ok(())
    }

    fn unload(&self) {
        self.loaded.iter().for_each(|f| {
            f.unload();
            info!("Unloaded font \"{}\" from dir \"{}\"", f.path, self.path);
        });
    }

    fn get_path(&self) -> &String {
        &self.path
    }
}
