//! `vidni upload` (TODO.md, P9-VIDNI-01): an upload to YouTube through its
//! Data API, for someone who asks for it. vidni uploads nothing unless
//! this command is run, and it asks before each upload.
//!
//! - What is uploaded is a release: the cut's newest approved build, as
//!   `vidni final` delivered it for YouTube, with the title, description
//!   and tags that its `metadata.md` has.
//! - Before anything is sent, the person types the release's name.
//! - The credentials are the person's own OAuth client and a refresh
//!   token, in a file outside the project that only they can read.
//! - It goes up private unless another visibility is asked for.
//! - An assistant can't run it (docs/assistant.md).
//!
//! It talks to the API with curl, and has been run against a made-up API
//! only: no live upload has been made with it (DECISIONS.md, R9).

use crate::{Loaded, load};
use std::fs;
use std::io::{BufRead, Write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use vidni_core::release::{self, Voice};
use vidni_core::{platforms, published};

/// The file that names another API to talk to, for tests.
const API: &str = "VIDNI_YOUTUBE_API";
/// The file that names another credentials file.
const FILE: &str = "VIDNI_YOUTUBE";
const SCOPE: &str = "https://www.googleapis.com/auth/youtube.upload";

/// The person's OAuth client, and the token that lets vidni upload for
/// them.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Credentials {
    client_id: String,
    client_secret: String,
    /// What `vidni upload --login` stores.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    refresh_token: Option<String>,
}

/// Where the credentials are: `$VIDNI_YOUTUBE`, or `youtube.json` in
/// vidni's folder of settings. Never in a project.
fn credentials_file() -> Result<PathBuf, String> {
    let var = |name: &str| std::env::var_os(name).filter(|v| !v.is_empty());
    if let Some(file) = var(FILE) {
        return Ok(PathBuf::from(file));
    }
    var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| var("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map(|config| config.join("vidni/youtube.json"))
        .ok_or_else(|| "there is nowhere to keep credentials: set HOME or VIDNI_YOUTUBE".into())
}

fn read_credentials(dir: &Path) -> Result<(PathBuf, Credentials), String> {
    let file = credentials_file()?;
    // A project is shared and committed; a secret isn't.
    let inside = |file: &Path| {
        let (Ok(file), Ok(dir)) = (file.canonicalize(), dir.canonicalize()) else {
            return false;
        };
        file.starts_with(dir)
    };
    if inside(&file) {
        return Err(format!(
            "{} is inside the project: credentials are kept outside it",
            file.display()
        ));
    }
    let meta = fs::metadata(&file).map_err(|_| {
        format!(
            "there are no YouTube credentials in {}; docs/upload.md says how to make them",
            file.display()
        )
    })?;
    // Where files have such modes, the secret is its owner's alone.
    #[cfg(unix)]
    if meta.permissions().mode() & 0o077 != 0 {
        return Err(format!(
            "{} can be read by others on this machine, and it holds a secret: `chmod 600` it",
            file.display()
        ));
    }
    let _ = &meta;
    let text = fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
    let credentials =
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", file.display()))?;
    Ok((file, credentials))
}

/// The addresses of the token and upload services.
struct Api {
    auth: String,
    token: String,
    upload: String,
    /// Where a video's page is, before its ID.
    watch: String,
}

impl Api {
    fn find() -> Self {
        match std::env::var(API).ok().filter(|v| !v.is_empty()) {
            Some(base) => {
                let base = base.trim_end_matches('/').to_string();
                Self {
                    auth: format!("{base}/auth"),
                    token: format!("{base}/token"),
                    upload: format!("{base}/upload/youtube/v3/videos"),
                    watch: format!("{base}/watch/"),
                }
            }
            None => Self {
                auth: "https://accounts.google.com/o/oauth2/v2/auth".into(),
                token: "https://oauth2.googleapis.com/token".into(),
                upload: "https://www.googleapis.com/upload/youtube/v3/videos".into(),
                watch: "https://youtu.be/".into(),
            },
        }
    }
}

/// What a request came to.
struct Reply {
    status: u16,
    headers: String,
    body: String,
}

impl Reply {
    fn json(&self) -> serde_json::Value {
        serde_json::from_str(&self.body).unwrap_or(serde_json::Value::Null)
    }

    /// What the service said was wrong, in its words.
    fn error(&self) -> String {
        let json = self.json();
        let said = json["error"]["message"]
            .as_str()
            .or(json["error_description"].as_str())
            .or(json["error"].as_str())
            .map(str::to_string);
        said.unwrap_or_else(|| self.body.trim().chars().take(200).collect())
    }
}

/// Runs curl with `config` on its standard input (so that no secret is
/// among its arguments, where other users can read it) and `args` after.
fn curl(config: &str, args: &[&std::ffi::OsStr]) -> Result<Reply, String> {
    let work = tempfile::tempdir().map_err(|e| e.to_string())?;
    let (headers, body) = (work.path().join("headers"), work.path().join("body"));
    let mut child = Command::new("curl")
        .args(["--silent", "--show-error", "--config", "-"])
        .arg("--dump-header")
        .arg(&headers)
        .arg("--output")
        .arg(&body)
        .args(["--write-out", "%{http_code}"])
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("curl: {e}"))?;
    child
        .stdin
        .take()
        .expect("a pipe")
        .write_all(config.as_bytes())
        .map_err(|e| format!("curl: {e}"))?;
    let out = child.wait_with_output().map_err(|e| format!("curl: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(Reply {
        status: String::from_utf8_lossy(&out.stdout)
            .trim()
            .parse()
            .unwrap_or(0),
        headers: fs::read_to_string(&headers).unwrap_or_default(),
        body: fs::read_to_string(&body).unwrap_or_default(),
    })
}

/// A value inside a curl config file's quotes.
fn quoted(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

/// `name=value`, as a form sends it.
fn form(pairs: &[(&str, &str)]) -> String {
    let encode = |value: &str| -> String {
        value
            .bytes()
            .map(|b| match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    (b as char).to_string()
                }
                other => format!("%{other:02X}"),
            })
            .collect()
    };
    pairs
        .iter()
        .map(|(name, value)| format!("{name}={}", encode(value)))
        .collect::<Vec<_>>()
        .join("&")
}

/// A value of an address's query, with its `%XX` escapes read.
fn decoded(value: &str) -> String {
    let bytes = value.as_bytes();
    let (mut out, mut at) = (Vec::with_capacity(bytes.len()), 0);
    while at < bytes.len() {
        let escaped = (bytes[at] == b'%')
            .then(|| value.get(at + 1..at + 3))
            .flatten()
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());
        match escaped {
            Some(byte) => {
                out.push(byte);
                at += 3;
            }
            None => {
                out.push(if bytes[at] == b'+' { b' ' } else { bytes[at] });
                at += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Asks the token service with `pairs`, and returns what it gave.
fn token(api: &Api, pairs: &[(&str, &str)]) -> Result<serde_json::Value, String> {
    let config = format!("data = \"{}\"\n", quoted(&form(pairs)));
    let reply = curl(
        &config,
        &["--request".as_ref(), "POST".as_ref(), api.token.as_ref()],
    )?;
    match reply.status {
        200 => Ok(reply.json()),
        status => Err(format!(
            "the sign-in was refused ({status}): {}. `vidni upload --login` signs in again",
            reply.error()
        )),
    }
}

/// Signs the person in: they open an address in their browser, allow
/// vidni to upload, and the browser comes back to this machine with a
/// code, which is exchanged for the token that is stored.
pub fn login() -> ExitCode {
    match sign_in() {
        Ok(file) => {
            say!("signed in; the token is in {}", file.display());
            ExitCode::SUCCESS
        }
        Err(why) => {
            tell!("vidni: {why}");
            ExitCode::FAILURE
        }
    }
}

fn sign_in() -> Result<PathBuf, String> {
    let (file, mut credentials) = read_credentials(Path::new("/nonexistent"))?;
    let api = Api::find();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let redirect = format!("http://127.0.0.1:{port}");
    say!("open this address in your browser, and allow vidni to upload videos:");
    say!(
        "  {}?{}",
        api.auth,
        form(&[
            ("client_id", &credentials.client_id),
            ("redirect_uri", &redirect),
            ("response_type", "code"),
            ("scope", SCOPE),
            ("access_type", "offline"),
            ("prompt", "consent"),
        ])
    );
    say!("waiting for the browser to come back to {redirect} …");
    // The browser's request: `GET /?code=…` or `GET /?error=…`.
    let (mut stream, _) = listener.accept().map_err(|e| e.to_string())?;
    let mut request = String::new();
    std::io::BufReader::new(&stream)
        .read_line(&mut request)
        .map_err(|e| e.to_string())?;
    let query = request
        .split_whitespace()
        .nth(1)
        .and_then(|path| path.split_once('?'))
        .map(|(_, query)| query.to_string())
        .unwrap_or_default();
    let value = |name: &str| {
        query
            .split('&')
            .filter_map(|pair| pair.split_once('='))
            .find(|(key, _)| *key == name)
            .map(|(_, value)| decoded(value))
    };
    let page = |words: &str| {
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nConnection: close\r\n\r\n{words}\n"
        )
    };
    let Some(code) = value("code") else {
        let _ = stream.write_all(page("vidni wasn't allowed. You can close this page.").as_bytes());
        return Err(format!(
            "the sign-in wasn't allowed: {}",
            value("error").unwrap_or_else(|| "the browser came back with no code".into())
        ));
    };
    let _ = stream.write_all(page("vidni is signed in. You can close this page.").as_bytes());
    drop(stream);
    let given = token(
        &api,
        &[
            ("client_id", &credentials.client_id),
            ("client_secret", &credentials.client_secret),
            ("code", &code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", &redirect),
        ],
    )?;
    let refresh = given["refresh_token"]
        .as_str()
        .ok_or("the sign-in gave no token to keep; allow it again with `vidni upload --login`")?;
    credentials.refresh_token = Some(refresh.to_string());
    let text = serde_json::to_string_pretty(&credentials).expect("JSON serializes") + "\n";
    fs::write(&file, text).map_err(|e| format!("{}: {e}", file.display()))?;
    Ok(file)
}

/// Asks for the release's name, and says whether it was typed.
fn confirmed(tag: &str, mut input: impl BufRead) -> bool {
    let mut typed = String::new();
    input.read_line(&mut typed).is_ok() && typed.trim() == tag
}

/// What is uploaded: the delivered file and the words that go with it.
struct Video {
    file: PathBuf,
    title: String,
    description: String,
    tags: Vec<String>,
}

/// Uploads `video` as `privacy`, and returns the video's ID.
fn send(
    api: &Api,
    credentials: &Credentials,
    video: &Video,
    privacy: &str,
) -> Result<String, String> {
    let refresh = credentials
        .refresh_token
        .as_deref()
        .ok_or("vidni isn't signed in to YouTube; `vidni upload --login` signs in")?;
    let given = token(
        api,
        &[
            ("client_id", &credentials.client_id),
            ("client_secret", &credentials.client_secret),
            ("refresh_token", refresh),
            ("grant_type", "refresh_token"),
        ],
    )?;
    let access = given["access_token"]
        .as_str()
        .ok_or("the sign-in gave no access token")?;
    let bearer = format!("header = \"Authorization: Bearer {}\"\n", quoted(access));
    let size = fs::metadata(&video.file)
        .map_err(|e| format!("{}: {e}", video.file.display()))?
        .len();
    let about = serde_json::json!({
        "snippet": {
            "title": video.title,
            "description": video.description,
            "tags": video.tags,
        },
        "status": { "privacyStatus": privacy, "selfDeclaredMadeForKids": false },
    });
    // First the video's words, which open a session; then its file.
    let config = format!(
        "{bearer}header = \"Content-Type: application/json; charset=UTF-8\"\n\
         header = \"X-Upload-Content-Type: video/mp4\"\n\
         header = \"X-Upload-Content-Length: {size}\"\n\
         data = \"{}\"\n",
        quoted(&about.to_string())
    );
    let address = format!("{}?uploadType=resumable&part=snippet,status", api.upload);
    let opened = curl(
        &config,
        &["--request".as_ref(), "POST".as_ref(), address.as_ref()],
    )?;
    if opened.status != 200 {
        return Err(format!(
            "YouTube refused the upload ({}): {}",
            opened.status,
            opened.error()
        ));
    }
    let session = opened
        .headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("location")
                .then(|| value.trim().to_string())
        })
        .ok_or("YouTube opened no session for the upload")?;
    let config = format!("{bearer}header = \"Content-Type: video/mp4\"\n");
    let sent = curl(
        &config,
        &[
            "--upload-file".as_ref(),
            video.file.as_os_str(),
            session.as_ref(),
        ],
    )?;
    if !(200..300).contains(&sent.status) {
        return Err(format!(
            "YouTube refused the file ({}): {}. Nothing is recorded as published",
            sent.status,
            sent.error()
        ));
    }
    sent.json()["id"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| "YouTube took the file and gave no video ID; look at your channel".into())
}

/// Uploads cut `cut`'s newest release to YouTube, after asking.
pub fn upload(path: &Path, cut: &str, platform: &str, privacy: &str) -> ExitCode {
    match run(path, cut, platform, privacy, std::io::stdin().lock()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(why) => {
            tell!("vidni: {why}");
            ExitCode::FAILURE
        }
    }
}

fn run(
    path: &Path,
    id: &str,
    platform: &str,
    privacy: &str,
    input: impl BufRead,
) -> Result<(), String> {
    let Loaded { dir, project, .. } = load(path).map_err(|_| "the project can't be read")?;
    let cut = project
        .cuts
        .iter()
        .find(|cut| cut.id == id)
        .ok_or_else(|| format!("there is no cut {id:?}"))?;
    if !matches!(platform, "youtube" | "youtube-shorts") {
        let known = platforms::get(platform).is_some();
        return Err(match known {
            true => format!("vidni uploads to YouTube only; {platform} is uploaded by hand"),
            false => format!("there is no platform {platform:?}; `vidni platforms` lists them"),
        });
    }
    if !cut.platforms().iter().any(|p| p == platform) {
        return Err(format!(
            "cut {id} isn't made for {platform}; `vidni exports --cut {id} --add {platform}` makes it so"
        ));
    }
    if !matches!(privacy, "private" | "unlisted" | "public") {
        return Err(format!(
            "{privacy:?} isn't a visibility: private, unlisted or public"
        ));
    }
    // What is uploaded is the newest release, as it was delivered.
    let releases = release::read(&dir)?;
    let newest = releases.latest(id).ok_or_else(|| {
        format!("cut {id} has no release; `vidni approve --cut {id}` makes one, and a release is what is uploaded")
    })?;
    let facts = dir
        .join(Voice::Takes.dir())
        .join(format!("{id}.upload.{platform}.json"));
    let words: serde_json::Value = fs::read_to_string(&facts)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .ok_or_else(|| {
            format!("cut {id} isn't exported for {platform}; `vidni final --cut {id}` exports it")
        })?;
    let name = words["video"].as_str().unwrap_or_default().to_string();
    let file = dir.join(crate::exports::folder(platform, id)).join(&name);
    let approved = newest
        .build
        .outputs
        .get(&name)
        .map(|output| output.mp4.clone());
    let delivered = release::file_hash(&file).ok();
    if delivered.is_none() || delivered != approved {
        return Err(format!(
            "{} isn't the file that was approved as {}; `vidni final --cut {id}` delivers it, and `vidni approve --cut {id}` approves what was built since",
            file.display(),
            newest.tag
        ));
    }
    let list = published::read(&dir)?;
    if let Some(there) = list
        .of(id)
        .find(|u| u.platform == platform && u.release == newest.tag)
    {
        return Err(format!(
            "{} is published on {platform} already, at {}",
            newest.tag, there.url
        ));
    }
    let video = Video {
        file,
        title: words["title"].as_str().unwrap_or_default().to_string(),
        description: words["description"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        tags: words["tags"]
            .as_array()
            .map(|tags| {
                tags.iter()
                    .filter_map(|tag| tag.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default(),
    };
    let (_, credentials) = read_credentials(&dir)?;

    // The person sees what goes up, and says so by the release's name.
    let size = fs::metadata(&video.file).map(|m| m.len()).unwrap_or(0);
    say!("to upload to YouTube, as {privacy}:");
    say!("  release  {}", newest.tag);
    say!(
        "  file     {} ({:.1} MB)",
        video.file.display(),
        size as f64 / 1e6
    );
    say!("  title    {}", video.title);
    say!("type the release's name to upload it, or anything else to stop:");
    if !confirmed(&newest.tag, input) {
        return Err("nothing was uploaded".into());
    }
    let api = Api::find();
    let video_id = send(&api, &credentials, &video, privacy)?;
    let url = format!("{}{video_id}", api.watch);
    let mut list = published::read(&dir)?;
    list.add(published::Upload {
        cut: id.to_string(),
        platform: platform.to_string(),
        release: newest.tag.clone(),
        url: url.clone(),
        on: release::today(),
    });
    published::write(&dir, &list)?;
    say!("{id}: {} is on YouTube, {privacy}, at {url}", newest.tag);
    say!(
        "  it is recorded in published.json; captions and the thumbnail are added by hand, from the upload's folder"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Read};
    use std::sync::mpsc;

    /// A made-up API on this machine: it answers each request with the
    /// next of `replies`, and hands each request over as it came.
    fn api(replies: Vec<String>) -> (String, mpsc::Receiver<String>) {
        api_at(|_| replies)
    }

    /// As [`api`], with replies that name the API's own address.
    fn api_at(replies: impl FnOnce(&str) -> Vec<String>) -> (String, mpsc::Receiver<String>) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let replies = replies(&base);
        let (seen, requests) = mpsc::channel();
        std::thread::spawn(move || {
            for reply in replies {
                let Ok((mut stream, _)) = listener.accept() else {
                    return;
                };
                let mut request = Vec::new();
                let mut block = [0u8; 65536];
                // The head, then as many bytes as it says the body has.
                let (mut head_end, mut length) = (None, 0usize);
                loop {
                    let n = stream.read(&mut block).unwrap_or(0);
                    if n == 0 {
                        break;
                    }
                    request.extend_from_slice(&block[..n]);
                    if head_end.is_none()
                        && let Some(at) = request.windows(4).position(|w| w == b"\r\n\r\n")
                    {
                        head_end = Some(at + 4);
                        let head = String::from_utf8_lossy(&request[..at]).to_lowercase();
                        length = head
                            .lines()
                            .find_map(|l| l.strip_prefix("content-length:"))
                            .and_then(|v| v.trim().parse().ok())
                            .unwrap_or(0);
                        if head.contains("expect: 100-continue") {
                            let _ = stream.write_all(b"HTTP/1.1 100 Continue\r\n\r\n");
                        }
                    }
                    if head_end.is_some_and(|end| request.len() >= end + length) {
                        break;
                    }
                }
                let _ = seen.send(String::from_utf8_lossy(&request).into_owned());
                let _ = stream.write_all(reply.as_bytes());
            }
        });
        (base, requests)
    }

    fn reply(status: &str, headers: &str, body: &str) -> String {
        format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n{headers}Connection: close\r\n\r\n{body}",
            body.len()
        )
    }

    fn at(base: &str) -> Api {
        Api {
            auth: format!("{base}/auth"),
            token: format!("{base}/token"),
            upload: format!("{base}/upload/youtube/v3/videos"),
            watch: format!("{base}/watch/"),
        }
    }

    fn video(dir: &Path) -> Video {
        let file = dir.join("long.mp4");
        fs::write(&file, b"not really a video, but 39 bytes of one").unwrap();
        Video {
            file,
            title: "A \"quoted\" title".into(),
            description: "Two lines\nof description, with a \\ in it.".into(),
            tags: vec!["one".into(), "two words".into()],
        }
    }

    fn signed_in() -> Credentials {
        Credentials {
            client_id: "the-client".into(),
            client_secret: "the secret & more".into(),
            refresh_token: Some("the/refresh+token".into()),
        }
    }

    #[test]
    fn a_video_goes_up_in_two_requests_after_a_token() {
        let dir = tempfile::tempdir().unwrap();
        let video = video(dir.path());
        // The session's address is on the made-up API too.
        let (base, requests) = api_at(|base| {
            vec![
                reply(
                    "200 OK",
                    "",
                    r#"{"access_token":"an-access-token","expires_in":3599}"#,
                ),
                reply(
                    "200 OK",
                    &format!("Location: {base}/upload/session/7\r\n"),
                    "",
                ),
                reply(
                    "200 OK",
                    "",
                    r#"{"id":"abc123XYZ","status":{"privacyStatus":"private"}}"#,
                ),
            ]
        });
        let id = send(&at(&base), &signed_in(), &video, "private").unwrap();
        assert_eq!(id, "abc123XYZ");

        // The token: asked for with the refresh token, as a form.
        let token = requests.recv().unwrap();
        assert!(token.starts_with("POST /token "), "{token}");
        assert!(
            token.ends_with(
                "client_id=the-client&client_secret=the%20secret%20%26%20more&\
                 refresh_token=the%2Frefresh%2Btoken&grant_type=refresh_token"
            ),
            "{token}"
        );
        // The session: the video's words as JSON, under the access token.
        let opened = requests.recv().unwrap();
        assert!(
            opened.starts_with(
                "POST /upload/youtube/v3/videos?uploadType=resumable&part=snippet,status "
            ),
            "{opened}"
        );
        let lower = opened.to_lowercase();
        assert!(
            lower.contains("authorization: bearer an-access-token\r\n"),
            "{opened}"
        );
        assert!(
            lower.contains("x-upload-content-length: 39\r\n"),
            "{opened}"
        );
        let body: serde_json::Value =
            serde_json::from_str(opened.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!(body["snippet"]["title"], "A \"quoted\" title");
        assert_eq!(
            body["snippet"]["description"],
            "Two lines\nof description, with a \\ in it."
        );
        assert_eq!(
            body["snippet"]["tags"],
            serde_json::json!(["one", "two words"])
        );
        assert_eq!(body["status"]["privacyStatus"], "private");
        // The file: its bytes, to the session's address.
        let sent = requests.recv().unwrap();
        assert!(sent.starts_with("PUT /upload/session/7 "), "{sent}");
        assert!(
            sent.to_lowercase()
                .contains("authorization: bearer an-access-token\r\n")
        );
        assert!(
            sent.ends_with("not really a video, but 39 bytes of one"),
            "{sent}"
        );
    }

    #[test]
    fn what_the_service_refuses_is_said_in_its_words() {
        let dir = tempfile::tempdir().unwrap();
        let video = video(dir.path());
        // A token that was withdrawn.
        let (base, _seen) = api(vec![reply(
            "400 Bad Request",
            "",
            r#"{"error":"invalid_grant","error_description":"Token has been expired or revoked."}"#,
        )]);
        let why = send(&at(&base), &signed_in(), &video, "private").unwrap_err();
        assert_eq!(
            why,
            "the sign-in was refused (400): Token has been expired or revoked.. `vidni upload --login` signs in again"
        );
        // The day's quota is used up: the session isn't opened.
        let (base, seen) = api(vec![
            reply("200 OK", "", r#"{"access_token":"t"}"#),
            reply(
                "403 Forbidden",
                "",
                r#"{"error":{"code":403,"message":"The request cannot be completed because you have exceeded your quota."}}"#,
            ),
        ]);
        let why = send(&at(&base), &signed_in(), &video, "private").unwrap_err();
        assert_eq!(
            why,
            "YouTube refused the upload (403): The request cannot be completed because you have exceeded your quota."
        );
        assert_eq!(seen.iter().take(2).count(), 2);
        // The file itself is refused, after the session opened.
        let (base, _seen) = {
            let (base, seen) = api(vec![
                reply("200 OK", "", r#"{"access_token":"t"}"#),
                // The session is on a port nobody listens on.
                reply("200 OK", "Location: http://127.0.0.1:9/gone\r\n", ""),
            ]);
            (base, seen)
        };
        let why = send(&at(&base), &signed_in(), &video, "private").unwrap_err();
        assert!(why.contains("127.0.0.1"), "{why}");
        // Nobody answers at all.
        let why = send(&at("http://127.0.0.1:9"), &signed_in(), &video, "private").unwrap_err();
        assert!(why.contains("127.0.0.1"), "{why}");
        // Not signed in: nothing is asked of anyone.
        let mut out = signed_in();
        out.refresh_token = None;
        let why = send(&at("http://127.0.0.1:9"), &out, &video, "private").unwrap_err();
        assert_eq!(
            why,
            "vidni isn't signed in to YouTube; `vidni upload --login` signs in"
        );
    }

    #[test]
    fn only_the_releases_own_name_confirms_an_upload() {
        assert!(confirmed("long-r2", Cursor::new("long-r2\n")));
        assert!(confirmed("long-r2", Cursor::new("  long-r2  \n")));
        for typed in ["", "\n", "y\n", "yes\n", "long-r1\n", "long\n"] {
            assert!(!confirmed("long-r2", Cursor::new(typed)), "{typed:?}");
        }
    }

    #[test]
    fn a_form_and_a_config_value_carry_any_character() {
        assert_eq!(
            form(&[("a", "x y&z=1/2+3"), ("b", "é")]),
            "a=x%20y%26z%3D1%2F2%2B3&b=%C3%A9"
        );
        assert_eq!(quoted("say \"hi\" \\ bye"), "say \\\"hi\\\" \\\\ bye");
        assert_eq!(decoded("4%2Fthe-code%20x+y%"), "4/the-code x y%");
    }
}
