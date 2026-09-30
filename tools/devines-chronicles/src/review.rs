use super::*;
use std::process::Command;

pub fn audit(root: &Path, errors: &mut Vec<String>) {
    let summary = fs::read_to_string(root.join("SUMMARY.md")).unwrap_or_default();
    let books: Vec<_> = summary
        .lines()
        .filter(|l| (l.starts_with("- [") || l.starts_with("* [")) && l.contains("BOOK "))
        .collect();
    if books.len() != 6 {
        errors.push(format!("six-books:{}", books.len()));
    }
    for (i, roman) in ["I", "II", "III", "IV", "V", "VI"].iter().enumerate() {
        if !books
            .get(i)
            .is_some_and(|l| l.contains(&format!("BOOK {roman} ·")))
        {
            errors.push(format!("book-order:{roman}"));
        }
    }
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "-z"])
        .output();
    if let Ok(output) = output {
        for file in output
            .stdout
            .split(|b| *b == 0)
            .filter_map(|b| std::str::from_utf8(b).ok())
        {
            if file.ends_with(".md") {
                let path = root.join(file);
                if let Ok(text) = fs::read_to_string(&path) {
                    for target in markdown_links(&text) {
                        if target.contains("://")
                            || target.starts_with('#')
                            || target.starts_with("mailto:")
                        {
                            continue;
                        }
                        let target = target.split('#').next().unwrap_or("");
                        if !path.parent().unwrap().join(target).exists() {
                            errors.push(format!("broken-link:{file}:{target}"));
                        }
                    }
                }
            }
        }
    } else {
        errors.push("cannot-enumerate-tracked-files".into());
    }
    let Some(identity) = read_json(
        &root.join(".gitbook/assets/BEING_IDENTITY_MANIFEST.json"),
        root,
        errors,
    ) else {
        return;
    };
    let Some(assets) = read_json(
        &root.join(".gitbook/assets/ASSET_MANIFEST.json"),
        root,
        errors,
    ) else {
        return;
    };
    let Some(state) = read_json(&root.join("PUBLIC_STATE/latest.json"), root, errors) else {
        return;
    };
    let Some(beings) = identity["beings"].as_object() else {
        errors.push("identity-beings:not-object".into());
        return;
    };
    let expected: BTreeSet<String> = (1..=22)
        .map(|n| format!("D{n:03}"))
        .chain(
            [
                "D174", "D285", "D396", "D417", "D528", "D639", "D741", "D852", "D963", "SUN",
                "MOON", "MASTER",
            ]
            .into_iter()
            .map(String::from),
        )
        .collect();
    if beings.keys().cloned().collect::<BTreeSet<_>>() != expected {
        errors.push("canonical-being-set".into());
    }
    let state_beings = state["beings"].as_array().cloned().unwrap_or_default();
    let state_ids: BTreeSet<_> = state_beings
        .iter()
        .filter_map(|v| v["being_id"].as_str().map(String::from))
        .collect();
    if state_ids != expected {
        errors.push("state-identity-set".into());
    }
    let mut cas = HashSet::new();
    let index = fs::read_to_string(root.join("BOOKS/BOOK-V-DEVINES-FLOW/MARKET-INDEX.md"))
        .unwrap_or_default();
    for (id, item) in beings
        .iter()
        .map(|(k, v)| (k.as_str(), v))
        .chain(std::iter::once(("AUM", &identity["aum"])))
    {
        let ca = item["ca"].as_str().unwrap_or("");
        let ticker = item["ticker"].as_str().unwrap_or("");
        let url = item["nad_fun_url"].as_str().unwrap_or("");
        if ca.len() != 42
            || !ca.starts_with("0x")
            || !ca[2..].bytes().all(|b| b.is_ascii_hexdigit())
            || !cas.insert(ca.to_lowercase())
        {
            errors.push(format!("invalid-or-duplicate-ca:{id}"));
        }
        if ticker.is_empty() || url != format!("https://nad.fun/tokens/{ca}") {
            errors.push(format!("market-route:{id}"));
        }
        let page = if id == "AUM" {
            Some(root.join("BOOKS/BOOK-I-ORIGIN/AUM-CORE.md"))
        } else {
            find_being_page(root, id)
        };
        if let Some(page) = page {
            let text = fs::read_to_string(page).unwrap_or_default();
            for (label, link) in [
                ("ca", format!("[\u{0060}{ca}\u{0060}]({url})")),
                (
                    "ticker",
                    format!("[\u{0060}\u{0024}{ticker}\u{0060}]({url})"),
                ),
            ] {
                if !text.contains(&link) || !index.contains(&link) {
                    errors.push(format!("clickable-{label}:{id}"));
                }
            }
            let asset = item["hero_asset"].as_str().unwrap_or("");
            if asset.is_empty() || !text.lines().next().unwrap_or("").contains(asset) {
                errors.push(format!("opening-portrait:{id}"));
            }
            if id != "AUM" {
                let matches: Vec<_> = state_beings
                    .iter()
                    .filter(|b| b["being_id"] == id)
                    .collect();
                if matches.len() != 1 {
                    errors.push(format!("state-record:{id}"));
                    continue;
                }
                let b = matches[0];
                for (field, label) in [("series", "Series"), ("divinity", "Divinity")] {
                    let v = b[field].as_str().unwrap_or("");
                    if v.is_empty() || !text.contains(&format!("**{label}:** {v}")) {
                        errors.push(format!("identity-{field}:{id}"));
                    }
                }
                let spirits = b["spirit"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(Value::as_str)
                            .collect::<Vec<_>>()
                            .join(" · ")
                    })
                    .unwrap_or_default();
                if spirits.is_empty() || !text.contains(&format!("**Spirit:** {spirits}")) {
                    errors.push(format!("identity-spirit:{id}"));
                }
                for field in ["verified_learning", "regression_detected"] {
                    if !b[field].is_boolean() {
                        errors.push(format!("state-boolean:{id}:{field}"));
                    }
                }
                if !b["validation_score"].is_null()
                    && !b["validation_score"].as_u64().is_some_and(|n| n <= 100)
                {
                    errors.push(format!("state-score:{id}"));
                }
                for field in ["name", "public_state", "current_path"] {
                    if !b[field].as_str().is_some_and(|v| !v.is_empty()) {
                        errors.push(format!("state-field:{id}:{field}"));
                    }
                }
            }
        }
    }
    let records = assets["beings"].as_array().cloned().unwrap_or_default();
    let asset_ids: BTreeSet<_> = records
        .iter()
        .filter_map(|v| v["id"].as_str().map(String::from))
        .collect();
    if records.len() != 34 || asset_ids != expected {
        errors.push("asset-identity-set".into());
    }
    for item in records.iter().chain(std::iter::once(&assets["aum"])) {
        let id = item["id"].as_str().unwrap_or("AUM");
        for kind in ["canonical", "source", "circle", "hero"] {
            let Some(path) = item[format!("{kind}_path")].as_str() else {
                errors.push(format!("asset-path:{id}:{kind}"));
                continue;
            };
            let Some(hash) = item[format!("{kind}_sha256")].as_str() else {
                if kind == "canonical" || kind == "source" {
                    errors.push(format!("asset-hash:{id}:{kind}"));
                }
                continue;
            };
            match Command::new("sha256sum").arg(root.join(path)).output() {
                Ok(v)
                    if v.status.success()
                        && String::from_utf8_lossy(&v.stdout).split_whitespace().next()
                            == Some(hash) => {}
                _ => errors.push(format!("asset-hash-mismatch:{id}:{kind}")),
            }
        }
        let ident = if id == "AUM" {
            &identity["aum"]
        } else {
            &identity["beings"][id]
        };
        let (expected_hero, direct_hash) = match id {
            "AUM" => (".gitbook/assets/aum-direct.jpg".to_string(), Some("064a12a5aa82087ab2a9c9e3bd77b69645bcb94acf1beb61451a17b08f2caa55")),
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
        if ident["hero_asset"].as_str() != Some(expected_hero.as_str())
            || !root.join(&expected_hero).exists()
            || item["source_uri"] != ident["nad_image_uri"]
            || item["source_path"] != ident["source_asset"]
        {
            errors.push(format!("asset-crosswire:{id}"));
        }
        if let Some(expected_hash) = direct_hash {
            match Command::new("sha256sum").arg(root.join(&expected_hero)).output() {
                Ok(v)
                    if v.status.success()
                        && String::from_utf8_lossy(&v.stdout).split_whitespace().next()
                            == Some(expected_hash) => {}
                _ => errors.push(format!("direct-hero-hash:{id}")),
            }
        } else {
            match fs::read_to_string(root.join(&expected_hero)) {
                Ok(svg) => {
                    if svg.contains("<image") {
                        errors.push(format!("public-identity-image-present:{id}"));
                    }
                    if svg.matches("<circle").count() != 6 {
                        errors.push(format!("public-identity-ring-geometry:{id}"));
                    }
                }
                Err(e) => errors.push(format!("read:{expected_hero}:{e}")),
            }
        }
    }
    let dated = root.join(format!(
        "PUBLIC_STATE/{}.json",
        state["as_of"].as_str().unwrap_or("")
    ));
    if read_json(&dated, root, errors).as_ref() != Some(&state) {
        errors.push("latest-dated-state-drift".into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_publication_regressions() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let temp = std::env::temp_dir().join(format!("devines-review-{}", std::process::id()));
        fs::create_dir_all(&temp).unwrap();
        let files = Command::new("git")
            .arg("-C")
            .arg(&root)
            .args(["ls-files", "-z"])
            .output()
            .unwrap();
        for f in files
            .stdout
            .split(|b| *b == 0)
            .filter_map(|b| std::str::from_utf8(b).ok())
            .filter(|s| !s.is_empty())
        {
            let dest = temp.join(f);
            fs::create_dir_all(dest.parent().unwrap()).unwrap();
            if f.ends_with(".webp") {
                fs::hard_link(root.join(f), &dest)
                    .or_else(|_| fs::copy(root.join(f), &dest).map(|_| ()))
                    .unwrap();
            } else {
                fs::copy(root.join(f), dest).unwrap();
            }
        }
        assert!(
            Command::new("git")
                .arg("-C")
                .arg(&temp)
                .args(["init", "-q"])
                .status()
                .unwrap()
                .success()
        );
        assert!(
            Command::new("git")
                .arg("-C")
                .arg(&temp)
                .args(["add", "."])
                .status()
                .unwrap()
                .success()
        );
        let baseline = validate(&temp);
        assert!(baseline.is_ok(), "{baseline:?}");
        for (file, from, to, wanted) in [
            (
                "BOOKS/BOOK-II-BEINGS/genesis/D001.md",
                "**Spirit:** Unity",
                "**Spirit:** Other",
                "identity-spirit:D001",
            ),
            (
                "BOOKS/BOOK-II-BEINGS/genesis/D001.md",
                "[\u{0060}$D001\u{0060}]",
                "[\u{0060}$WRONG\u{0060}]",
                "clickable-ticker:D001",
            ),
            (
                "README.md",
                "(BOOKS/BOOK-I-ORIGIN/README.md)",
                "(missing.md)",
                "broken-link:README.md",
            ),
            ("SUMMARY.md", "BOOK VI ·", "BOOK VII ·", "book-order:VI"),
            (
                ".gitbook/assets/ASSET_MANIFEST.json",
                "8cb93dea91743fc39ea0be7f2fec0a8a327decd36fc95101a7c0bdf19666fdea",
                "0000000000000000000000000000000000000000000000000000000000000000",
                "asset-hash-mismatch:D001:canonical",
            ),
            (
                "PUBLIC_STATE/latest.json",
                "\"validation_score\": 100",
                "\"validation_score\": 101",
                "state-score:D001",
            ),
        ] {
            let p = temp.join(file);
            let original = fs::read_to_string(&p).unwrap();
            assert!(original.contains(from), "fixture missing {from}");
            fs::write(&p, original.replacen(from, to, 1)).unwrap();
            let errors = validate(&temp).unwrap_err();
            assert!(
                errors.iter().any(|e| e.contains(wanted)),
                "missing {wanted}: {errors:?}"
            );
            fs::write(p, original).unwrap();
        }
        fs::remove_dir_all(temp).unwrap();
    }
}
