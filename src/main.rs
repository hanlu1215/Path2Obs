use std::env;
use std::io::{IsTerminal, Write};
use std::path::PathBuf;
use std::process::ExitCode;

const IMAGE_EXTS: &[&str] = &[
    "png", "jpg", "jpeg", "jpe", "gif", "bmp", "svg", "webp", "tif", "tiff", "avif", "ico",
    "heic", "heif", "apng",
];

const VIDEO_EXTS: &[&str] = &[
    "mp4", "webm", "mov", "m4v", "mkv", "avi", "ogv", "ogg", "wmv", "flv", "ts", "mpg", "mpeg",
    "3gp",
];

const HELP: &str = "\
path2obs - 把本地图片/视频路径转成 Obsidian 嵌入标签，并写入剪贴板

用法:
  path2obs                          交互模式：直接粘贴路径回车，可连续转换
  path2obs <路径> [更多路径...]     直接转换给定路径
  path2obs -                        从管道读取路径（一行一个）

选项:
  --as=<image|video>                扩展名无法识别时强制指定类型
  --max-width=<像素>                 默认 1920
  --max-height=<像素>                默认 1080
  --no-copy, -n                     只打印，不写入剪贴板
  -h, --help                        显示帮助

交互模式下: 空行 / q / exit 退出

示例:
  path2obs
  path2obs \"G:\\PhD\\compare\\outputs\\s00.png\"
  path2obs 'D:\\sim\\demo.mp4' --max-width=1280
";

struct Options {
    max_width: u32,
    max_height: u32,
    copy: bool,
    force: Option<Kind>,
    inputs: Vec<String>,
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Image,
    Video,
}

fn main() -> ExitCode {
    let opts = match parse_args(env::args().skip(1).collect()) {
        Ok(o) => o,
        Err(msg) => {
            eprintln!("错误: {msg}\n");
            eprint!("{HELP}");
            return ExitCode::FAILURE;
        }
    };

    if opts.inputs.is_empty() {
        // 没有参数：区分「人对着终端粘贴」和「管道输入」
        if std::io::stdin().is_terminal() {
            return interactive(&opts);
        }
        return batch(piped_input(), &opts);
    }

    batch(opts.inputs.clone(), &opts)
}

/// 交互模式：反复读一行路径 -> 打印标签 -> 写剪贴板。
fn interactive(opts: &Options) -> ExitCode {
    eprintln!("path2obs 交互模式");
    eprintln!("粘贴图片/视频路径后回车即可转换并复制；空行、q 或 exit 退出。\n");

    let stdin = std::io::stdin();
    let mut line = String::new();
    let mut count = 0usize;

    loop {
        eprint!("路径> ");
        let _ = std::io::stderr().flush();

        line.clear();
        match stdin.read_line(&mut line) {
            Ok(0) => {
                eprintln!();
                break;
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("读取输入失败: {e}");
                return ExitCode::FAILURE;
            }
        }

        let trimmed = line.trim();
        if trimmed.is_empty()
            || matches!(
                trimmed.to_ascii_lowercase().as_str(),
                "q" | "quit" | "exit"
            )
        {
            break;
        }

        match render(trimmed, opts) {
            Ok(tag) => {
                println!("{tag}");
                if opts.copy {
                    match copy_to_clipboard(&tag) {
                        Ok(()) => eprintln!("[已复制到剪贴板]\n"),
                        Err(e) => eprintln!("[剪贴板写入失败: {e}]\n"),
                    }
                } else {
                    eprintln!();
                }
                count += 1;
            }
            Err(msg) => eprintln!("[未转换] {msg}\n"),
        }
    }

    eprintln!("已转换 {count} 项，退出。");
    ExitCode::SUCCESS
}

/// 管道 / 参数模式：一次性全部转换，结果整体写入剪贴板。
fn batch(inputs: Vec<String>, opts: &Options) -> ExitCode {
    let inputs: Vec<String> = inputs
        .into_iter()
        .flat_map(|i| if i == "-" { piped_input() } else { vec![i] })
        .collect();

    if inputs.is_empty() {
        eprintln!("错误: 没有读到任何路径。");
        eprint!("{HELP}");
        return ExitCode::FAILURE;
    }

    let mut out = String::new();
    let mut failed = false;

    for raw in &inputs {
        match render(raw, opts) {
            Ok(tag) => {
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str(&tag);
            }
            Err(msg) => {
                eprintln!("跳过 `{raw}`: {msg}");
                failed = true;
            }
        }
    }

    if out.is_empty() {
        return ExitCode::FAILURE;
    }

    println!("{out}");

    if opts.copy {
        match copy_to_clipboard(&out) {
            Ok(()) => eprintln!("已复制到剪贴板 ({} 个字符)", out.chars().count()),
            Err(e) => {
                eprintln!("写入剪贴板失败: {e}");
                return ExitCode::FAILURE;
            }
        }
    }

    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn piped_input() -> Vec<String> {
    let mut v = Vec::new();
    for line in std::io::stdin().lines() {
        match line {
            Ok(l) => push_line(&mut v, &l),
            Err(e) => {
                eprintln!("读取 stdin 失败: {e}");
                break;
            }
        }
    }
    v
}

fn push_line(out: &mut Vec<String>, line: &str) {
    let t = line.trim();
    if !t.is_empty() {
        out.push(t.to_string());
    }
}

fn copy_to_clipboard(text: &str) -> Result<(), String> {
    arboard::Clipboard::new()
        .and_then(|mut c| c.set_text(text.to_string()))
        .map_err(|e| e.to_string())
}

fn parse_args(args: Vec<String>) -> Result<Options, String> {
    let mut o = Options {
        max_width: 1920,
        max_height: 1080,
        copy: true,
        force: None,
        inputs: Vec::new(),
    };

    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        // 非选项（或单独的 `-`，表示从 stdin 读）一律当作路径
        if !a.starts_with('-') || a == "-" {
            o.inputs.push(a);
            continue;
        }
        match a.as_str() {
            "-h" | "--help" | "/?" => {
                print!("{HELP}");
                std::process::exit(0);
            }
            "-n" | "--no-copy" => o.copy = false,
            s if s.starts_with("--as=") => {
                o.force = Some(parse_kind(&s[5..])?);
            }
            s if s.starts_with("--max-width=") => {
                o.max_width = parse_u32(&s[12..]).ok_or("`--max-width` 需要正整数")?;
            }
            s if s.starts_with("--max-height=") => {
                o.max_height = parse_u32(&s[13..]).ok_or("`--max-height` 需要正整数")?;
            }
            "--as" | "--max-width" | "--max-height" => {
                let v = it.next().ok_or(format!("`{a}` 缺少取值"))?;
                match a.as_str() {
                    "--as" => o.force = Some(parse_kind(&v)?),
                    "--max-width" => {
                        o.max_width = parse_u32(&v).ok_or("`--max-width` 需要正整数")?
                    }
                    _ => o.max_height = parse_u32(&v).ok_or("`--max-height` 需要正整数")?,
                }
            }
            s => return Err(format!("未知选项: {s}")),
        }
    }

    Ok(o)
}

fn parse_kind(s: &str) -> Result<Kind, String> {
    match s {
        "image" | "img" | "picture" => Ok(Kind::Image),
        "video" | "movie" => Ok(Kind::Video),
        other => Err(format!("未知的 --as 取值: {other}")),
    }
}

fn parse_u32(s: &str) -> Option<u32> {
    let v: u32 = s.trim().parse().ok()?;
    if v == 0 {
        None
    } else {
        Some(v)
    }
}

/// 把单个路径渲染成 Obsidian 嵌入标签。
fn render(raw: &str, opts: &Options) -> Result<String, String> {
    // Explorer「复制为路径」会带双引号，顺手去掉
    let raw = raw.trim().trim_matches(|c| c == '"' || c == '\'');
    if raw.is_empty() {
        return Err("路径为空".into());
    }

    let path = normalize_path(raw)?;
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();

    let kind = match opts.force {
        Some(k) => k,
        None => {
            if IMAGE_EXTS.contains(&ext.as_str()) {
                Kind::Image
            } else if VIDEO_EXTS.contains(&ext.as_str()) {
                Kind::Video
            } else if ext.is_empty() {
                return Err("路径没有扩展名，无法判断类型（可用 --as=video 指定）".into());
            } else {
                return Err(format!(
                    "无法识别的扩展名 `.{ext}`（可用 --as=image 或 --as=video 指定）"
                ));
            }
        }
    };

    let url = to_file_url(&path);
    let style = format!(
        "width:100%; height:auto; max-width:{}px; max-height:{}px;",
        opts.max_width, opts.max_height
    );

    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_string();

    let media = match kind {
        Kind::Image => format!(
            "<img src=\"{}\" alt=\"{}\" style=\"{}\" loading=\"lazy\">",
            url,
            escape_attr(&name),
            style
        ),
        Kind::Video => format!(
            "<video src=\"{}\" style=\"{}\" controls preload=\"metadata\"></video>",
            url, style
        ),
    };

    Ok(format!("{WRAP_OPEN}{media}</div>"))
}

/// 外层居中容器，保证标签整体水平居中显示。
const WRAP_OPEN: &str = "<div style=\"display: flex; align-items: center; justify-content: center;\">";

/// 去掉 `file:///` 前缀、统一斜杠、把相对路径补全为绝对路径。
fn normalize_path(raw: &str) -> Result<PathBuf, String> {
    let mut s = raw.trim().to_string();

    if let Some(rest) = s.strip_prefix("file://") {
        s = rest.trim_start_matches('/').to_string();
        if s.len() >= 2 && s.as_bytes()[1] == b':' {
            // 形如 file:///C:/... 时前导斜杠被裁掉了，这里补回
            s.insert(0, '/');
        }
    } else if let Some(rest) = s.strip_prefix("file:") {
        s = rest.to_string();
    }

    // /C:/a/b -> C:/a/b
    if s.len() >= 3 && s.as_bytes()[0] == b'/' && s.as_bytes()[1].is_ascii_alphabetic() {
        if s.as_bytes()[2] == b':' {
            s.remove(0);
        }
    }

    s = s.replace('\\', "/");

    let path = PathBuf::from(&s);
    let path = if is_absolute(&s) {
        path
    } else {
        env::current_dir()
            .map_err(|e| format!("无法获取当前目录: {e}"))?
            .join(path)
    };

    Ok(path)
}

fn is_absolute(s: &str) -> bool {
    let b = s.as_bytes();
    if s.starts_with("//") {
        return true; // UNC
    }
    b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':'
}

/// 生成 file:// URL，对空格 / 中文等非 ASCII 字符做百分号编码。
fn to_file_url(path: &PathBuf) -> String {
    let s = path.to_string_lossy().replace('\\', "/");

    let (prefix, rest) = if s.starts_with("//") {
        ("file:", s.as_str())
    } else {
        ("file:///", s.trim_start_matches('/'))
    };

    let mut out = String::with_capacity(s.len() + 16);
    out.push_str(prefix);
    for b in rest.bytes() {
        let unreserved =
            b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~' | b'/' | b':');
        if unreserved {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts() -> Options {
        Options {
            max_width: 1920,
            max_height: 1080,
            copy: false,
            force: None,
            inputs: vec![],
        }
    }

    #[test]
    fn image_tag() {
        let got = render("G:\\a b\\中文.png", &opts()).unwrap();
        assert!(got.starts_with(WRAP_OPEN));
        assert!(got.contains("<img src=\"file:///G:/a%20b/"));
        assert!(got.ends_with("</div>"));
        assert!(got.contains("max-width:1920px"));
    }

    #[test]
    fn video_tag() {
        let got = render("D:\\sim\\demo.mp4", &opts()).unwrap();
        assert_eq!(
            got,
            format!(
                "{WRAP_OPEN}<video src=\"file:///D:/sim/demo.mp4\" style=\"width:100%; \
                 height:auto; max-width:1920px; max-height:1080px;\" controls \
                 preload=\"metadata\"></video></div>"
            )
        );
    }

    #[test]
    fn quoted_explorer_path() {
        let got = render("\"G:\\clips\\a.mov\"", &opts()).unwrap();
        assert!(got.contains("<video src=\"file:///G:/clips/a.mov\""));
        assert!(!got.contains("clips\\a"));
    }

    #[test]
    fn strips_file_prefix() {
        let got = render("file:///G:/x/y.mp4", &opts()).unwrap();
        assert!(got.contains("file:///G:/x/y.mp4"));
    }

    #[test]
    fn unknown_ext() {
        assert!(render("a.txt", &opts()).is_err());
        let mut o = opts();
        o.force = Some(Kind::Video);
        assert!(render("a.txt", &o).unwrap().contains("<video"));
    }
}
