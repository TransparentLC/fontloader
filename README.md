# fontloader

https://github.com/user-attachments/assets/291135a4-ff71-4599-b53d-193e628e69a9

在使用外挂字幕、编辑文档、编辑图像时可能需要用到很多额外的字体。如果这些字体的使用频率很低又要一一安装它们的话，系统字体库会变得越来越臃肿，不仅浪费系统盘空间，而且很多软件在加载大量的字体列表时也会出现卡顿。

这个小工具可以在 Windows 上按需临时加载字体。

使用方法：

1. 从 Release 下载 `fontloader.exe`
2. 将需要使用的字体文件拖到 `fontloader.exe` 上，可以一次拖拽多个字体文件
3. 也可以将文件夹或 ZIP、7z、RAR、tar[.{gz,bz2,xz,zst}\] 压缩包拖到 `fontloader.exe` 上，此时会遍历加载里面的所有字体
4. 使用完字体后，在终端中按 <kbd>Enter</kbd> 卸载字体

When working with external subtitles, editing documents, or editing images, you may need many additional fonts. If these fonts are used infrequently and you install them one by one, your system font folder will be cluttered, not only wasting system drive space but also causing many software applications to lag when loading large font lists.

This tool allows you to temporarily load fonts on demand on Windows.

Usage:

1. Download `fontloader.exe` from release.
2. Drag the fonts you need onto `fontloader.exe`. You can drag multiple fonts at once.
3. You can also drag folders or ZIP, 7z, RAR, tar[.{gz,bz2,xz,zst}] archives onto `fontloader.exe`, it will traverse and load all fonts inside.
4. After using the fonts, press <kbd>Enter</kbd> in the terminal to unload them.

## 开发

需要安装 Nightly Rust 和 `rustup component add rust-src`，因为使用了 [min-sized-rust](https://github.com/johnthagen/min-sized-rust) 来减少可执行文件大小。

```sh
cargo build --release -Z build-std=std,panic_abort -Z build-std-features=optimize_for_size && upx --ultra-brute target/release/fontloader.exe
```

Commit message 需要符合 [Conventional Commits](https://www.conventionalcommits.org/zh-hans/v1.0.0/) 规范。

### 不会考虑的功能

* 适配 Windows 以外的系统（真的需要这个吗？）
* 读取和展示字体的 PostScript 名称
* 在遍历字体时根据 OTF/TTF 等字体类型、文件名、PostScript 名称、是否已安装等进行筛选
* 处理带密码或分卷的压缩包
* 从文件夹/压缩包中的压缩包加载字体

## 一些碎碎念

本项目参考自 cryptw 在 Bitbucket 上开源的同名工具 [FontLoader](https://bitbucket.org/cryptw/fontloader)，遗憾的是作者已经删库了，可以在 Wayback Machine 上看看它的[快照](https://web.archive.org/web/20130412161755/https://bitbucket.org/cryptw/fontloader)。

不过我找到了 [sfc9982/FontLoader](https://github.com/sfc9982/FontLoader) 这个备份，因此可以参考它的源代码。

如果你需要原版 FontLoader 的可执行文件，可以从 VCB-Studio 的“超级字体整合包 XZ”的[下载页面](https://pan.acgrip.com/?dir=%E8%B6%85%E7%BA%A7%E5%AD%97%E4%BD%93%E6%95%B4%E5%90%88%E5%8C%85%20XZ)下载，实际上我就是从这里知道这个工具的。

遍历文件夹和压缩包是原版 FontLoader 没有实现的功能。对于压缩包，会先把里面的字体解压到临时目录然后再加载。

cryptw 的原始版本在加载字体后没有[调用 `SendMessageW` 发送 `WM_FONTCHANGE`](https://learn.microsoft.com/zh-cn/windows/win32/gdi/wm-fontchange)，因此其他程序可能无法得知字体变更，这个问题最大的影响是在 Photoshop 等程序中需要重新启动才能使用加载的字体，比较麻烦。sfc9982 的备份和这个项目都加上了 `SendMessageW` 修正这个问题。不过，有些程序可能没有对这一点进行适配，此时仍然需要重新启动才能使用加载的字体。

加载字体后对一些应用的测试：

* Adobe 全家桶：可以立即使用
* Office 全家桶：可以立即使用
* Aegisub：字体列表中不会出现加载的字体，但是手动输入字体名称仍然可以使用
* MPC-HC：需要重启才能使用加载的字体渲染外挂字幕
