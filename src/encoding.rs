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

    pub fn insert_text(self) -> &'static str {
        match self {
            Tabulation::Spaces2 => "  ",
            Tabulation::Spaces4 => "    ",
            Tabulation::Spaces8 => "        ",
            Tabulation::TabLiteral => "\t",
        }
    }
}

pub fn read_with_encoding(path: &std::path::Path, enc: FileEncoding) -> std::io::Result<Vec<String>> {
    let bytes = std::fs::read(path)?;
    let content = decode_bytes(&bytes, enc)?;
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
    let text = lines.join("\n");
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
