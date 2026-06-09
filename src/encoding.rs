#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileEncoding {
    Utf8,
    Utf8NoBom,
    Utf16Le,
    Utf16Be,
    Iso88591,
    Ansi,
}

impl Default for FileEncoding {
    fn default() -> Self {
        FileEncoding::Utf8
    }
}

impl FileEncoding {
    pub fn label(self) -> &'static str {
        match self {
            FileEncoding::Utf8 => "UTF-8",
            FileEncoding::Utf8NoBom => "UTF-8 sem BOM",
            FileEncoding::Utf16Le => "UTF-16 LE",
            FileEncoding::Utf16Be => "UTF-16 BE",
            FileEncoding::Iso88591 => "ISO-8859-1",
            FileEncoding::Ansi => "ANSI",
        }
    }

    pub fn all() -> &'static [FileEncoding] {
        &[
            FileEncoding::Utf8,
            FileEncoding::Utf8NoBom,
            FileEncoding::Utf16Le,
            FileEncoding::Utf16Be,
            FileEncoding::Iso88591,
            FileEncoding::Ansi,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tabulation {
    Spaces2,
    Spaces4,
    Spaces8,
    TabLiteral,
}

impl Default for Tabulation {
    fn default() -> Self {
        Tabulation::Spaces4
    }
}

impl Tabulation {
    pub fn label(self) -> &'static str {
        match self {
            Tabulation::Spaces2 => "2",
            Tabulation::Spaces4 => "4",
            Tabulation::Spaces8 => "8",
            Tabulation::TabLiteral => "Tab",
        }
    }

    /// Rótulo legível na barra de status (ex.: `Tab 4`; literal = `Tab`).
    pub fn footer_label(self) -> &'static str {
        match self {
            Tabulation::Spaces2 => "Tab 2",
            Tabulation::Spaces4 => "Tab 4",
            Tabulation::Spaces8 => "Tab 8",
            Tabulation::TabLiteral => "Tab",
        }
    }

    pub fn insert_text(self) -> &'static str {
        match self {
            Tabulation::Spaces2 => "  ",
            Tabulation::Spaces4 => "    ",
            Tabulation::Spaces8 => "        ",
            Tabulation::TabLiteral => "\t",
        }
    }

    /// Rótulo no modal de conversão de tabulação.
    pub fn convert_option_label(self) -> &'static str {
        match self {
            Tabulation::Spaces2 => "2 espaços",
            Tabulation::Spaces4 => "4 espaços",
            Tabulation::Spaces8 => "8 espaços",
            Tabulation::TabLiteral => "Tab literal",
        }
    }

    /// Parada usada em Tab → Espaços / Espaços → Tab (2, 4 ou 8; literal = 8).
    pub fn convert_stop_width(self) -> usize {
        match self {
            Tabulation::Spaces2 => 2,
            Tabulation::Spaces4 => 4,
            Tabulation::Spaces8 | Tabulation::TabLiteral => 8,
        }
    }
}

/// Converte `\r\n` e `\r` solto em `\n` para round-trip estável no Windows.
pub fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

pub fn read_with_encoding(path: &std::path::Path, enc: FileEncoding) -> std::io::Result<Vec<String>> {
    let bytes = std::fs::read(path)?;
    let content = normalize_newlines(&decode_bytes(&bytes, enc)?);
    if content.is_empty() {
        return Ok(vec![String::new()]);
    }
    Ok(content.lines().map(str::to_string).collect())
}

pub fn write_with_encoding(
    path: &std::path::Path,
    lines: &[String],
    enc: FileEncoding,
) -> std::io::Result<()> {
    write_text_with_encoding(path, &lines.join("\n"), enc)
}

pub fn write_text_with_encoding(
    path: &std::path::Path,
    text: &str,
    enc: FileEncoding,
) -> std::io::Result<()> {
    let text = normalize_newlines(text);
    let bytes = encode_text(&text, enc)?;
    std::fs::write(path, bytes)
}

fn decode_bytes(bytes: &[u8], enc: FileEncoding) -> std::io::Result<String> {
    match enc {
        FileEncoding::Utf8 | FileEncoding::Utf8NoBom => {
            String::from_utf8(bytes.to_vec()).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        }
        FileEncoding::Utf16Le => decode_utf16(bytes, true),
        FileEncoding::Utf16Be => decode_utf16(bytes, false),
        FileEncoding::Iso88591 | FileEncoding::Ansi => Ok(bytes.iter().map(|&b| b as char).collect()),
    }
}

fn decode_utf16(bytes: &[u8], le: bool) -> std::io::Result<String> {
    if bytes.len() % 2 != 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "UTF-16: tamanho ímpar",
        ));
    }
    let u16s: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| {
            if le {
                u16::from_le_bytes([c[0], c[1]])
            } else {
                u16::from_be_bytes([c[0], c[1]])
            }
        })
        .collect();
    String::from_utf16(&u16s).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_newlines_strips_cr() {
        assert_eq!(normalize_newlines("a\r\nb\rc"), "a\nb\nc");
    }

    #[test]
    fn read_write_round_trip_preserves_line_count() {
        let dir = std::env::temp_dir().join(format!("edit-io-rt-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("sample.md");
        let original = "# Título\n\nParágrafo\nLinha final";
        write_text_with_encoding(&path, original, FileEncoding::Utf8).unwrap();
        let initial = read_with_encoding(&path, FileEncoding::Utf8).unwrap();
        for _ in 0..3 {
            let lines = read_with_encoding(&path, FileEncoding::Utf8).unwrap();
            write_text_with_encoding(&path, &lines.join("\n"), FileEncoding::Utf8).unwrap();
        }
        let final_lines = read_with_encoding(&path, FileEncoding::Utf8).unwrap();
        assert_eq!(final_lines.len(), initial.len());
        assert_eq!(
            final_lines.join("\n"),
            normalize_newlines(original),
            "conteúdo deve permanecer estável após ciclos save/load"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn crlf_on_disk_normalizes_without_blank_line_growth() {
        let dir = std::env::temp_dir().join(format!("edit-crlf-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("crlf.txt");
        std::fs::write(&path, b"linha1\r\nlinha2\r\n").unwrap();
        let lines = read_with_encoding(&path, FileEncoding::Utf8).unwrap();
        assert_eq!(lines, vec!["linha1".to_string(), "linha2".to_string()]);
        write_text_with_encoding(&path, &lines.join("\n"), FileEncoding::Utf8).unwrap();
        let again = read_with_encoding(&path, FileEncoding::Utf8).unwrap();
        assert_eq!(again.len(), 2);
        let _ = std::fs::remove_dir_all(&dir);
    }
}

fn encode_text(text: &str, enc: FileEncoding) -> std::io::Result<Vec<u8>> {
    match enc {
        FileEncoding::Utf8 | FileEncoding::Utf8NoBom | FileEncoding::Ansi | FileEncoding::Iso88591 => {
            if matches!(enc, FileEncoding::Iso88591 | FileEncoding::Ansi) {
                Ok(text.chars().map(|c| c as u8).collect())
            } else {
                Ok(text.as_bytes().to_vec())
            }
        }
        FileEncoding::Utf16Le => {
            let mut out = vec![0xFF, 0xFE];
            for u in text.encode_utf16() {
                out.extend_from_slice(&u.to_le_bytes());
            }
            Ok(out)
        }
        FileEncoding::Utf16Be => {
            let mut out = vec![0xFE, 0xFF];
            for u in text.encode_utf16() {
                out.extend_from_slice(&u.to_be_bytes());
            }
            Ok(out)
        }
    }
}
