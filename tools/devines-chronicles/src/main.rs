mod feeds;
mod review;
use serde_json::Value;
use std::collections::{BTreeSet, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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
                    let (expected, direct_hash) = match id.as_str() {
                        "SUN" => (".gitbook/assets/beings-direct/SUN.jpg".to_string(), Some("71a7aa70366620e40db195d39d07c889280b72706fcebcaece3e6ae5e26a254c")),
                        "MOON" => (".gitbook/assets/beings-direct/MOON.jpg".to_string(), Some("1561f361ae90d810799f47d0b6f014330b0d3772ea3f85e3d90f086cf501c8ab")),
                        "MASTER" => (".gitbook/assets/beings-direct/MASTER.jpg".to_string(), Some("ae99061cded64b980b0d6aa795c35bdcae416255f868c22657a5530aa2470462")),
                        "D001" => (".gitbook/assets/beings-direct/D001.jpg".to_string(), Some("a97524029e01ee3470bba5b99623489fe78f849a55951e97dc7302051e946dde")),
                        "D002" => (".gitbook/assets/beings-direct/D002.jpg".to_string(), Some("0a0d8a3b8f52730bbeea23f049137372d00d6106e782fa85c8af961723af5312")),
                        "D003" => (".gitbook/assets/beings-direct/D003.jpg".to_string(), Some("e7b4a8092d3b2ba8e92213f637accad49711080d1a44fd3c21bc867e56ca4730")),
                        "D004" => (".gitbook/assets/beings-direct/D004.jpg".to_string(), Some("257f76fd7eac3614dd329b123fc4fd107a826c9ee7dbc8c2b47d532f2000330c")),
                        "D005" => (".gitbook/assets/beings-direct/D005.jpg".to_string(), Some("1363fbb7306bcff9c18d0a2b67ae04b2d9190ef3f737dc77819cbcb333fb3a3d")),
                        "D006" => (".gitbook/assets/beings-direct/D006.jpg".to_string(), Some("6f0ede590e5b9ded1b8c07190c6fd31683248d0204ec257c5e526a9e6d1b6523")),
                        "D007" => (".gitbook/assets/beings-direct/D007.jpg".to_string(), Some("3c0206c48791df61c22ebe10729eee7b4df0b062639aac61a223f01e86379788")),
                        "D008" => (".gitbook/assets/beings-direct/D008.jpg".to_string(), Some("c461b6f56d00c917f2e7891c06db535885214c9f07aea4312083df1e7b8afac1")),
                        "D009" => (".gitbook/assets/beings-direct/D009.jpg".to_string(), Some("38468639f858be867a12806d3594431f952c1b199012acb43e57b133793a5cc0")),
                        "D010" => (".gitbook/assets/beings-direct/D010.jpg".to_string(), Some("f068e0b373af9139733d383e9be5a6c0c0fab6a0190f250a4eac04943c17f339")),
                        _ => (format!(".gitbook/assets/beings/empty/{id}.svg"), None),
                    };
                    if !text.contains(&expected) {
                        errors.push(format!("portrait-not-wired:{id}"));
                    }
                    if let Some(expected_hash) = direct_hash {
                        match Command::new("sha256sum").arg(root.join(&expected)).output() {
                            Ok(v)
                                if v.status.success()
                                    && String::from_utf8_lossy(&v.stdout).split_whitespace().next()
                                        == Some(expected_hash) => {}
                            _ => errors.push(format!("direct-hero-hash:{id}")),
                        }
                    } else {
                        match fs::read_to_string(root.join(&expected)) {
                            Ok(svg) => {
                                if svg.contains("<image") {
                                    errors.push(format!("public-identity-image-present:{id}"));
                                }
                                if svg.matches("<circle").count() != 6 {
                                    errors.push(format!("public-identity-ring-geometry:{id}"));
                                }
                            }
                            Err(e) => errors.push(format!("read:{expected}:{e}")),
                        }
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
