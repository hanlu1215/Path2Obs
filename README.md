# path2obs

![logo](assets/logo.svg)

把本地**图片 / 视频路径**转成 **Obsidian 可直接渲染的 HTML 嵌入标签**，结果自动写入剪贴板，回到 Obsidian 里 `Ctrl+V` 即可。

```
路径 ──► path2obs ──► <video src="file:///..."></video>  (已在剪贴板)
```

## 1. 编译

需要 Rust 工具链（`cargo`）。

```bash
cd Path2Obs
cargo build --release
```

产物：`target\release\path2obs.exe`（约 160 KB，无额外依赖）。

建议把它所在目录加入 `PATH`，或把 exe 复制到已在 `PATH` 的目录，之后在任何地方都能直接敲 `path2obs`。

## 2. 基本用法（交互模式）

这个程序最常用的方式就是**先运行，再往里粘贴路径**：

```bash
path2obs
```

```
path2obs 交互模式
粘贴图片/视频路径后回车即可转换并复制；空行、q 或 exit 退出。

路径> G:\PhD_Period\2025_TetheredAerialRobot\TAR_MUJOCO\schemes\compare\outputs\s00_s03_s04_s06.png
<div style="display: flex; align-items: center; justify-content: center;"><img src="file:///G:/PhD_Period/.../s00_s03_s04_s06.png" alt="s00_s03_s04_s06.png" style="width:100%; height:auto; max-width:1920px; max-height:1080px;" loading="lazy"></div>
[已复制到剪贴板]

路径> G:\PhD_Period\...\result_s00\s03_eval_sequence.mp4
<div style="display: flex; align-items: center; justify-content: center;"><video src="file:///G:/PhD_Period/.../s03_eval_sequence.mp4" style="width:100%; height:auto; max-width:1920px; max-height:1080px;" controls preload="metadata"></video></div>
[已复制到剪贴板]

路径>
已转换 2 项，退出。
```

- 粘贴方式：Windows Terminal / CMD 里 `右键` 或 `Ctrl+V` 都可以；也可以直接把文件从资源管理器**拖进**终端窗口。
- 每转换一条就复制一次，可以连续处理多个文件。
- 直接粘贴资源管理器「复制为路径」得到的带双引号路径也没问题，引号会自动去掉。
- 退出：输入空行、`q`、`exit`，或按 `Ctrl+C`。

## 3. 直接带参数（批量）

```bash
path2obs "G:\sim\a.png"
path2obs "G:\sim\a.png" "G:\sim\b.mp4"        # 多个文件，标签按行拼接后一次复制
path2obs "G:\sim\b.mp4" --max-width=1280
```

也可以走管道（一行一个路径）：

```bash
type paths.txt | path2obs
path2obs - < paths.txt
```

## 4. 选项

| 选项 | 说明 |
| --- | --- |
| `--as=image` / `--as=video` | 扩展名无法识别时强制指定类型 |
| `--max-width=<像素>` | 默认 `1920` |
| `--max-height=<像素>` | 默认 `1080` |
| `-n`, `--no-copy` | 只打印标签，不写剪贴板 |
| `-h`, `--help` | 显示帮助 |

## 5. 生成规则

输出统一用居中容器包一层，保证在笔记里水平居中：

```html
<div style="display: flex; align-items: center; justify-content: center;">...媒体标签...</div>
```

**图片**（扩展名：`png jpg jpeg gif bmp svg webp tif tiff avif ico heic heif apng`）

```html
<div style="display: flex; align-items: center; justify-content: center;"><img src="file:///G:/path/a.png" alt="a.png" style="width:100%; height:auto; max-width:1920px; max-height:1080px;" loading="lazy"></div>
```

**视频**（扩展名：`mp4 webm mov m4v mkv avi ogv ogg wmv flv ts mpg mpeg 3gp`）

```html
<div style="display: flex; align-items: center; justify-content: center;"><video src="file:///G:/path/a.mp4" style="width:100%; height:auto; max-width:1920px; max-height:1080px;" controls preload="metadata"></video></div>
```

路径处理细节：

- 反斜杠统一转成正斜杠，生成标准 `file:///G:/...` 形式（Obsidian / Chromium 内核可正常加载）；
- 空格、中文等字符按 UTF-8 做百分号编码（`a b` → `a%20b`，`中文` → `%E4%B8%AD%E6%96%87`）；
- 支持 UNC 路径（`\\NAS\share\x.png` → `file://NAS/share/x.png`）；
- 已经是 `file:///...` 的输入会被识别，不会重复加前缀；
- 相对路径会补全成当前目录下的绝对路径；
- 扩展名无法识别时报错，代码不会瞎猜（可用 `--as=` 覆盖）。

## 6. Logo / 图标

紫色水晶（Obsidian 标志的简化变形）+ 一支射入水晶的箭头，寓意「路径 → Obsidian」。

| 文件 | 用途 |
| --- | --- |
| `assets/logo.svg` | 矢量源文件（README、网页、Obsidian 内嵌） |
| `assets/logo.png` | 512×512 PNG |
| `assets/logo.ico` | 多尺寸（16/24/32/48/64/128/256）Windows 图标 |

- `build.rs` 会在 Windows 上通过 `winres` 把 `assets/logo.ico` 嵌进 exe，资源管理器、任务栏、快捷方式都会显示这个图标；若机器上没有资源编译器（`rc.exe` / `windres`），构建只会打印一条 warning，不影响使用。
- 想改配色或形状：编辑 `tools/make_logo.py` 里的常量后重新生成（`assets/logo.svg` 需手工同步）：

```bash
python tools/make_logo.py
cargo build --release
```

## 7. 小技巧

在 PowerShell 里加个别名（写入 `$PROFILE`）：

```powershell
Set-Alias p2o "C:\tools\path2obs.exe"
```

在 Obsidian 里：粘贴到笔记的阅读视图能直接播放/显示；编辑模式下 HTML 标签同样会被渲染。若视频太大不想自动加载，`preload="metadata"` 已保证只取元数据。

## 8. 退出码

| 码 | 含义 |
| --- | --- |
| `0` | 全部成功 |
| `1` | 有路径转换失败 / 读不到输入 / 剪贴板写入失败（错误信息在 stderr） |

## 9. 测试

```bash
cargo test
```

## 10. 自动构建 & 发布 exe

`.github/workflows/release.yml` 提供了手动触发的 GitHub Actions：

1. 打开仓库 → **Actions** → 选 **Build & Release (Windows exe)** → **Run workflow**；
2. 输入 `tag`（如 `v0.1.0`，不能与已有标签重复），可勾选 prerelease；
3. 工作流在 `windows-latest` 上 `cargo build --release --locked`，跑一遍测试，上传构建产物，并用该 tag 创建 Release，附件包含：
   - `path2obs.exe`
   - `path2obs.exe.sha256`（校验用：`certutil -hashfile path2obs.exe SHA256`）

> 仓库需要开启 Actions 的写权限（`permissions: contents: write` 已在 workflow 中声明）。Release 说明会自动从提交记录生成。
