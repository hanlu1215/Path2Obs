// 把紫色 logo 作为 exe 图标与版本信息嵌入（仅 Windows）。
// 没有资源编译器（Windows SDK 的 rc.exe / MinGW 的 windres）时只打印提示，不中断构建。
fn main() {
    println!("cargo:rerun-if-changed=assets/logo.ico");

    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/logo.ico");
        res.set("ProductName", "path2obs");
        res.set("FileDescription", "路径转 Obsidian 嵌入标签");
        if let Err(e) = res.compile() {
            println!("cargo:warning=嵌入 exe 图标失败，已跳过: {e}");
        }
    }
}
