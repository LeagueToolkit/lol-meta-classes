//! Step outputs for the workflow that runs `meta-sync`.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

/// Appends `outputs` to the file at `path` as `key=value` lines, the format of
/// the `$GITHUB_OUTPUT` file. Creates the file if it does not exist.
///
/// Fails with [`std::io::ErrorKind::InvalidInput`] if a value contains a line
/// break, because the line format cannot hold it.
pub fn append_outputs(path: &Path, outputs: &[(&str, String)]) -> std::io::Result<()> {
    let mut text = String::new();
    for (key, value) in outputs {
        if value.contains(['\n', '\r']) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("output `{}` contains a line break", key),
            ));
        }
        text.push_str(&format!("{}={}\n", key, value));
    }
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?
        .write_all(text.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_outputs_appends_key_value_lines() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("output");
        std::fs::write(&path, "earlier=1\n").unwrap();

        append_outputs(
            &path,
            &[("changed", "true".to_string()), ("version", String::new())],
        )
        .unwrap();

        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "earlier=1\nchanged=true\nversion=\n"
        );
    }

    #[test]
    fn append_outputs_fails_if_value_contains_line_break() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("output");

        let error = append_outputs(&path, &[("removed", "a\nb".to_string())]).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
        assert!(!path.exists());
    }
}
