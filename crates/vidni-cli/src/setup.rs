//! `vidni setup` (TODO.md, P8-VIDNI-03): for the portable bundle, on a
//! Linux with no Nix. It fetches the tools vidni drives, as one archive
//! pinned by its SHA-256, unpacks them into the user's cache, and moves
//! them there; then the draft voice and the listening model, each by its
//! SHA-256 too. It needs no root, and writes nowhere but the cache and
//! the voice store.
//!
//! The tools are the flake's own, built by Nix, so every path inside them
//! starts with the store they were built in. Moving them means putting
//! the folder they are unpacked into in that prefix's place: in each
//! program's loader and library search path (with `patchelf`, which the
//! bundle carries), in each text file, and in each symbolic link.

use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use vidni_core::voices;

/// What the bundle says `vidni setup` fetches: `share/vidni/setup.json`.
#[derive(Debug, serde::Deserialize)]
struct Pins {
    tools: Tools,
    /// The draft voice to fetch, by its ID.
    voice: String,
    listening_model: Pinned,
}

#[derive(Debug, serde::Deserialize)]
struct Tools {
    file: String,
    url: String,
    sha256: String,
    bytes: u64,
    /// The prefix every path in the archive's files starts with.
    store: String,
    /// The file, inside the archive, that the bundle's `vidni` reads its
    /// environment from.
    env: String,
}

#[derive(Debug, serde::Deserialize)]
struct Pinned {
    file: String,
    url: String,
    sha256: String,
}

/// Where vidni keeps what it sets up: `$XDG_CACHE_HOME/vidni`, or
/// `~/.cache/vidni`.
fn cache() -> Result<PathBuf, String> {
    let var = |name: &str| std::env::var_os(name).filter(|v| !v.is_empty());
    var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| var("HOME").map(|home| PathBuf::from(home).join(".cache")))
        .map(|cache| cache.join("vidni"))
        .ok_or_else(|| "there is no cache to set up in: set HOME or XDG_CACHE_HOME".to_string())
}

/// Refuses a folder whose path can't be written into a script as it is.
/// The tools' own scripts name each other by their paths, on `#!` lines
/// and in plain shell, so a space or a quote in the path breaks them.
fn plain(path: &Path) -> Result<(), String> {
    let text = path.to_str().unwrap_or_default();
    let fine = |c: char| c.is_alphanumeric() || "/._-+@%,:=".contains(c);
    match !text.is_empty() && text.chars().all(fine) {
        true => Ok(()),
        false => Err(format!(
            "the tools can't be set up in {}: its path has a space, a quote or another \
             character that the tools' scripts can't hold. Set XDG_CACHE_HOME to a folder with \
             a plain path, for `vidni setup` and for vidni afterwards",
            path.display()
        )),
    }
}

/// The SHA-256 of the file at `path`, read a part at a time.
fn sha256(path: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let shown = |e: std::io::Error| format!("{}: {e}", path.display());
    let mut file = fs::File::open(path).map_err(shown)?;
    let mut hash = Sha256::new();
    let mut part = vec![0u8; 1 << 20];
    loop {
        match file.read(&mut part).map_err(shown)? {
            0 => break,
            n => hash.update(&part[..n]),
        }
    }
    Ok(hash.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

/// Downloads `url` to `to` with curl, showing how far it is when someone
/// is watching.
fn download(url: &str, to: &Path) -> Result<(), String> {
    use std::io::IsTerminal;
    let mut curl = Command::new("curl");
    curl.args(["--fail", "--location", "--retry", "3", "--output"])
        .arg(to)
        .arg(url)
        .stdin(Stdio::null());
    match std::io::stderr().is_terminal() {
        true => curl.arg("--progress-bar"),
        false => curl.args(["--silent", "--show-error"]),
    };
    let done = curl.status().map_err(|e| format!("curl: {e}"))?;
    match done.success() {
        true => Ok(()),
        false => {
            let _ = fs::remove_file(to);
            Err(format!("{url} couldn't be downloaded"))
        }
    }
}

/// Every file, folder and link under `root`, links not followed.
fn walk(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut found = Vec::new();
    let mut folders = vec![root.to_path_buf()];
    while let Some(folder) = folders.pop() {
        let entries = fs::read_dir(&folder).map_err(|e| format!("{}: {e}", folder.display()))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("{}: {e}", folder.display()))?;
            let kind = entry
                .file_type()
                .map_err(|e| format!("{}: {e}", entry.path().display()))?;
            match kind.is_dir() {
                true => folders.push(entry.path()),
                false => found.push(entry.path()),
            }
        }
    }
    Ok(found)
}

/// `bytes` with every `old` replaced by `new`; none when `old` isn't in it.
fn replaced(bytes: &[u8], old: &[u8], new: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(bytes.len());
    let (mut at, mut any) = (0, false);
    while at < bytes.len() {
        if bytes[at..].starts_with(old) {
            out.extend_from_slice(new);
            at += old.len();
            any = true;
        } else {
            out.push(bytes[at]);
            at += 1;
        }
    }
    any.then_some(out)
}

/// What `patchelf <flag> <file>` prints; none when it fails, as it does
/// on a file that has no such thing.
fn patchelf_says(patchelf: &Path, flag: &str, file: &Path) -> Option<String> {
    let out = Command::new(patchelf)
        .arg(flag)
        .arg(file)
        .stdin(Stdio::null())
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn patchelf_sets(patchelf: &Path, flag: &str, value: &str, file: &Path) -> Result<(), String> {
    let out = Command::new(patchelf)
        .arg(flag)
        .arg(value)
        .arg(file)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("{}: {e}", patchelf.display()))?;
    match out.status.success() {
        true => Ok(()),
        false => Err(format!(
            "{}: {}",
            file.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        )),
    }
}

/// The largest text file that is read to look for the prefix in it.
const TEXT: u64 = 8 << 20;

/// Moves the one file, link or program at `path` from the prefix `old`
/// to `new`.
fn relocate_one(
    path: &Path,
    old: &str,
    new: &str,
    patchelf: Option<&Path>,
    libc: &[String],
) -> Result<(), String> {
    let shown = |e: std::io::Error| format!("{}: {e}", path.display());
    let meta = fs::symlink_metadata(path).map_err(shown)?;
    if meta.file_type().is_symlink() {
        let target = fs::read_link(path).map_err(shown)?;
        if let Some(rest) = target.to_str().and_then(|t| t.strip_prefix(old)) {
            fs::remove_file(path).map_err(shown)?;
            std::os::unix::fs::symlink(format!("{new}{rest}"), path).map_err(shown)?;
        }
        return Ok(());
    }
    if !meta.is_file() {
        return Ok(());
    }
    let mut magic = [0u8; 4];
    let elf = fs::File::open(path)
        .and_then(|mut file| file.read_exact(&mut magic))
        .is_ok_and(|()| magic == *b"\x7fELF");
    if elf {
        let Some(patchelf) = patchelf else {
            return Ok(());
        };
        // A library's loader is never used: only a program's is set.
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if !name.contains(".so")
            && let Some(loader) = patchelf_says(patchelf, "--print-interpreter", path)
            && let Some(rest) = loader.strip_prefix(old)
        {
            patchelf_sets(patchelf, "--set-interpreter", &format!("{new}{rest}"), path)?;
        }
        // A loader looks for the C library's own parts where it was built
        // to be, which is nowhere now: every program and library is told
        // where they are. The C library itself is left as it is.
        let own = format!("{}/", path.display());
        let in_libc = |dir: &String| {
            let name = dir.rsplit('/').nth(1).unwrap_or_default();
            own.contains(&format!("/{name}/"))
        };
        if libc.iter().any(in_libc) {
            return Ok(());
        }
        if let Some(search) = patchelf_says(patchelf, "--print-rpath", path) {
            let mut dirs: Vec<String> = search
                .replace(old, new)
                .split(':')
                .filter(|dir| !dir.is_empty())
                .map(str::to_string)
                .collect();
            for dir in libc {
                if !dirs.contains(dir) {
                    dirs.push(dir.clone());
                }
            }
            let moved = dirs.join(":");
            if moved != search {
                patchelf_sets(patchelf, "--set-rpath", &moved, path)?;
            }
        }
        return Ok(());
    }
    if meta.len() > TEXT {
        return Ok(());
    }
    let bytes = fs::read(path).map_err(shown)?;
    // A file with a zero byte in it isn't text, and is left as it is.
    if bytes.contains(&0) {
        return Ok(());
    }
    if let Some(moved) = replaced(&bytes, old.as_bytes(), new.as_bytes()) {
        fs::write(path, moved).map_err(shown)?;
    }
    Ok(())
}

/// Moves everything under `root` from the prefix `old` to `new`: links,
/// text files and, with `patchelf`, programs and libraries. Returns how
/// many files it looked at.
pub fn relocate(
    root: &Path,
    old: &str,
    new: &str,
    patchelf: Option<&Path>,
) -> Result<usize, String> {
    let files = walk(root)?;
    // Where the C library's parts will be: beside each loader, under the
    // new prefix.
    let libc: Vec<String> = files
        .iter()
        .filter(|path| {
            path.file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with("ld-linux"))
        })
        .filter_map(|path| path.parent()?.strip_prefix(root.join("store")).ok())
        .map(|dir| format!("{new}/{}", dir.display()))
        .collect();
    let workers = std::thread::available_parallelism().map_or(4, |n| n.get());
    let next = std::sync::atomic::AtomicUsize::new(0);
    let failed = std::sync::Mutex::new(None);
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                loop {
                    let at = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some(path) = files.get(at) else { break };
                    if let Err(why) = relocate_one(path, old, new, patchelf, &libc) {
                        failed.lock().expect("no worker panics").get_or_insert(why);
                        break;
                    }
                }
            });
        }
    });
    match failed.into_inner().expect("no worker panics") {
        Some(why) => Err(why),
        None => Ok(files.len()),
    }
}

/// Unpacks the tools' archive into `to`.
fn unpack(archive: &Path, to: &Path) -> Result<(), String> {
    let shown = |e: &dyn std::fmt::Display| format!("{}: {e}", archive.display());
    let file = fs::File::open(archive).map_err(|e| shown(&e))?;
    let packed = std::io::BufReader::with_capacity(1 << 20, file);
    let plain = ruzstd::decoding::StreamingDecoder::new(packed).map_err(|e| shown(&e))?;
    let mut tar = tar::Archive::new(plain);
    // Of a file's mode, only whether it is a program is kept: nothing
    // unpacked is set-user-ID, and everything can be written to be moved.
    tar.set_preserve_permissions(false);
    tar.set_preserve_mtime(true);
    tar.unpack(to).map_err(|e| shown(&e))
}

/// Sets the tools up from the archive at `archive`, which has been
/// checked, under `tools`; returns their folder.
fn install(pins: &Tools, archive: &Path, tools: &Path, bundle: &Path) -> Result<PathBuf, String> {
    let id = &pins.sha256[..pins.sha256.len().min(16)];
    // Each run unpacks into a folder of its own, so that two at once
    // don't write into each other's; the first to finish is the one kept.
    let home = tools.join(id);
    let part = tools.join(format!("{id}.part.{}", std::process::id()));
    // What a run that was stopped left behind.
    for entry in fs::read_dir(tools).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let stopped = name
            .strip_prefix(&format!("{id}.part."))
            .is_some_and(|pid| !Path::new("/proc").join(pid).exists());
        if stopped {
            let _ = fs::remove_dir_all(entry.path());
        }
    }
    let made = |path: &Path| {
        let path = path.to_path_buf();
        move |e: std::io::Error| format!("{}: {e}", path.display())
    };
    if part.exists() {
        fs::remove_dir_all(&part).map_err(made(&part))?;
    }
    fs::create_dir_all(&part).map_err(made(&part))?;
    say!("unpacking the tools into {}", home.display());
    unpack(archive, &part)?;
    say!("moving them there: every program's loader and paths");
    // The files are moved to where the folder will be once it is whole.
    let new = home.join("store").display().to_string();
    let patchelf = bundle.join("bin/patchelf");
    let files = relocate(&part, &pins.store, &new, Some(&patchelf))?;
    // What the bundle's `vidni` reads its environment from.
    let env = part.join(&pins.env);
    fs::copy(&env, part.join("env.sh")).map_err(made(&env))?;
    match home.join("env.sh").is_file() {
        // Another run set the same tools up meanwhile: they are whole.
        true => fs::remove_dir_all(&part).map_err(made(&part))?,
        false => {
            if home.exists() {
                fs::remove_dir_all(&home).map_err(made(&home))?;
            }
            fs::rename(&part, &home).map_err(made(&home))?;
        }
    }
    say!("the tools are set up: {files} files");
    Ok(home)
}

/// Fetches the tools, the draft voice and the listening model. `from` is
/// a folder that holds the tools' archive already.
pub fn setup(from: Option<&Path>, mirror: Option<&str>) -> ExitCode {
    match run(from, mirror) {
        Ok(()) => ExitCode::SUCCESS,
        Err(why) => {
            tell!("vidni: {why}");
            ExitCode::FAILURE
        }
    }
}

fn run(from: Option<&Path>, mirror: Option<&str>) -> Result<(), String> {
    let bundle = std::env::var_os("VIDNI_BUNDLE")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .ok_or(
            "`vidni setup` is for vidni's portable bundle, which this isn't. With Nix, the \
             flake has the tools already: `nix run github:dhilipsiva/vidni`",
        )?;
    let file = bundle.join("share/vidni/setup.json");
    let pins: Pins = fs::read_to_string(&file)
        .map_err(|e| e.to_string())
        .and_then(|text| serde_json::from_str(&text).map_err(|e| e.to_string()))
        .map_err(|e| format!("{}: {e}", file.display()))?;
    let cache = cache()?;
    plain(&cache)?;
    let tools = cache.join("tools");
    let made = |path: &Path| {
        let path = path.to_path_buf();
        move |e: std::io::Error| format!("{}: {e}", path.display())
    };
    fs::create_dir_all(&tools).map_err(made(&tools))?;

    // The tools: set up already, or fetched, checked and unpacked.
    let id = &pins.tools.sha256[..pins.tools.sha256.len().min(16)];
    let home = tools.join(id);
    if home.join("env.sh").is_file() {
        say!("the tools are set up already, in {}", home.display());
    } else {
        let (archive, fetched) = match from {
            Some(folder) => (folder.join(&pins.tools.file), false),
            None => {
                // The release's own address, or the same file at a mirror.
                let url = match mirror {
                    Some(mirror) => format!("{}/{}", mirror.trim_end_matches('/'), pins.tools.file),
                    None => pins.tools.url.clone(),
                };
                let to = cache.join("downloads");
                fs::create_dir_all(&to).map_err(made(&to))?;
                let archive = to.join(&pins.tools.file);
                // One that was fetched before and is whole is used again.
                if !sha256(&archive).is_ok_and(|sha| sha == pins.tools.sha256) {
                    say!(
                        "fetching the tools: {:.0} MB from {url}",
                        pins.tools.bytes as f64 / 1e6
                    );
                    download(&url, &archive)?;
                }
                (archive, true)
            }
        };
        say!("checking {}", archive.display());
        let sha = sha256(&archive)?;
        if sha != pins.tools.sha256 {
            if fetched {
                let _ = fs::remove_file(&archive);
            }
            return Err(format!(
                "{} isn't the archive this vidni pins: its SHA-256 is {sha}, not {}",
                archive.display(),
                pins.tools.sha256
            ));
        }
        let home = install(&pins.tools, &archive, &tools, &bundle)?;
        if fetched {
            // It is unpacked; the download isn't needed again.
            let _ = fs::remove_file(&archive);
        }
        debug_assert!(home.join("env.sh").is_file());
    }

    // The listening model, where the bundle's `vidni` is told to find it.
    let models = cache.join("models");
    fs::create_dir_all(&models).map_err(made(&models))?;
    let model = models.join(&pins.listening_model.file);
    if !sha256(&model).is_ok_and(|sha| sha == pins.listening_model.sha256) {
        say!(
            "fetching the listening model from {}",
            pins.listening_model.url
        );
        let part = models.join(format!("{}.part", pins.listening_model.file));
        download(&pins.listening_model.url, &part)?;
        let sha = sha256(&part)?;
        if sha != pins.listening_model.sha256 {
            let _ = fs::remove_file(&part);
            return Err(format!(
                "{} isn't the model this vidni pins: its SHA-256 is {sha}",
                pins.listening_model.url
            ));
        }
        fs::rename(&part, &model).map_err(made(&model))?;
    }
    let env = home.join("env.sh");
    let told = format!("VIDNI_WHISPER_MODEL=\"{}\"\n", model.display());
    let text = fs::read_to_string(&env).map_err(made(&env))?;
    if !text.contains(&told) {
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&env)
            .map_err(made(&env))?;
        // The file came out of the archive, which nobody may write to.
        let _ = fs::set_permissions(&env, fs::Permissions::from_mode(0o644));
        file.write_all(format!("{told}export VIDNI_WHISPER_MODEL\n").as_bytes())
            .map_err(made(&env))?;
    }
    say!("the listening model is in {}", model.display());

    // The draft voice, into the voice store, as `vidni voices fetch` does.
    let store = voices::store().ok_or("no voice store; set VIDNI_VOICES or HOME")?;
    let pack = voices::all()
        .into_iter()
        .find(|pack| pack.id == pins.voice)
        .ok_or_else(|| format!("{} isn't a voice vidni can fetch", pins.voice))?;
    let dir = vidni_media::tts::fetch(&pack, &store)?;
    say!("the draft voice {} is in {}", pins.voice, dir.display());
    say!("vidni is set up; `vidni new my-video` and `vidni draft my-video` make a first video");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_and_links_are_moved_and_what_isnt_text_is_left() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let (old, new) = ("/nix/store", "/home/someone/.cache/vidni/tools/abc/store");
        fs::create_dir_all(root.join("store/a-tool/bin")).unwrap();
        let script = root.join("store/a-tool/bin/tool");
        fs::write(
            &script,
            "#!/nix/store/b-bash/bin/bash\nexec /nix/store/a-tool/libexec/tool \"$@\"\n",
        )
        .unwrap();
        fs::set_permissions(&script, fs::Permissions::from_mode(0o555)).unwrap();
        let data = root.join("store/a-tool/data.bin");
        fs::write(&data, b"\x00/nix/store/a-tool\x00").unwrap();
        let plain = root.join("store/a-tool/notes.txt");
        fs::write(&plain, "nothing to move here\n").unwrap();
        let link = root.join("store/a-tool/lib");
        std::os::unix::fs::symlink("/nix/store/c-libs/lib", &link).unwrap();
        let near = root.join("store/a-tool/near");
        std::os::unix::fs::symlink("bin/tool", &near).unwrap();

        // A read-only script can't be written over: say so, and stop.
        assert!(relocate(root, old, new, None).is_err());
        fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(relocate(root, old, new, None).unwrap(), 5);
        assert_eq!(
            fs::read_to_string(&script).unwrap(),
            format!("#!{new}/b-bash/bin/bash\nexec {new}/a-tool/libexec/tool \"$@\"\n")
        );
        // It is still a program.
        assert_eq!(
            fs::metadata(&script).unwrap().permissions().mode() & 0o777,
            0o755
        );
        assert_eq!(fs::read(&data).unwrap(), b"\x00/nix/store/a-tool\x00");
        assert_eq!(
            fs::read_to_string(&plain).unwrap(),
            "nothing to move here\n"
        );
        assert_eq!(
            fs::read_link(&link).unwrap(),
            PathBuf::from(format!("{new}/c-libs/lib"))
        );
        assert_eq!(fs::read_link(&near).unwrap(), PathBuf::from("bin/tool"));
        // Done twice, nothing more changes.
        relocate(root, old, new, None).unwrap();
        assert_eq!(
            fs::read_link(&link).unwrap(),
            PathBuf::from(format!("{new}/c-libs/lib"))
        );
    }

    #[test]
    fn a_cache_whose_path_a_script_cant_hold_is_refused() {
        for fine in ["/home/someone/.cache/vidni", "/mnt/data+2/cache@home/vidni"] {
            assert_eq!(plain(Path::new(fine)), Ok(()), "{fine}");
        }
        for odd in [
            "/home/some one/.cache/vidni",
            "/home/o'neil/.cache/vidni",
            "/home/a\"b/vidni",
            "/home/$USER/vidni",
            "/home/a`b`/vidni",
            "",
        ] {
            let why = plain(Path::new(odd)).unwrap_err();
            assert!(why.contains("Set XDG_CACHE_HOME"), "{odd}: {why}");
        }
    }

    #[test]
    fn setup_is_refused_outside_the_bundle() {
        // No test sets VIDNI_BUNDLE, and the dev shell doesn't either.
        if std::env::var_os("VIDNI_BUNDLE").is_none() {
            let why = run(None, None).unwrap_err();
            assert!(
                why.starts_with("`vidni setup` is for vidni's portable bundle"),
                "{why}"
            );
        }
    }
}
