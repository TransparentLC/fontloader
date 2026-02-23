use crate::font_source::FontSource;
use anyhow::{Result, bail};
use log::{debug, warn};
use windows::Win32::Graphics::Gdi::{AddFontResourceW, RemoveFontResourceW};
use windows_strings::HSTRING;

/// 一个字体文件
pub struct FontFile {
    pub path: String,
}

impl FontFile {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

impl FontSource for FontFile {
    fn load(&mut self) -> Result<()> {
        debug!("Call AddFontResourceW for \"{}\"", self.path);
        unsafe {
            if AddFontResourceW(&HSTRING::from(&self.path)) == 0 {
                bail!("Failed to call AddFontResourceW for \"{}\"", self.path)
            } else {
                Ok(())
            }
        }
    }

    fn unload(&self) {
        debug!("Call RemoveFontResourceW for \"{}\"", self.path);
        unsafe {
            if !RemoveFontResourceW(&HSTRING::from(&self.path)).as_bool() {
                warn!("Failed to call RemoveFontResourceW for \"{}\"", self.path);
            }
        }
    }

    fn get_path(&self) -> &String {
        &self.path
    }
}
