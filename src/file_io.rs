use std::path::Path;

use crate::encoding::{read_with_encoding, write_with_encoding, FileEncoding};

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

pub fn path_exists(path: &Path) -> bool {
    path.exists()
}
