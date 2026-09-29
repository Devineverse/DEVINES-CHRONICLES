mod feeds;
mod review;
use serde_json::Value;
use std::collections::{BTreeSet, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "validate".to_string());

    match command.as_str() {
        "render-feeds" => {
            let root = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."));
            match feeds::render(&root) {
                Ok(report) => println!("{report}"),
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            }
        }
        "validate" => {
            let root = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."));
            match validate(&root) {
                Ok(report) => {
                    println!(
                        "PASS beings={} portraits={} as_of={} six_books=ok markdown_targets=ok identities=ok markets=35 hashes=ok state_structure=ok cycles=ok",
                        report.beings, report.portraits, report.as_of
                    );
                }
                Err(errors) => {
                    for error in errors {
                        eprintln!("{error}");
                    }
                    std::process::exit(1);
                }
            }
        }
        other => {
            eprintln!("unknown command: {other}");
            eprintln!("usage: devines-chronicles validate [repo-root]");
            std::process::exit(2);
        }
    }
}

#[derive(Debug)]
struct Report {
    beings: usize,
    portraits: usize,
    as_of: String,
}

fn validate(root: &Path) -> Result<Report, Vec<String>> {
    let mut errors = Vec::new();

    validate_summary(root, &mut errors);
    review::audit(root, &mut errors);

    let latest_path = root.join("PUBLIC_STATE/latest.json");
    let latest = read_json(&latest_path, root, &mut errors);
    let schema_path = root.join("SCHEMAS/public-cycle-v1.schema.json");
    let _schema = read_json(&schema_path, root, &mut errors);

    let mut ids = Vec::new();
    let mut as_of = String::new();

    if let Some(state) = latest.as_ref() {
        if state.get("schema").and_then(Value::as_str) != Some("devines.public-state.v1") {
            errors.push("state-schema:expected-devines.public-state.v1".into());
        }
        if state.get("mirror").and_then(Value::as_str) != Some("PUBLIC") {
            errors.push("state-mirror:expected-PUBLIC".into());
        }

        as_of = state
            .get("as_of")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();

        if !valid_date(&as_of) {
            errors.push(format!("invalid-as-of:{as_of}"));
        } else if !root.join(format!("PUBLIC_STATE/{as_of}.json")).exists() {
            errors.push(format!("missing-dated-state:{as_of}"));
        }

        match state.get("beings").and_then(Value::as_array) {
            Some(beings) => {
                for being in beings {
                    match being.get("being_id").and_then(Value::as_str) {
                        Some(id) if !id.trim().is_empty() => ids.push(id.to_string()),
                        _ => errors.push("state-being:missing-being_id".into()),
                    }
                }
            }
            None => errors.push("state-beings:not-array".into()),
        }
    }

    if ids.len() != 34 {
        errors.push(format!("being-count:{}", ids.len()));
    }

    let unique: HashSet<_> = ids.iter().cloned().collect();
    if unique.len() != ids.len() {
        errors.push("duplicate-being-id".into());
    }

    let page_ids = collect_markdown_stems(&root.join("BOOKS/BOOK-II-BEINGS"));
    for id in &ids {
        if !page_ids.contains(id) {
            errors.push(format!("missing-being-page:{id}"));
        }
    }

    if valid_date(&as_of) {
        let parts: Vec<_> = as_of.split('-').collect();
        let cycle_root = root.join(format!("CYCLES/{}/{}/{}", parts[0], parts[1], parts[2]));
        for id in &ids {
            if !cycle_root.join(format!("{id}.md")).exists() {
                errors.push(format!("missing-cycle:{as_of}:{id}"));
            }
        }
    }

    let asset_root = root.join(".gitbook/assets");
    if !asset_root.join("aum-sigil.webp").exists() {
        errors.push("missing-aum-asset".into());
    }
    if !asset_root.join("devines-chronicles-cover.webp").exists() {
        errors.push("missing-cover-asset".into());
    }

    let portraits = collect_webp_stems(&asset_root.join("beings"));
    for id in &unique {
        if !portraits.contains(id) {
            errors.push(format!("missing-portrait:{id}"));
        }
    }
    for id in portraits.difference(&unique.iter().cloned().collect()) {
        errors.push(format!("unexpected-portrait:{id}"));
    }
    if portraits.len() != 34 {
        errors.push(format!("portrait-count:{}", portraits.len()));
    }

    for id in &ids {
        let page = find_being_page(root, id);
        if let Some(page) = page {
            match fs::read_to_string(&page) {
                Ok(text) => {
                    let expected = format!(".gitbook/assets/beings/circle/{id}.webp");
                    if !text.contains(&expected) {
                        errors.push(format!("portrait-not-wired:{id}"));
                    }
                }
                Err(e) => errors.push(format!("read:{}:{e}", display(root, &page))),
            }
        }
    }

    if root.join(".staging/devines-gitbook-assets.tar.gz").exists() {
        errors.push("staging-archive-present".into());
    }

    if errors.is_empty() {
        Ok(Report {
            beings: ids.len(),
            portraits: portraits.len(),
            as_of,
        })
    } else {
        Err(errors)
    }
}

fn validate_summary(root: &Path, errors: &mut Vec<String>) {
    let path = root.join("SUMMARY.md");
    let summary = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            errors.push(format!("read:SUMMARY.md:{e}"));
            return;
        }
    };

    for target in markdown_links(&summary) {
        if target.starts_with("http://")
            || target.starts_with("https://")
            || target.starts_with('#')
            || !target.ends_with(".md")
        {
            continue;
        }
        if !root.join(&target).exists() {
            errors.push(format!("missing:{target}"));
        }
    }
}

fn markdown_links(text: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut cursor = 0;

    while let Some(open_rel) = text[cursor..].find("](") {
        let start = cursor + open_rel + 2;
        let Some(close_rel) = text[start..].find(')') else {
            break;
        };
        let end = start + close_rel;
        links.push(text[start..end].trim().to_string());
        cursor = end + 1;
    }

    links
}

fn read_json(path: &Path, root: &Path, errors: &mut Vec<String>) -> Option<Value> {
    match fs::read_to_string(path) {
        Ok(text) => match serde_json::from_str::<Value>(&text) {
            Ok(value) => Some(value),
            Err(e) => {
                errors.push(format!("json:{}:{e}", display(root, path)));
                None
            }
        },
        Err(e) => {
            errors.push(format!("read:{}:{e}", display(root, path)));
            None
        }
    }
}

fn valid_date(s: &str) -> bool {
    let parts: Vec<_> = s.split('-').collect();
    if parts.len() != 3 || parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2 {
        return false;
    }
    if !parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit())) {
        return false;
    }
    let month = parts[1].parse::<u8>().unwrap_or(0);
    let day = parts[2].parse::<u8>().unwrap_or(0);
    let year = parts[0].parse::<u32>().unwrap_or(0);
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        2 => {
            if leap {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => 0,
    };
    year > 0 && day > 0 && day <= days
}

fn collect_markdown_stems(root: &Path) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    walk_files(root, &mut |path| {
        if path.extension().and_then(|s| s.to_str()) == Some("md")
            && path.file_name().and_then(|s| s.to_str()) != Some("README.md")
        {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                out.insert(stem.to_string());
            }
        }
    });
    out
}

fn collect_webp_stems(root: &Path) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    walk_files(root, &mut |path| {
        if path.extension().and_then(|s| s.to_str()) == Some("webp") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                out.insert(stem.to_string());
            }
        }
    });
    out
}

fn walk_files(root: &Path, f: &mut impl FnMut(&Path)) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_files(&path, f);
        } else if path.is_file() {
            f(&path);
        }
    }
}

fn find_being_page(root: &Path, id: &str) -> Option<PathBuf> {
    let mut found = None;
    let book_root = root.join("BOOKS/BOOK-II-BEINGS");
    walk_files(&book_root, &mut |path| {
        if found.is_none()
            && path.extension().and_then(|s| s.to_str()) == Some("md")
            && path.file_stem().and_then(|s| s.to_str()) == Some(id)
        {
            found = Some(path.to_path_buf());
        }
    });
    found
}

fn display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}
