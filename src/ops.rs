use anyhow::{bail, Context, Result};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Default storage location, used when `--dir` is not given.
pub fn default_store_dir() -> PathBuf {
    let home = std::env::home_dir().expect("could not determine home directory");
    home.join("Library/Application Support/Google/private extensions")
}

fn ensure_store_dir(dir: &Path) -> Result<()> {
    fs::create_dir_all(dir)
        .with_context(|| format!("failed to create store directory: {}", dir.display()))
}

/// Clone `url` into the store directory.
///
/// - If `name` is given, clone explicitly into `<store_dir>/<name>`.
/// - If `name` is omitted, this behaves exactly like running a plain
///   `git clone <url>` from inside `store_dir`: git itself picks the
///   destination folder name, same as normal `git clone` usage.
///
/// Either way, an extension that is already saved is skipped rather than
/// treated as an error, so re-running `add` or `import` is safe.
///
/// `git` inherits our stdio, so its own progress output is the report for
/// this command; we only print for the case where git never runs.
pub fn add(store_dir: &Path, url: &str, name: Option<&str>) -> Result<()> {
    ensure_store_dir(store_dir)?;

    let dest = name.map(|name| store_dir.join(name));

    // When git picks the name itself we still need to know what it would pick,
    // otherwise a second run would fail with "destination path already exists".
    let existing = match &dest {
        Some(dest) => dest.exists().then(|| dest.clone()),
        None => dir_name_from_url(url)
            .map(|name| store_dir.join(name))
            .filter(|dest| dest.exists()),
    };
    if let Some(existing) = existing {
        println!("skip: already exists at {}", existing.display());
        return Ok(());
    }

    run_clone(url, dest.as_deref(), store_dir)
}

/// The folder name `git clone <url>` picks when given no destination:
/// the last path component, without a trailing `.git`.
fn dir_name_from_url(url: &str) -> Option<String> {
    let last = url.trim_end_matches('/').rsplit(['/', ':']).next()?;
    let name = last.strip_suffix(".git").unwrap_or(last);
    (!name.is_empty()).then(|| name.to_string())
}

/// Run `git clone <url> [dest]` with `cwd` as the working directory.
fn run_clone(url: &str, dest: Option<&Path>, cwd: &Path) -> Result<()> {
    let mut cmd = Command::new("git");
    cmd.current_dir(cwd).arg("clone").arg(url);
    if let Some(dest) = dest {
        cmd.arg(dest);
    }
    let status = cmd
        .status()
        .context("failed to run `git`. Is git installed and on PATH?")?;
    if !status.success() {
        bail!("git clone failed for {url}");
    }
    Ok(())
}

/// Read the `origin` remote URL of a git repo directory, if any.
pub fn remote_url(repo_dir: &Path) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo_dir)
        .args(["remote", "get-url", "origin"])
        .output()
        .ok()?;

    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// Print (or write to file) the saved extensions as a plain URL list.
pub fn list(store_dir: &Path, output: Option<&Path>) -> Result<()> {
    ensure_store_dir(store_dir)?;

    let mut entries: Vec<PathBuf> = fs::read_dir(store_dir)
        .with_context(|| format!("failed to read {}", store_dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    entries.sort();

    let mut lines = Vec::new();
    for dir in &entries {
        let name = dir.file_name().unwrap().to_string_lossy().to_string();
        match remote_url(dir) {
            Some(url) => lines.push(url),
            None => eprintln!("warning: '{name}' has no git remote 'origin'; skipping"),
        }
    }

    let text = lines.join("\n");
    match output {
        Some(path) => {
            fs::write(path, format!("{text}\n"))
                .with_context(|| format!("failed to write {}", path.display()))?;
            println!("wrote {} URL(s) to {}", lines.len(), path.display());
        }
        None => {
            if lines.is_empty() {
                println!("(no saved extensions)");
            } else {
                println!("{text}");
            }
        }
    }
    Ok(())
}

/// Read a URL list file and add every extension listed in it.
pub fn import(store_dir: &Path, file: &Path) -> Result<()> {
    let f = fs::File::open(file).with_context(|| format!("failed to open {}", file.display()))?;
    let reader = io::BufReader::new(f);

    let mut ok_count = 0;
    let mut err_count = 0;
    for line in reader.lines() {
        let line = line?;
        let url = line.trim();
        if url.is_empty() || url.starts_with('#') {
            continue;
        }
        match add(store_dir, url, None) {
            Ok(()) => ok_count += 1,
            Err(e) => {
                eprintln!("error: failed to add {url}: {e}");
                err_count += 1;
            }
        }
    }

    println!("import finished: {ok_count} succeeded, {err_count} failed");
    if err_count > 0 {
        bail!("{err_count} extension(s) failed to import");
    }
    Ok(())
}

/// Remove a saved extension by folder name.
pub fn remove(store_dir: &Path, name: &str, yes: bool) -> Result<()> {
    let dest = store_dir.join(name);
    if !dest.exists() {
        bail!(
            "no saved extension named '{name}' in {}",
            store_dir.display()
        );
    }

    if !yes {
        print!("remove '{name}' at {}? [y/N] ", dest.display());
        io::stdout().flush().ok();
        let mut resp = String::new();
        io::stdin().read_line(&mut resp)?;
        if !resp.trim().eq_ignore_ascii_case("y") {
            println!("cancelled");
            return Ok(());
        }
    }

    fs::remove_dir_all(&dest).with_context(|| format!("failed to remove {}", dest.display()))?;
    println!("removed: {name}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::dir_name_from_url;

    #[test]
    fn derives_the_same_folder_name_git_would() {
        let cases = [
            ("https://github.com/user/my-ext.git", "my-ext"),
            ("https://github.com/user/my-ext", "my-ext"),
            ("https://github.com/user/my-ext/", "my-ext"),
            ("git@github.com:user/my-ext.git", "my-ext"),
            ("ssh://git@example.com:2222/user/my-ext.git", "my-ext"),
            ("/local/path/my-ext", "my-ext"),
        ];
        for (url, expected) in cases {
            assert_eq!(dir_name_from_url(url).as_deref(), Some(expected), "{url}");
        }
    }
}
