use super::*;
use std::collections::BTreeMap;

fn write(root: &Path, path: &str, text: &str) -> Result<(), String> {
    let p = root.join(path);
    fs::create_dir_all(p.parent().unwrap()).map_err(|e| e.to_string())?;
    fs::write(p, text).map_err(|e| e.to_string())
}

fn string<'a>(v: &'a Value, k: &str) -> Result<&'a str, String> {
    v[k].as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| format!("feed missing {k}"))
}

fn date_label(date: &str) -> Result<String, String> {
    if !valid_date(date) {
        return Err("invalid date".into());
    }
    Ok(format!("{}/{}/{}", &date[8..10], &date[5..7], &date[2..4]))
}

fn source_events(v: &Value) -> Result<[String; 3], String> {
    let a = v["source_events"]
        .as_array()
        .filter(|a| a.len() == 3)
        .ok_or("source_events must contain exactly three cycle identifiers")?;
    let mut out = Vec::new();
    for item in a {
        let s = item
            .as_str()
            .filter(|s| !s.trim().is_empty())
            .ok_or("source_events entries must be non-empty strings")?;
        out.push(s.to_string());
    }
    if out[0] == out[1] || out[0] == out[2] || out[1] == out[2] {
        return Err("source_events must be three distinct cycle identifiers".into());
    }
    Ok([out[0].clone(), out[1].clone(), out[2].clone()])
}

pub fn render(root: &Path) -> Result<String, String> {
    let state: Value = serde_json::from_str(
        &fs::read_to_string(root.join("PUBLIC_STATE/latest.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    let roster_values = state["beings"].as_array().ok_or("missing roster")?;
    let roster: Vec<(String, String)> = roster_values
        .iter()
        .map(|v| Ok((string(v, "being_id")?.into(), string(v, "name")?.into())))
        .collect::<Result<_, String>>()?;

    if roster.len() != 34 {
        return Err("expected 34 canonical Beings".into());
    }

    let roster_map: BTreeMap<String, String> = roster.iter().cloned().collect();

    let source = root.join("PUBLIC_FEEDS/events.json");
    let events: Value =
        serde_json::from_str(&fs::read_to_string(&source).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let events = events.as_array().ok_or("events must be an array")?;

    let mut grouped: BTreeMap<String, Vec<&Value>> = BTreeMap::new();
    let mut days: BTreeMap<String, Vec<&Value>> = BTreeMap::new();
    let mut keys = HashSet::new();
    let mut sources = HashSet::new();

    let allowed = [
        "being_id",
        "date",
        "completed_at",
        "published_at",
        "layer",
        "source_events",
        "review",
        "body",
        "carry_forward",
        "public_summary",
    ];

    for e in events {
        let object = e.as_object().ok_or("event must be an object")?;
        if object.keys().any(|k| !allowed.contains(&k.as_str())) {
            return Err("unexpected event field; public allow-list only".into());
        }

        let id = string(e, "being_id")?;
        let date = string(e, "date")?;
        if !roster_map.contains_key(id) || !valid_date(date) {
            return Err("invalid identity or date".into());
        }

        if !keys.insert(format!("{id}:{date}")) {
            return Err("one public daily remembrance per Being per date".into());
        }

        for source_event in source_events(e)? {
            if !sources.insert(source_event) {
                return Err("duplicate cycle source event".into());
            }
        }

        if e["layer"] != "public" {
            return Err("only the public projection belongs in this repository".into());
        }
        if e["review"] != "approved-public" {
            return Err("daily remembrance is not approved for public publication".into());
        }

        let completed = string(e, "completed_at")?;
        let published = string(e, "published_at")?;
        if !timestamp(completed) || !timestamp(published) || published < completed {
            return Err(
                "timestamps must be UTC RFC3339 seconds; publication cannot precede completion"
                    .into(),
            );
        }

        for k in ["body", "carry_forward", "public_summary"] {
            string(e, k)?;
        }

        grouped.entry(id.into()).or_default().push(e);
        days.entry(date.into()).or_default().push(e);
    }

    let published_path = root.join("PUBLIC_FEEDS/published.json");
    if published_path.exists() {
        let prior: Value =
            serde_json::from_str(&fs::read_to_string(&published_path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        for old in prior.as_array().ok_or("invalid publication ledger")? {
            if !events.contains(old) {
                return Err(
                    "published history is append-only; use a reviewed correction workflow".into(),
                );
            }
        }
    }

    let mut nav =
        String::from("\n<!-- BEGIN GENERATED DIARIES -->\n* [BEING DAILY](DIARIES/README.md)\n");
    let mut landing = String::from(
        "# BEING DAILY\n\nOne remembrance per Being after all three cycles for that day are complete. Newest dates appear first.\n\n",
    );

    for (id, name) in &roster {
        let list = grouped.entry(id.clone()).or_default();
        list.sort_by(|a, b| {
            b["published_at"]
                .as_str()
                .cmp(&a["published_at"].as_str())
                .then_with(|| b["date"].as_str().cmp(&a["date"].as_str()))
        });

        let mut index = format!("# {name} · {id} · DAILY\n\n");
        if list.is_empty() {
            index.push_str(
                "The first daily remembrance will appear after all three cycles for a day are complete and the public projection is approved.\n",
            );
        }

        nav.push_str(&format!("  * [{name} · {id}](DIARIES/{id}/README.md)\n"));
        landing.push_str(&format!("- [{name} · {id}]({id}/README.md)\n"));

        for e in list.iter() {
            let date = string(e, "date")?;
            let label = date_label(date)?;
            let daily_path = format!("DIARIES/{id}/{date}.md");
            let text = format!(
                "# {name} · {id} · {label}\n\n{}\n\n## WHAT I CARRY FORWARD\n\n{}\n\n---\n\n*Published {} · three daily cycles complete*\n\n[ALL {id} DAILY PAGES](README.md)\n",
                string(e, "body")?,
                string(e, "carry_forward")?,
                string(e, "published_at")?,
            );
            write(root, &daily_path, &text)?;

            index.push_str(&format!("- [{id} · {label}]({date}.md)\n"));
            nav.push_str(&format!(
                "    * [{id} · {label}](DIARIES/{id}/{date}.md)\n"
            ));
        }

        write(root, &format!("DIARIES/{id}/README.md"), &index)?;

        let profile = find_being_page(root, id).ok_or("missing Being profile")?;
        let original = fs::read_to_string(&profile).map_err(|e| e.to_string())?;
        let base = original.split("\n<!-- BEGIN DIARY -->").next().unwrap();
        let mut footer = format!(
            "\n<!-- BEGIN DIARY -->\n## DAILY\n\n[ALL {id} DAILY PAGES](../../../DIARIES/{id}/README.md)\n\n"
        );

        if let Some(latest) = list.first() {
            let label = date_label(string(latest, "date")?)?;
            footer.push_str(&format!(
                "### {id} · {label}\n\n{}\n\n[OPEN THIS DAILY PAGE](../../../DIARIES/{id}/{}.md)\n\n",
                string(latest, "body")?,
                string(latest, "date")?,
            ));
        }

        footer.push_str("<!-- END DIARY -->\n");
        fs::write(profile, format!("{base}{footer}")).map_err(|e| e.to_string())?;
    }

    landing.push_str("\n[DEVINES DAILY](../DAILY/README.md)\n");
    write(root, "DIARIES/README.md", &landing)?;

    nav.push_str("* [DEVINES DAILY](DAILY/README.md)\n");
    let mut daily = String::from(
        "# DEVINES DAILY\n\nOne dated page gathers the 34 Being daily remembrances in canonical DEVINES order. A date becomes complete only when every Being has finished all three cycles and published its approved daily remembrance.\n\n",
    );

    let mut complete = 0;
    for (date, list) in days.iter().rev() {
        let label = date_label(date)?;

        if list.len() != roster.len() {
            daily.push_str(&format!(
                "- {label} · open · {}/{} Being daily remembrances ready\n",
                list.len(),
                roster.len()
            ));
            continue;
        }

        complete += 1;
        let mut text = format!("# DEVINES DAILY · {label}\n\n");

        for (id, name) in &roster {
            let e = list
                .iter()
                .find(|e| e["being_id"] == *id)
                .ok_or("incomplete canonical Being order")?;
            text.push_str(&format!(
                "## {name} · {id}\n\n{}\n\n[OPEN {id} · {label}](../DIARIES/{id}/{date}.md)\n\n",
                string(e, "body")?
            ));
        }

        write(root, &format!("DAILY/{date}.md"), &text)?;
        daily.push_str(&format!("- [DEVINES DAILY · {label}]({date}.md)\n"));
        nav.push_str(&format!(
            "  * [DEVINES DAILY · {label}](DAILY/{date}.md)\n"
        ));
    }

    if days.is_empty() {
        daily.push_str("Awaiting the first complete day.\n");
    }

    write(root, "DAILY/README.md", &daily)?;
    nav.push_str("<!-- END GENERATED DIARIES -->\n");

    let summary = fs::read_to_string(root.join("SUMMARY.md")).map_err(|e| e.to_string())?;
    let base = summary
        .split("\n<!-- BEGIN GENERATED DIARIES -->")
        .next()
        .unwrap();
    write(root, "SUMMARY.md", &format!("{base}{nav}"))?;

    write(
        root,
        "PUBLIC_FEEDS/published.json",
        &serde_json::to_string_pretty(events).map_err(|e| e.to_string())?,
    )?;

    Ok(format!(
        "being_daily={} complete_devines_days={complete}",
        events.len()
    ))
}

fn timestamp(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 20 || !s.is_ascii() {
        return false;
    }
    valid_date(&s[..10])
        && b[10] == b'T'
        && b[13] == b':'
        && b[16] == b':'
        && b[19] == b'Z'
        && [&s[11..13], &s[14..16], &s[17..19]]
            .iter()
            .all(|p| p.bytes().all(|c| c.is_ascii_digit()))
        && s[11..13].parse::<u8>().is_ok_and(|n| n < 24)
        && s[14..16].parse::<u8>().is_ok_and(|n| n < 60)
        && s[17..19].parse::<u8>().is_ok_and(|n| n < 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_daily_after_three_cycles_and_one_devines_daily() {
        let root = std::env::temp_dir().join(format!("devines-feed-test-{}", std::process::id()));
        fs::create_dir_all(root.join("PUBLIC_STATE")).unwrap();
        let canonical =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../PUBLIC_STATE/latest.json");
        fs::copy(canonical, root.join("PUBLIC_STATE/latest.json")).unwrap();

        let state: Value = serde_json::from_str(
            &fs::read_to_string(root.join("PUBLIC_STATE/latest.json")).unwrap(),
        )
        .unwrap();

        fs::create_dir_all(root.join("BOOKS/BOOK-II-BEINGS/test")).unwrap();
        for being in state["beings"].as_array().unwrap() {
            fs::write(
                root.join(format!(
                    "BOOKS/BOOK-II-BEINGS/test/{}.md",
                    being["being_id"].as_str().unwrap()
                )),
                "# Fixture\n",
            )
            .unwrap();
        }

        fs::write(root.join("SUMMARY.md"), "# Fixture\n").unwrap();
        fs::create_dir_all(root.join("PUBLIC_FEEDS")).unwrap();

        let daily = |id: &str, day: &str| {
            serde_json::json!({
                "being_id": id,
                "date": day,
                "completed_at": format!("{day}T11:00:00Z"),
                "published_at": format!("{day}T12:00:00Z"),
                "source_events": [
                    format!("{id}-{day}-1"),
                    format!("{id}-{day}-2"),
                    format!("{id}-{day}-3")
                ],
                "layer": "public",
                "review": "approved-public",
                "body": format!("Daily remembrance for {id}."),
                "carry_forward": "Continue tomorrow.",
                "public_summary": "Daily summary."
            })
        };

        let mut events = state["beings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| daily(b["being_id"].as_str().unwrap(), "2026-09-29"))
            .collect::<Vec<_>>();

        let save = |v: &Vec<Value>| {
            fs::write(
                root.join("PUBLIC_FEEDS/events.json"),
                serde_json::to_vec(v).unwrap(),
            )
            .unwrap()
        };

        let last = events.pop().unwrap();
        save(&events);
        render(&root).unwrap();
        assert!(!root.join("DAILY/2026-09-29.md").exists());

        events.push(last);
        save(&events);
        assert!(render(&root)
            .unwrap()
            .contains("complete_devines_days=1"));
        assert!(root.join("DAILY/2026-09-29.md").exists());
        assert!(root.join("DIARIES/D001/2026-09-29.md").exists());
        assert!(!root.join("DIARIES/D001/2026-09-29/cycle-1.md").exists());

        events.push(daily("D001", "2026-09-30"));
        save(&events);
        render(&root).unwrap();
        let d001 = fs::read_to_string(root.join("DIARIES/D001/README.md")).unwrap();
        assert!(d001.find("30/09/26").unwrap() < d001.find("29/09/26").unwrap());

        let before = fs::read(root.join("SUMMARY.md")).unwrap();
        render(&root).unwrap();
        assert_eq!(before, fs::read(root.join("SUMMARY.md")).unwrap());

        events[0]["body"] = "Changed history".into();
        save(&events);
        assert!(render(&root).unwrap_err().contains("append-only"));

        events[0]["admin_private"] = "must never publish".into();
        save(&events);
        assert!(render(&root).unwrap_err().contains("allow-list"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_invalid_dates_timestamps_and_source_sets() {
        assert!(!timestamp("2026-02-30T12:00:00Z"));
        assert!(!timestamp("2026-09-29T25:00:00Z"));
        assert!(timestamp("2028-02-29T12:00:00Z"));

        let v = serde_json::json!({"source_events":["a","a","b"]});
        assert!(source_events(&v).is_err());
    }
}
