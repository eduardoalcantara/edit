use std::path::{Path, PathBuf};

use crate::encoding::{
    read_with_encoding, write_text_with_encoding, write_with_encoding, FileEncoding,
};

pub fn read_lines(path: &Path) -> std::io::Result<Vec<String>> {
    read_with_encoding(path, FileEncoding::Utf8)
}

pub fn read_lines_encoded(path: &Path, enc: FileEncoding) -> std::io::Result<Vec<String>> {
    read_with_encoding(path, enc)
}

pub fn write_lines(path: &Path, lines: &[String]) -> std::io::Result<()> {
    write_with_encoding(path, lines, FileEncoding::Utf8)
}

pub fn write_lines_encoded(path: &Path, lines: &[String], enc: FileEncoding) -> std::io::Result<()> {
    write_with_encoding(path, lines, enc)
}

pub fn write_content_encoded(path: &Path, content: &str, enc: FileEncoding) -> std::io::Result<()> {
    write_text_with_encoding(path, content, enc)
}

pub fn path_exists(path: &Path) -> bool {
    path.exists()
}

/// Caminho absoluto para abrir arquivo; usa `canonicalize` quando o alvo existe no FS.
pub fn normalize_open_path(path: &Path) -> PathBuf {
    let absolute = absolute_path(path);
    std::fs::canonicalize(&absolute).unwrap_or(absolute)
}

/// Dois caminhos referem-se ao mesmo arquivo no disco (case-insensitive no Windows).
pub fn same_file_path(a: &Path, b: &Path) -> bool {
    if paths_equal(a, b) {
        return true;
    }
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(ca), Ok(cb)) => paths_equal(&ca, &cb),
        _ => paths_equal(&absolute_path(a), &absolute_path(b)),
    }
}

fn absolute_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}

#[cfg(windows)]
fn paths_equal(a: &Path, b: &Path) -> bool {
    a.as_os_str()
        .eq_ignore_ascii_case(b.as_os_str())
}

#[cfg(not(windows))]
fn paths_equal(a: &Path, b: &Path) -> bool {
    a == b
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn normalize_makes_relative_absolute() {
        let cwd = std::env::current_dir().expect("cwd");
        let normalized = normalize_open_path(Path::new("Cargo.toml"));
        assert!(normalized.is_absolute());
        assert!(normalized.ends_with("Cargo.toml") || normalized.ends_with("cargo.toml"));
        let _ = cwd;
    }

    #[test]
    fn same_file_path_matches_canonical_pair() {
        let dir = std::env::temp_dir().join(format!("edit-path-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("mkdir");
        let file_path = dir.join("sample.txt");
        fs::File::create(&file_path)
            .and_then(|mut f| f.write_all(b"x"))
            .expect("create");

        let via_dir = dir.join("./sample.txt");
        assert!(same_file_path(&file_path, &via_dir));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_content_round_trip_via_editor_matches_disk() {
        use crate::editor::Editor;
        use crate::theme::ThemeId;

        let dir = std::env::temp_dir().join(format!("edit-save-rt-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("doc.txt");
        let palette = ThemeId::ClassicBlue.palette();
        let original = "alfa\n\nbeta\ngamma";
        write_content_encoded(&path, original, FileEncoding::Utf8).unwrap();

        for _ in 0..3 {
            let lines = read_lines_encoded(&path, FileEncoding::Utf8).unwrap();
            let mut editor = Editor::new(&palette);
            editor.set_lines(lines);
            write_content_encoded(&path, &editor.content_string(), FileEncoding::Utf8).unwrap();
        }

        let final_lines = read_lines_encoded(&path, FileEncoding::Utf8).unwrap();
        assert_eq!(final_lines.join("\n"), original);
        let _ = fs::remove_dir_all(&dir);
    }

    #[cfg(windows)]
    #[test]
    fn same_file_path_ignores_case_on_windows() {
        let dir = std::env::temp_dir().join(format!("edit-case-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("mkdir");
        let lower = dir.join("readme.md");
        fs::File::create(&lower)
            .and_then(|mut f| f.write_all(b"#"))
            .expect("create");
        let upper = dir.join("README.MD");
        assert!(same_file_path(&lower, &upper));

        let _ = fs::remove_dir_all(&dir);
    }
}
