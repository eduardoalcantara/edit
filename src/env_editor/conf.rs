//! Parse/serialize de ficheiros `NOME=valor` (environment.d).

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvVar {
    pub name: String,
    pub value: String,
}

/// Nome válido: começa com letra ou `_`, resto alfanumérico ou `_`.
pub fn validate_var_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Interpreta conteúdo estilo `environment.d` / `/etc/environment`.
/// Ignora linhas vazias e comentários `#`. Mantém a primeira ocorrência de cada nome.
pub fn parse_conf(text: &str) -> Vec<EnvVar> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (name, value) = match line.split_once('=') {
            Some((n, v)) => (n.trim(), v.trim()),
            None => continue,
        };
        if name.is_empty() || !validate_var_name(name) {
            continue;
        }
        if !seen.insert(name.to_string()) {
            continue;
        }
        let value = strip_quotes(value);
        out.push(EnvVar {
            name: name.to_string(),
            value,
        });
    }
    out
}

fn strip_quotes(value: &str) -> String {
    let bytes = value.as_bytes();
    if bytes.len() >= 2 {
        let first = bytes[0];
        let last = bytes[bytes.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return value[1..value.len() - 1].to_string();
        }
    }
    value.to_string()
}

pub fn serialize_conf(vars: &[EnvVar]) -> String {
    let mut lines = Vec::with_capacity(vars.len() + 2);
    lines.push("# Gerido pelo edit — não editar .bashrc aqui".to_string());
    for var in vars {
        if !validate_var_name(&var.name) {
            continue;
        }
        let value = escape_value(&var.value);
        lines.push(format!("{}={}", var.name, value));
    }
    if !lines.is_empty() {
        lines.push(String::new());
    }
    lines.join("\n")
}

fn escape_value(value: &str) -> String {
    if value.is_empty() {
        return "\"\"".to_string();
    }
    if value.chars().any(|c| c.is_whitespace() || matches!(c, '#' | '"' | '\'' | '\\')) {
        let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
        format!("\"{escaped}\"")
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_names() {
        assert!(validate_var_name("PATH"));
        assert!(validate_var_name("_FOO"));
        assert!(validate_var_name("My_Var2"));
        assert!(!validate_var_name(""));
        assert!(!validate_var_name("1ABC"));
        assert!(!validate_var_name("A-B"));
        assert!(!validate_var_name("A B"));
    }

    #[test]
    fn parse_and_serialize_roundtrip() {
        let text = "# c\nFOO=bar\nBAZ=\"hello world\"\n\n";
        let vars = parse_conf(text);
        assert_eq!(vars.len(), 2);
        assert_eq!(vars[0].name, "FOO");
        assert_eq!(vars[0].value, "bar");
        assert_eq!(vars[1].value, "hello world");
        let again = parse_conf(&serialize_conf(&vars));
        assert_eq!(again, vars);
    }

    #[test]
    fn skips_invalid_and_duplicates() {
        let vars = parse_conf("A=1\nA=2\n9X=y\nOK=z\n");
        assert_eq!(vars.len(), 2);
        assert_eq!(vars[0].name, "A");
        assert_eq!(vars[0].value, "1");
        assert_eq!(vars[1].name, "OK");
    }
}
