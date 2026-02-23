use log::{debug, error, info, warn};
use std::io::{self, Read};
use windows::Win32::UI::WindowsAndMessaging::{
    HWND_BROADCAST, MB_ICONINFORMATION, MB_OK, MessageBoxW, SendMessageW, WM_FONTCHANGE,
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

    info!(
        "{} v{} by {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS")
    );
    info!("Repository: {}", env!("CARGO_PKG_REPOSITORY"));

    let args: Vec<String> = if cfg!(debug_assertions) {
        vec![
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

    font_sources.retain_mut(|fs| match fs.load() {
        Ok(_) => {
            info!("Loaded font from \"{}\"", fs.get_path());
            return true;
        }
        Err(err) => {
            error!("Failed to load font from \"{}\": {err}", fs.get_path());
            return false;
        }
    });
    debug!("Call SendMessageW WM_FONTCHANGE");
    unsafe {
        SendMessageW(HWND_BROADCAST, WM_FONTCHANGE, None, None);
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

    font_sources.iter().for_each(|fs| {
        fs.unload();
        info!("Unloaded font from \"{}\"", fs.get_path());
    });
    debug!("Call SendMessageW WM_FONTCHANGE");
    unsafe {
        SendMessageW(HWND_BROADCAST, WM_FONTCHANGE, None, None);
    }
}
