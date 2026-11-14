//! What changed (TODO.md, P7-VIDNI-04): the project as it is now against
//! the project as it was at its last build, line by line.
//!
//! A build writes the project it was made from, with every default filled
//! in, as `build/resolved.json`. Compared with the project resolved now,
//! it says which lines are new, gone or changed, and in which keys.

use serde_json::Value;
use std::collections::BTreeSet;

/// A resolved project, read from the text of `build/resolved.json`.
pub fn parse(text: &str) -> Result<Value, String> {
    serde_json::from_str(text).map_err(|e| e.to_string())
}

/// The keys in which two tables differ, sorted.
fn differing(before: &Value, now: &Value) -> Vec<String> {
    let keys = |v: &Value| -> BTreeSet<String> {
        v.as_object()
            .map(|o| o.keys().cloned().collect())
            .unwrap_or_default()
    };
    keys(before)
        .union(&keys(now))
        .filter(|key| before.get(key.as_str()) != now.get(key.as_str()))
        .cloned()
        .collect()
}

/// What differs between two resolved projects, a sentence each: the
/// lines first, in the script's order now, then the lines that are gone,
/// the order, the sections, the scenes, the cuts and the rest.
pub fn between(before: &Value, now: &Value) -> Vec<String> {
    let lines = |v: &Value| -> Vec<(String, Value)> {
        v["lines"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|line| Some((line["id"].as_str()?.to_string(), line.clone())))
            .collect()
    };
    let (was, is) = (lines(before), lines(now));
    let mut found = Vec::new();
    for (id, line) in &is {
        match was.iter().find(|(old, _)| old == id) {
            None => found.push(format!("line {id} is new")),
            Some((_, old)) if old != line => {
                found.push(format!(
                    "line {id} changed: {}",
                    differing(old, line).join(", ")
                ));
            }
            Some(_) => {}
        }
    }
    for (id, _) in &was {
        if !is.iter().any(|(new, _)| new == id) {
            found.push(format!("line {id} is gone"));
        }
    }
    // The lines both have, in each one's order.
    let common = |lines: &[(String, Value)], other: &[(String, Value)]| -> Vec<String> {
        lines
            .iter()
            .filter(|(id, _)| other.iter().any(|(o, _)| o == id))
            .map(|(id, _)| id.clone())
            .collect()
    };
    if common(&was, &is) != common(&is, &was) {
        found.push("the lines are in another order".to_string());
    }
    for (table, what) in [("sections", "section"), ("scenes", "scene")] {
        let ids: BTreeSet<&String> = [before, now]
            .iter()
            .filter_map(|v| v[table].as_object())
            .flat_map(|o| o.keys())
            .collect();
        for id in ids {
            match (before[table].get(id), now[table].get(id)) {
                (None, Some(_)) => found.push(format!("{what} {id} is new")),
                (Some(_), None) => found.push(format!("{what} {id} is gone")),
                (Some(old), Some(new)) if old != new => {
                    found.push(format!(
                        "{what} {id} changed: {}",
                        differing(old, new).join(", ")
                    ));
                }
                _ => {}
            }
        }
    }
    // The cuts, each by its ID.
    let cuts = |v: &Value| -> Vec<(String, Value)> {
        v["cuts"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|cut| Some((cut["id"].as_str()?.to_string(), cut.clone())))
            .collect()
    };
    let (was, is) = (cuts(before), cuts(now));
    for (id, cut) in &is {
        match was.iter().find(|(old, _)| old == id) {
            None => found.push(format!("cut {id} is new")),
            Some((_, old)) if old != cut => {
                found.push(format!(
                    "cut {id} changed: {}",
                    differing(old, cut).join(", ")
                ));
            }
            Some(_) => {}
        }
    }
    for (id, _) in &was {
        if !is.iter().any(|(new, _)| new == id) {
            found.push(format!("cut {id} is gone"));
        }
    }
    for key in differing(before, now) {
        if !["lines", "sections", "scenes", "cuts"].contains(&key.as_str()) {
            found.push(format!("{key} changed"));
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::Project;
    use crate::ratify::resolved;

    fn project(text: &str) -> Value {
        resolved(&toml::from_str::<Project>(text).unwrap())
    }

    #[test]
    fn each_change_is_named_by_its_line_and_its_keys() {
        let before = project(crate::TEMPLATE);
        assert_eq!(between(&before, &before), Vec::<String>::new());
        let edited = crate::TEMPLATE
            .replace("show = \"Hello, and welcome.\"", "show = \"Hello.\"\nsay = \"Hullo.\"")
            .replace(
                "[[lines]]\nid = \"intro-2\"",
                "[[lines]]\nid = \"intro-1a\"\nsection = \"intro\"\nshow = \"New.\"\n\n[[lines]]\nid = \"intro-2\"",
            )
            .replace("label = \"Introduction\"", "label = \"Hello\"")
            .replace("scale = 2", "scale = 3")
            .replace("title = \"My first video\"", "title = \"Another title\"");
        assert_eq!(
            between(&before, &project(&edited)),
            [
                "line intro-1 changed: say, show",
                "line intro-1a is new",
                "section intro changed: label",
                "scene page changed: scale",
                "cut long changed: lines",
                "cut vertical changed: lines",
                "meta changed",
            ]
        );
        // The other way round, the new line is gone.
        assert!(between(&project(&edited), &before).contains(&"line intro-1a is gone".to_string()));
        // Two lines that change places.
        let swapped = {
            let (a, b) = (
                edited.find("[[lines]]\nid = \"intro-1a\"").unwrap(),
                edited.find("[[lines]]\nid = \"intro-2\"").unwrap(),
            );
            let end = edited.find("# What the lines show").unwrap();
            format!(
                "{}{}{}{}",
                &edited[..a],
                &edited[b..end],
                &edited[a..b],
                &edited[end..]
            )
        };
        assert_eq!(
            between(&project(&edited), &project(&swapped)),
            [
                "the lines are in another order",
                "cut long changed: lines",
                "cut vertical changed: lines"
            ]
        );
    }
}
