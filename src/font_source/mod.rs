use anyhow::Result;
use log::warn;
use std::{collections::HashSet, path::Path, sync::LazyLock};

mod file;
use file::FontFile;

#[cfg(feature = "dir")]
mod dir;
#[cfg(feature = "dir")]
use dir::FontDir;

#[cfg(feature = "archive-zip")]
mod archive_zip;
#[cfg(feature = "archive-zip")]
use archive_zip::FontArchiveZip;

#[cfg(feature = "archive-7z")]
mod archive_7z;
#[cfg(feature = "archive-7z")]
use archive_7z::FontArchive7z;

#[cfg(feature = "archive-rar")]
mod archive_rar;
#[cfg(feature = "archive-rar")]
use archive_rar::FontArchiveRar;

#[cfg(feature = "archive-tar")]
mod archive_tar;
#[cfg(feature = "archive-tar")]
use archive_tar::{FontArchiveTar, TarCompression};

/// AddFontResourceW 支持的字体文件扩展名
///
/// 参见：
/// AddFontResourceW 函数 （wingdi.h） - Win32 apps | Microsoft Learn
/// https://learn.microsoft.com/zh-cn/windows/win32/api/wingdi/nf-wingdi-addfontresourcew
static FONT_EXTENSION: LazyLock<HashSet<String>> = LazyLock::new(|| {
    [
        "fon", "fnt", "ttf", "ttc", "fot", "otf", "mmm", "pfb", "pfm",
    ]
    .iter()
    .map(|&s| s.to_string())
    .collect()
});

#[cfg(any(
    feature = "archive-zip",
    feature = "archive-rar",
    feature = "archive-7z",
    feature = "archive-tar",
))]
const TEMPDIR_PREFIX: &str = ".fontloader";

/// 根据扩展名检查一个路径是否为字体文件
fn path_is_font(path: &Path) -> bool {
    path.extension()
        .map(|ext| FONT_EXTENSION.contains(&ext.to_str().unwrap().to_ascii_lowercase()))
        .unwrap_or(false)
}

/// 加载字体文件的源
pub trait FontSource {
    /// 加载字体
    fn load(&mut self) -> Result<()>;
    /// 卸载字体
    fn unload(&self);
    /// 获取源的路径，主要用于日志输出
    fn get_path(&self) -> &String;
}

/// 输入路径得到字体源，可能是文件、文件夹或压缩包等等
/// 如果不是任何一种可以处理的源则返回 None
pub fn from_path(path: String) -> Option<Box<dyn FontSource>> {
    let p = Path::new(&path);
    if !p.exists() {
        warn!("File \"{path}\" not exists");
        return None;
    }
    #[cfg(feature = "dir")]
    if p.is_dir() {
        return Some(Box::new(FontDir::new(path)));
    }
    if p.is_file() {
        match p
            .extension()
            .map(|ext| ext.to_str().unwrap().to_ascii_lowercase())
        {
            Some(ext) => {
                if FONT_EXTENSION.contains(&ext) {
                    return Some(Box::new(FontFile::new(path)));
                }
                #[cfg(feature = "archive-zip")]
                if ext == "zip" {
                    return Some(Box::new(FontArchiveZip::new(path)));
                }
                #[cfg(feature = "archive-7z")]
                if ext == "7z" {
                    return Some(Box::new(FontArchive7z::new(path)));
                }
                #[cfg(feature = "archive-rar")]
                if ext == "rar" {
                    return Some(Box::new(FontArchiveRar::new(path)));
                }
                #[cfg(feature = "archive-tar")]
                if ext == "tar" {
                    return Some(Box::new(FontArchiveTar::new(path, TarCompression::None)));
                }
            }
            None => {
                return None;
            }
        }
        #[cfg(feature = "archive-tar")]
        {
            let path_lowercase = path.to_ascii_lowercase();
            for (ext, compression) in [
                ("tar.gz", TarCompression::GZ),
                ("tar.bz2", TarCompression::BZ2),
                ("tar.xz", TarCompression::XZ),
                ("tar.zst", TarCompression::ZSTD),
            ] {
                if path_lowercase.ends_with(ext) {
                    return Some(Box::new(FontArchiveTar::new(path, compression)));
                }
            }
        }
    }
    None
}
