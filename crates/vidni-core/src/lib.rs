//! vidni's core: the project model, validation, line IDs, hashes and locks,
//! per-cut state, the timeline and caption cues, lint and platform profiles.
//!
//! Everything here is deterministic and does no media I/O. See `TODO.md`,
//! milestone P2.

pub mod project;
pub mod validate;

/// The vidni version, shared by every crate and front end.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The JSON Schema of `vidni.toml`, for editors (`docs/vidni.schema.json`).
pub fn schema_json() -> String {
    let schema = schemars::schema_for!(project::Project);
    serde_json::to_string_pretty(&schema).expect("a JSON Schema always serializes") + "\n"
}

#[cfg(test)]
mod tests {
    #[test]
    fn version_is_semver() {
        let parts: Vec<&str> = super::VERSION.split('.').collect();
        assert_eq!(parts.len(), 3, "{}", super::VERSION);
        assert!(parts.iter().all(|p| p.parse::<u64>().is_ok()));
    }
}
