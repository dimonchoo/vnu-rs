use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn document_url(path: &str, cwd: &Path) -> String {
    if path == "-" {
        return "stdin".to_string();
    }
    if path.starts_with("http://") || path.starts_with("https://") || path.starts_with("file:") {
        return path.to_string();
    }
    let path_buf = Path::new(path);
    let abs = if path_buf.is_absolute() {
        path_buf.to_path_buf()
    } else {
        cwd.join(path_buf)
    };
    let mut normalized = abs.to_string_lossy().replace('\\', "/");
    if !normalized.starts_with('/') {
        normalized.insert(0, '/');
    }
    format!("file:{normalized}")
}

pub fn expand_inputs(paths: &[String]) -> Result<Vec<String>, String> {
    let mut files = Vec::new();
    let mut seen_dirs = HashSet::new();
    for path in paths {
        if path == "-" || path.starts_with("http://") || path.starts_with("https://") {
            files.push(path.clone());
            continue;
        }
        match fs::metadata(path) {
            Ok(meta) if meta.is_dir() => collect_dir(Path::new(path), &mut files, &mut seen_dirs)?,
            _ => files.push(path.clone()),
        }
    }
    Ok(files)
}

fn collect_dir(
    dir: &Path,
    files: &mut Vec<String>,
    seen_dirs: &mut HashSet<PathBuf>,
) -> Result<(), String> {
    let key = fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
    if !seen_dirs.insert(key) {
        return Ok(());
    }
    let mut entries = fs::read_dir(dir)
        .map_err(|err| format!("Error reading directory {}: {}", dir.display(), err))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("Error reading directory {}: {}", dir.display(), err))?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|err| format!("Error reading {}: {}", path.display(), err))?;
        if file_type.is_dir() {
            collect_dir(&path, files, seen_dirs)?;
        } else if file_type.is_symlink() && fs::metadata(&path).is_ok_and(|meta| meta.is_dir()) {
            // Directory symlinks are not followed; walking them can cycle.
        } else {
            files.push(path.to_string_lossy().into_owned());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_url_uses_file_scheme_for_absolute_paths() {
        let cwd = Path::new("/tmp");
        assert_eq!(document_url("/tmp/a.html", cwd), "file:/tmp/a.html");
        assert_eq!(document_url("-", cwd), "stdin");
        assert_eq!(
            document_url("https://example.com/a", cwd),
            "https://example.com/a"
        );
        assert_eq!(document_url("a.html", cwd), "file:/tmp/a.html");
    }

    #[test]
    fn expand_inputs_walks_directories_in_sorted_order() {
        let root = std::env::temp_dir().join(format!(
            "vnu-rs-expand-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(root.join("nested")).expect("mkdir");
        fs::write(root.join("b.html"), "<!DOCTYPE html>").expect("write");
        fs::write(root.join("nested").join("a.html"), "<!DOCTYPE html>").expect("write");

        let files = expand_inputs(&[root.to_string_lossy().into_owned()]).expect("expand");
        let names: Vec<String> = files
            .iter()
            .map(|path| {
                Path::new(path)
                    .strip_prefix(&root)
                    .expect("prefix")
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect();
        assert_eq!(
            names,
            vec!["b.html".to_string(), "nested/a.html".to_string()]
        );
        let _ = fs::remove_dir_all(&root);
    }
}
