use cfg_if::cfg_if;
use log::{debug, error, info, warn};
#[cfg(feature = "parallel")]
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::{
    io::{self, Read},
    time::Instant,
};
use windows::Win32::{
    Foundation::{LPARAM, WPARAM},
    UI::WindowsAndMessaging::{
        HWND_BROADCAST, MB_ICONINFORMATION, MB_OK, MessageBoxW, PostMessageW, WM_FONTCHANGE,
    },
};
use windows_strings::h;

mod font_source;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().filter_or(
        env_logger::DEFAULT_FILTER_ENV,
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "info"
        },
    ))
    .try_init()
    .unwrap();
    let mut stdin = io::stdin();

    #[cfg(feature = "parallel")]
    if std::env::var("RAYON_NUM_THREADS").is_err() {
        if let Ok(parallelism) = std::thread::available_parallelism() {
            let parallelism = parallelism.get();
            let threads = parallelism * 4;

            match rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build_global()
            {
                Ok(_) => {
                    debug!("Build thread pool with threads: {threads} Parallelism: {parallelism}");
                }
                Err(err) => {
                    warn!("Failed to build thread pool: {err}");
                }
            };
        }
    }

    info!(
        "{} v{} by {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS")
    );
    info!("Repository: {}", env!("CARGO_PKG_REPOSITORY"));

    let args: Vec<String> = if cfg!(debug_assertions) {
        vec![
            "sample/DreamHanSerif".to_string(),
            "sample/DreamHanSans".to_string(),
            // "sample/RenOuFangSong-16.ttf".to_string(),
            // "sample/游趣体0.812".to_string(),
            // "sample/阿里健康体2.0_猫啃网.zip".to_string(),
            // "sample/械黑GB_1.100_猫啃网.7z".to_string(),
            // "sample/权衡度量体_猫啃网.rar".to_string(),
            // "sample/荆南圆体(けいなん丸ポップ体)_猫啃网.tar".to_string(),
            // "sample/荆南圆体(けいなん丸ポップ体)_猫啃网.tar.gz".to_string(),
            // "sample/荆南圆体(けいなん丸ポップ体)_猫啃网.tar.bz2".to_string(),
            // "sample/荆南圆体(けいなん丸ポップ体)_猫啃网.tar.xz".to_string(),
            // "sample/荆南圆体(けいなん丸ポップ体)_猫啃网.tar.zst".to_string(),
        ]
    } else {
        std::env::args().skip(1).collect()
    };
    if args.len() == 0 {
        warn!("No input file");
        unsafe {
            MessageBoxW(
                None,
                h!("Drag and drop font file(s) onto the executable file."),
                h!("Usage"),
                MB_OK | MB_ICONINFORMATION,
            )
        };
        return;
    }

    let mut font_sources = Vec::new();
    for arg in args {
        match font_source::from_path(arg.clone()) {
            Some(fs) => {
                font_sources.push(fs);
            }
            None => {
                warn!("Unable to handle \"{arg}\"");
            }
        }
    }

    let start = Instant::now();
    cfg_if! {
        if #[cfg(feature = "parallel")] {
            let iter = font_sources.into_par_iter();
        } else {
            let iter = font_sources.into_iter();
        }
    }
    font_sources = iter
        .filter_map(|mut fs| match fs.load() {
            Ok(_) => {
                info!("Loaded font from \"{}\"", fs.get_path());
                Some(fs)
            }
            Err(err) => {
                error!("Failed to font load from \"{}\": {err}", fs.get_path());
                None
            }
        })
        .collect();
    info!("Loaded font in {}s", start.elapsed().as_secs_f64());
    debug!("Call PostMessageW WM_FONTCHANGE");
    unsafe {
        if let Err(err) = PostMessageW(Some(HWND_BROADCAST), WM_FONTCHANGE, WPARAM(0), LPARAM(0)) {
            warn!("Failed to call PostMessageW WM_FONTCHANGE: {err}");
        }
    }

    warn!("Press ENTER to unload fonts");
    let _ = stdin.read(&mut [0u8]).unwrap();
    // unsafe {
    //     MessageBoxW(
    //         None,
    //         &HSTRING::from(
    //             String::from("Close this message box to unload fonts from:\n\n")
    //                 + &font_sources
    //                     .iter()
    //                     .map(|fs| fs.get_path().clone())
    //                     .collect::<Vec<String>>()
    //                     .join("\n"),
    //         ),
    //         h!("Fontloader"),
    //         MB_OK | MB_ICONINFORMATION,
    //     );
    // };

    let start = Instant::now();
    cfg_if! {
        if #[cfg(feature = "parallel")] {
            let iter = font_sources.par_iter();
        } else {
            let iter = font_sources.iter();
        }
    }
    iter.for_each(|fs| {
        fs.unload();
        info!("Unloaded font from \"{}\"", fs.get_path());
    });
    info!("Unloaded font in {}s", start.elapsed().as_secs_f64());
    debug!("Call PostMessageW WM_FONTCHANGE");
    unsafe {
        if let Err(err) = PostMessageW(Some(HWND_BROADCAST), WM_FONTCHANGE, WPARAM(0), LPARAM(0)) {
            warn!("Failed to call PostMessageW WM_FONTCHANGE: {err}");
        }
    }
}
