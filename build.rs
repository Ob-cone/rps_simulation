fn main() {
    // 타겟 패밀리가 wasm이 아니고, 타겟 OS가 windows일 때만 실행
    #[cfg(all(target_os = "windows", not(target_family = "wasm")))]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("rps_icon.ico");
        if let Err(e) = res.compile() {
            eprintln!("Windows resource compilation skipped or failed: {}", e);
        }
    }
}
