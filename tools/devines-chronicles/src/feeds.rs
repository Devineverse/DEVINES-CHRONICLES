use super::*;
use std::collections::BTreeMap;

const POSTS_PER_PAGE: usize = 12;
const PROFILE_PREVIEW_POSTS: usize = 3;

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

fn cleanup_being_history(root: &Path, id: &str) -> Result<(), String> {
    let dir = root.join(format!("DIARIES/{id}"));
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().and_then(|v| v.to_str()) != Some("md") {
            continue;
        }
        let name = path.file_name().and_then(|v| v.to_str()).unwrap_or("");
        if name == "README.md" {
            continue;
        }
        let stem = path.file_stem().and_then(|v| v.to_str()).unwrap_or("");
        if name.starts_with("page-") || valid_date(stem) {
            fs::remove_file(path).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn source_events(v: &Value) -> Result<Vec<String>, String> {
    let a = v["source_events"]
        .as_array()
        .ok_or("source_events must be an array")?;
    if a.len() > 3 {
        return Err("source_events may contain at most three cycle identifiers".into());
    }
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for item in a {
        let s = item
            .as_str()
            .filter(|s| !s.trim().is_empty())
            .ok_or("source_events entries must be non-empty strings")?;
        if !seen.insert(s.to_string()) {
            return Err("source_events must be distinct cycle identifiers".into());
        }
        out.push(s.to_string());
    }
    Ok(out)
}

fn event_kind(v: &Value) -> &str {
    v["event_kind"].as_str().unwrap_or("DAILY_REMEMBRANCE")
}

fn catchup(v: &Value) -> bool {
    event_kind(v) == "CATCH_UP_REFLECTION"
}

fn completion_note(v: &Value) -> String {
    if catchup(v) {
        format!(
            "catch-up reflection · {}/3 verified cycles",
            v["verified_cycle_count"].as_u64().unwrap_or(0)
        )
    } else {
        "three daily cycles complete".into()
    }
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
        "event_kind",
        "verified_cycle_count",
        "expected_cycle_count",
        "complete_day",
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

        let source_events = source_events(e)?;
        for source_event in &source_events {
            if !sources.insert(source_event.clone()) {
                return Err("duplicate cycle source event".into());
            }
        }

        if e["layer"] != "public" {
            return Err("only the public projection belongs in this repository".into());
        }

        match event_kind(e) {
            "DAILY_REMEMBRANCE" => {
                if source_events.len() != 3 {
                    return Err("daily remembrance requires exactly three cycle identifiers".into());
                }
                if e["review"] != "approved-public" {
                    return Err("daily remembrance is not approved for public publication".into());
                }
            }
            "CATCH_UP_REFLECTION" => {
                if e["review"] != "approved-public-catchup" {
                    return Err("catch-up reflection is not approved for public publication".into());
                }
                if e["expected_cycle_count"].as_u64() != Some(3)
                    || e["verified_cycle_count"].as_u64() != Some(source_events.len() as u64)
                    || e["complete_day"] != false
                {
                    return Err("invalid catch-up reflection cycle accounting".into());
                }
            }
            _ => return Err("invalid public event kind".into()),
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
        "# BEING DAILY\n\nOne public diary entry per Being and date. Normal daily remembrances are written only after all three cycles are complete. Founder-authorized DEV RHYTHM catch-up reflections may also appear when a past day needs truthful recovery; they preserve verified-cycle counts and never claim 3/3 without three receipts. Each Being profile shows the latest three approved public entries. Full history is grouped into pages of twelve posts. Page 1 is the earliest page; the highest page number is always the latest.\n\n",
    );
    let mut page_for_date: BTreeMap<(String, String), usize> = BTreeMap::new();

    for (id, name) in &roster {
        let list = grouped.entry(id.clone()).or_default();
        list.sort_by(|a, b| {
            a["published_at"]
                .as_str()
                .cmp(&b["published_at"].as_str())
                .then_with(|| a["date"].as_str().cmp(&b["date"].as_str()))
        });

        cleanup_being_history(root, id)?;

        let mut index = format!(
            "# {id} Diary\n\n**{name}**\n\nTwelve daily posts per page. Page 1 begins the history; the highest page number contains the latest posts. Inside each page, the newest post appears first.\n\n"
        );

        nav.push_str(&format!("  * [{id} Diary · {name}](DIARIES/{id}/README.md)\n"));
        landing.push_str(&format!("- [{id} Diary · {name}]({id}/README.md)\n"));

        if list.is_empty() {
            index.push_str(
                "The first public diary entry will appear after a normal 3/3 day is approved or after an authorized DEV RHYTHM catch-up reflection is approved.\n",
            );
        } else {
            let total_pages = (list.len() + POSTS_PER_PAGE - 1) / POSTS_PER_PAGE;
            index.push_str(&format!("**{} POSTS · {} PAGES**\n\n", list.len(), total_pages));

            for page_idx in 0..total_pages {
                let page_number = page_idx + 1;
                let start = page_idx * POSTS_PER_PAGE;
                let end = usize::min(start + POSTS_PER_PAGE, list.len());
                let chunk = &list[start..end];

                let mut text = format!(
                    "# {id} Diary · Page {page_number}\n\n**{name}**\n\n**POST HISTORY · {}–{} OF {}**\n\n",
                    start + 1,
                    end,
                    list.len()
                );

                if page_number > 1 {
                    text.push_str(&format!("[← PAGE {}](page-{}.md)  ", page_number - 1, page_number - 1));
                }
                text.push_str("[ALL PAGES](README.md)");
                if page_number < total_pages {
                    text.push_str(&format!("  [PAGE {} →](page-{}.md)", page_number + 1, page_number + 1));
                }
                text.push_str("\n\n---\n\n");

                for e in chunk.iter().rev() {
                    let date = string(e, "date")?;
                    let label = date_label(date)?;
                    page_for_date.insert((id.clone(), date.to_string()), page_number);
                    text.push_str(&format!(
                        "## {label}\n\n{}\n\n### WHAT I CARRY FORWARD\n\n{}\n\n<sub>Published {} · {}</sub>\n\n---\n\n",
                        string(e, "body")?,
                        string(e, "carry_forward")?,
                        string(e, "published_at")?,
                        completion_note(e),
                    ));
                }

                write(root, &format!("DIARIES/{id}/page-{page_number}.md"), &text)?;
                let latest = if page_number == total_pages { " · LATEST" } else { "" };
                index.push_str(&format!(
                    "- [PAGE {page_number}](page-{page_number}.md){latest}\n"
                ));
                nav.push_str(&format!(
                    "    * [PAGE {page_number}](DIARIES/{id}/page-{page_number}.md)\n"
                ));
            }
        }

        write(root, &format!("DIARIES/{id}/README.md"), &index)?;

        let profile = find_being_page(root, id).ok_or("missing Being profile")?;
        let original = fs::read_to_string(&profile).map_err(|e| e.to_string())?;
        let base = original.split("\n<!-- BEGIN DIARY -->").next().unwrap();
        let mut footer = format!(
            "\n<!-- BEGIN DIARY -->\n## DAILY\n\n[OPEN {id} DIARY](../../../DIARIES/{id}/README.md)\n\n"
        );

        if !list.is_empty() {
            let latest_page = (list.len() + POSTS_PER_PAGE - 1) / POSTS_PER_PAGE;
            footer.push_str("### LATEST 3 POSTS\n\n");
            for e in list.iter().rev().take(PROFILE_PREVIEW_POSTS) {
                let label = date_label(string(e, "date")?)?;
                footer.push_str(&format!(
                    "#### {label}\n\n{}\n\n### WHAT I CARRY FORWARD\n\n{}\n\n<sub>Published {} · {}</sub>\n\n---\n\n",
                    string(e, "body")?,
                    string(e, "carry_forward")?,
                    string(e, "published_at")?,
                    completion_note(e),
                ));
            }
            footer.push_str(&format!(
                "[OPEN {id} DIARY · LATEST PAGE {latest_page}](../../../DIARIES/{id}/page-{latest_page}.md)\n\n"
            ));
        }

        footer.push_str("<!-- END DIARY -->\n");
        fs::write(profile, format!("{base}{footer}")).map_err(|e| e.to_string())?;
    }

    landing.push_str("\n[DEVINES DAILY](../DAILY/README.md)\n");
    write(root, "DIARIES/README.md", &landing)?;

    nav.push_str("* [DEVINES DAILY](DAILY/README.md)\n");
    let mut daily = String::from(
        "# DEVINES DAILY\n\nOne dated page gathers the 34 Being public diary entries in canonical DEVINES order. A date becomes a complete DEVINES day only when every Being has finished all three cycles and published an approved DAILY_REMEMBRANCE. A full 34-Being CATCH_UP_REFLECTION set may be published for recovery, but it never counts as a complete DEVINES day.\n\n",
    );

    let mut complete = 0;
    for (date, list) in days.iter().rev() {
        let label = date_label(date)?;

        if list.len() != roster.len() {
            daily.push_str(&format!(
                "- {label} · open · {}/{} Being public diary entries ready\n",
                list.len(),
                roster.len()
            ));
            continue;
        }

        let normal_complete = list.iter().all(|e| !catchup(e) && e["complete_day"] != false);
        let all_catchup = list.iter().all(|e| catchup(e));
        if normal_complete {
            complete += 1;
        }

        let mut text = if all_catchup {
            format!("# DEVINES DAILY · {label} · CATCH-UP REFLECTIONS\n\n**DEV RHYTHM RECOVERY SET · NOT A COMPLETE 3/3 DEVINES DAY**\n\n")
        } else {
            format!("# DEVINES DAILY · {label}\n\n")
        };

        for (id, name) in &roster {
            let e = list
                .iter()
                .find(|e| e["being_id"] == *id)
                .ok_or("incomplete canonical Being order")?;
            text.push_str(&format!(
                "## {name} · {id}\n\n{}\n\n[OPEN {id} POST HISTORY · PAGE {}](../DIARIES/{id}/page-{}.md)\n\n",
                string(e, "body")?,
                page_for_date
                    .get(&(id.clone(), date.clone()))
                    .ok_or("missing Being history page")?,
                page_for_date
                    .get(&(id.clone(), date.clone()))
                    .ok_or("missing Being history page")?
            ));
        }

        write(root, &format!("DAILY/{date}.md"), &text)?;
        if all_catchup {
            daily.push_str(&format!("- [DEVINES DAILY · {label} · CATCH-UP REFLECTIONS]({date}.md) · not complete 3/3\n"));
        } else {
            daily.push_str(&format!("- [DEVINES DAILY · {label}]({date}.md)\n"));
        }
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
                "event_kind": "DAILY_REMEMBRANCE",
                "verified_cycle_count": 3,
                "expected_cycle_count": 3,
                "complete_day": true,
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
        assert!(root.join("DIARIES/D001/page-1.md").exists());
        assert!(!root.join("DIARIES/D001/2026-09-29.md").exists());
        assert!(!root.join("DIARIES/D001/2026-09-29/cycle-1.md").exists());

        for day in [
            "2026-09-30", "2026-10-01", "2026-10-02", "2026-10-03", "2026-10-04",
            "2026-10-05", "2026-10-06", "2026-10-07", "2026-10-08", "2026-10-09",
            "2026-10-10", "2026-10-11",
        ] {
            events.push(daily("D001", day));
        }
        save(&events);
        render(&root).unwrap();
        let d001 = fs::read_to_string(root.join("DIARIES/D001/README.md")).unwrap();
        assert!(d001.contains("[PAGE 1](page-1.md)"));
        assert!(d001.contains("[PAGE 2](page-2.md) · LATEST"));
        let page1 = fs::read_to_string(root.join("DIARIES/D001/page-1.md")).unwrap();
        let page2 = fs::read_to_string(root.join("DIARIES/D001/page-2.md")).unwrap();
        assert!(page1.contains("29/09/26"));
        assert!(page1.contains("10/10/26"));
        assert!(page2.contains("11/10/26"));

        let profile = fs::read_to_string(root.join("BOOKS/BOOK-II-BEINGS/test/D001.md")).unwrap();
        assert!(profile.contains("LATEST 3 POSTS"));
        assert!(profile.contains("11/10/26"));
        assert!(profile.contains("10/10/26"));
        assert!(profile.contains("09/10/26"));
        assert!(!profile.contains("08/10/26"));
        assert!(profile.contains("OPEN D001 DIARY · LATEST PAGE 2"));

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

        let catchup = serde_json::json!({
            "source_events":["a","b"],
            "event_kind":"CATCH_UP_REFLECTION",
            "verified_cycle_count":2,
            "expected_cycle_count":3,
            "complete_day":false
        });
        assert_eq!(source_events(&catchup).unwrap().len(), 2);
    }
}
