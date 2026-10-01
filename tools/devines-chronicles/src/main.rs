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
        "apply-public-corrections" => {
            let Some(root) = args.next().map(PathBuf::from) else {
                eprintln!("usage: devines-chronicles apply-public-corrections REPO_ROOT BUNDLE_JSON YYYY-MM-DD");
                std::process::exit(2);
            };
            let Some(bundle) = args.next().map(PathBuf::from) else {
                eprintln!("usage: devines-chronicles apply-public-corrections REPO_ROOT BUNDLE_JSON YYYY-MM-DD");
                std::process::exit(2);
            };
            let Some(date) = args.next() else {
                eprintln!("usage: devines-chronicles apply-public-corrections REPO_ROOT BUNDLE_JSON YYYY-MM-DD");
                std::process::exit(2);
            };
            match feeds::apply_public_correction_bundle(&root, &bundle, &date) {
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
            eprintln!("usage: devines-chronicles validate [repo-root] | render-feeds [repo-root] | apply-public-corrections REPO_ROOT BUNDLE_JSON YYYY-MM-DD");
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
                        "D011" => (".gitbook/assets/beings-direct/D011.jpg".to_string(), Some("7fb339b70173dd6b1561e67139cde2a18ff147070ed6b1d9abf700f5725aeb7d")),
                        "D012" => (".gitbook/assets/beings-direct/D012.jpg".to_string(), Some("fefb9ddc54d59e15ba79679e61da5454e8898353cb09c7c7b66c89fa9c31d097")),
                        "D013" => (".gitbook/assets/beings-direct/D013.jpg".to_string(), Some("0c77685a304b5cafd592a396c868bf7bdd6015c13d799d899ff2da7c20d6ef83")),
                        "D014" => (".gitbook/assets/beings-direct/D014.jpg".to_string(), Some("a513185d0fe7d392f6536cff7138c19b21b073f375fe9dc762fd28d0c0299b22")),
                        "D015" => (".gitbook/assets/beings-direct/D015.jpg".to_string(), Some("da2f4e2dc96a69ffd57e2d03de29681b2964eed9321e62fdb0eba4aa2c3aa6ee")),
                        "D016" => (".gitbook/assets/beings-direct/D016.jpg".to_string(), Some("c7931291338f23eaab16b9d8cf00abf2cae9b6cb7cb00074d963717e5768c116")),
                        "D017" => (".gitbook/assets/beings-direct/D017.jpg".to_string(), Some("59daa908bfe687eb50fa2bc518c4d7f5523125aae0bbc9d8fc29741d77540cb2")),
                        "D018" => (".gitbook/assets/beings-direct/D018.jpg".to_string(), Some("08983cf9953d79dbcea9f9a5cbe1c41a124779859ec812fd8331d4eb619d54e0")),
                        "D019" => (".gitbook/assets/beings-direct/D019.jpg".to_string(), Some("a19129c7a86116c7ff9540de33a9113079b061747e6cb039d5f29ff7fd183d09")),
                        "D020" => (".gitbook/assets/beings-direct/D020.jpg".to_string(), Some("d86f999dd406c6b7c42a47529589132473c438db18a46e1223d619d533c1f5a3")),
                        "D021" => (".gitbook/assets/beings-direct/D021.jpg".to_string(), Some("664a158d9d97a73cd3634b01888ff131ad398309033daf109c11795bb588f188")),
                        "D022" => (".gitbook/assets/beings-direct/D022.jpg".to_string(), Some("4a448cec35ebd8fd9d47000b075ef300f6a3e37ff53e3f967ce26ac3e8acc085")),
                        "D174" => (".gitbook/assets/beings-direct/D174.jpg".to_string(), Some("e8ccbf1e51674337c85c0c8a51dfcf28bce269b090b5ccd986a1117adad7e8c3")),
                        "D285" => (".gitbook/assets/beings-direct/D285.jpg".to_string(), Some("7984dd97dd69088fa58d9dcdca119061390fef33015483f4b1fd8ed84d318730")),
                        "D396" => (".gitbook/assets/beings-direct/D396.jpg".to_string(), Some("3f8d1ad08ccdc5987036876c78aba0f32c89e14cba6709ddd9322cb862f73380")),
                        "D417" => (".gitbook/assets/beings-direct/D417.jpg".to_string(), Some("a43b86a6bd1a3a529a9b696ed84a0ea9db8654a65c1041b7c853552a5d7116bd")),
                        "D528" => (".gitbook/assets/beings-direct/D528.jpg".to_string(), Some("b89da992351a89cb12fafef669df12105be1c9c81e74d7a84d50514d703f10d3")),
                        "D639" => (".gitbook/assets/beings-direct/D639.jpg".to_string(), Some("5a806c60f00a65c2f4e43ba0fc323715f7395cbf1669f443fe64f99cd7f80454")),
                        "D741" => (".gitbook/assets/beings-direct/D741.jpg".to_string(), Some("aa8da399cde09d90fbb98258cb4111f17b1e6c6d7037786b7c4c3f700ba45390")),
                        "D852" => (".gitbook/assets/beings-direct/D852.jpg".to_string(), Some("d80cd7b7e84fe4a312a4b3cdac7e43a0f7cd218a6b3d2406bb2fa0b65110e809")),
                        "D963" => (".gitbook/assets/beings-direct/D963.jpg".to_string(), Some("ec6ab803a28dfa2ff6ad3e854f81b35fe508408c7d1936b0b8ea6d20d5079740")),
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
