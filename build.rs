use std::{
    error::Error,
    fs::{DirEntry, File, ReadDir},
    io::{Read, Write},
    process::exit,
};

fn generate_pe_header() {
    use time::OffsetDateTime;

    let today = OffsetDateTime::now_utc();
    let copyright = format!("Copyright © 2017-{} Vincent Prouillet", today.year());
    let mut res = winres::WindowsResource::new();
    // needed for MinGW cross-compiling
    if cfg!(unix) {
        res.set_windres_path("x86_64-w64-mingw32-windres");
    }
    res.set_icon("docs/static/favicon.ico");
    res.set("LegalCopyright", &copyright);
    res.compile().expect("Failed to compile Windows resources!");
}

fn generate_theme_list() -> Option<()> {
    fn extract_entry(buffer: &mut String, dir: std::io::Result<DirEntry>) -> Option<()> {
        let dir = dir.ok()?;
        let dir_name = dir.file_name().into_string().ok()?;
        let path = dir.path();
        if path.is_dir() {
            let mut index = File::open(format!("{}\\index.md", path.to_str()?)).ok()?;
            let mut buf = String::new();
            index.read_to_string(&mut buf).ok()?;
            let repo_start = buf.find("repository")?;
            let repo_end = buf[repo_start..].find("\n")? + repo_start - 1;
            let repo_line = &buf[repo_start..repo_end];
            let repo = repo_line.split("=").nth(1)?.trim();
            *buffer += &format!("(\"{}\",{}),\n", dir_name, repo);
        }
        Some(())
    }

    let mut output = String::with_capacity(8 * 1024); // 8KB
    output += "pub const THEME_LIST: &[(&str, &str)] = &[\n";

    for dir in std::fs::read_dir(r".\docs\content\themes").ok()? {
        extract_entry(&mut output, dir);
    }
    output += "];";
    let mut file = File::create("src\\themes_list.rs").ok()?;
    write!(file, "{output}").ok()?;

    Some(())
}

fn main() {
    generate_theme_list();
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() != "windows"
        && std::env::var("PROFILE").unwrap() != "release"
    {
        return;
    }
    if cfg!(windows) {
        generate_pe_header();
    }
}
