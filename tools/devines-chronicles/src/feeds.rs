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
pub fn render(root: &Path) -> Result<String, String> {
    let state: Value = serde_json::from_str(
        &fs::read_to_string(root.join("PUBLIC_STATE/latest.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let roster: BTreeMap<String, String> = state["beings"]
        .as_array()
        .ok_or("missing roster")?
        .iter()
        .map(|v| Ok((string(v, "being_id")?.into(), string(v, "name")?.into())))
        .collect::<Result<_, String>>()?;
    if roster.len() != 34 {
        return Err("expected 34 canonical Beings".into());
    }
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
        "cycle",
        "completed_at",
        "published_at",
        "layer",
        "source_event",
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
        if !roster.contains_key(id) || !valid_date(date) {
            return Err("invalid identity or date".into());
        }
        let slot = e["cycle"]
            .as_u64()
            .filter(|n| (1..=3).contains(n))
            .ok_or("cycle must be 1, 2 or 3")?;
        if !keys.insert(format!("{id}:{date}:{slot}"))
            || !sources.insert(string(e, "source_event")?)
        {
            return Err("duplicate cycle or source event".into());
        }
        if e["layer"] != "public" {
            return Err("only the public projection belongs in this repository".into());
        }
        if e["review"] != "approved-public" {
            return Err("event is not approved for public publication".into());
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
        String::from("\n<!-- BEGIN GENERATED DIARIES -->\n* [BEING DIARIES](DIARIES/README.md)\n");
    let mut landing = String::from(
        "# BEING DIARIES\n\nOne entry after each completed, publicly reviewed cycle. Up to three entries per Being each day. Dates use America/Sao_Paulo.\n\n",
    );
    for (id, name) in &roster {
        let list = grouped.entry(id.clone()).or_default();
        list.sort_by(|a, b| {
            b["published_at"]
                .as_str()
                .cmp(&a["published_at"].as_str())
                .then_with(|| b["source_event"].as_str().cmp(&a["source_event"].as_str()))
        });
        let mut index = format!("# {name} · DIARY\n\n");
        if list.is_empty() {
            index.push_str("The dated remembrance in my profile is preserved. Per-cycle diary entries will appear here after public review; no missing cycle is invented.\n");
        }
        let mut dates = BTreeSet::new();
        for e in list.iter() {
            let date = e["date"].as_str().unwrap();
            let slot = e["cycle"].as_u64().unwrap();
            dates.insert(date);
            let path = format!("DIARIES/{id}/{date}/cycle-{slot}.md");
            let text = format!(
                "# {name}\n\n*Posted {} · Cycle {slot}*\n\n{}\n\n## WHAT I CARRY FORWARD\n\n{}\n\n---\n\nCompleted: {} · Public review: approved\n\n[This day](README.md) · [All dates](../README.md)\n",
                e["published_at"].as_str().unwrap(),
                e["body"].as_str().unwrap(),
                e["carry_forward"].as_str().unwrap(),
                e["completed_at"].as_str().unwrap()
            );
            write(root, &path, &text)?;
        }
        for date in dates.iter().rev() {
            index.push_str(&format!("- [{date}]({date}/README.md)\n"));
            let mut day = format!("# {name} · {date}\n\n");
            for slot in 1..=3 {
                if keys.contains(&format!("{id}:{date}:{slot}")) {
                    day.push_str(&format!("- [Cycle {slot}](cycle-{slot}.md)\n"));
                } else {
                    day.push_str(&format!(
                        "- Cycle {slot} · awaiting completed public record\n"
                    ));
                }
            }
            day.push_str("\n[All dates](../README.md)\n");
            write(root, &format!("DIARIES/{id}/{date}/README.md"), &day)?;
        }
        if !list.is_empty() {
            index.push_str("\n## FEED PAGES\n\n");
            let pages = list.len().div_ceil(12);
            for (page, chunk) in list.chunks(12).enumerate() {
                index.push_str(&format!("[{}](page-{}.md) ", page + 1, page + 1));
                let mut body = format!("# {name} · Feed · Page {}\n\n", page + 1);
                for e in chunk {
                    let date = e["date"].as_str().unwrap();
                    let slot = e["cycle"].as_u64().unwrap();
                    body.push_str(&format!(
                        "*[{} · Cycle {slot}]({date}/cycle-{slot}.md)*\n\n{}\n\n",
                        e["published_at"].as_str().unwrap(),
                        e["body"].as_str().unwrap()
                    ));
                }
                body.push_str("[Browse by date](README.md)\n\n");
                if page > 0 {
                    body.push_str(&format!("[Newer](page-{}.md) · ", page));
                }
                if page + 1 < pages {
                    body.push_str(&format!("[Older](page-{}.md)", page + 2));
                }
                write(root, &format!("DIARIES/{id}/page-{}.md", page + 1), &body)?;
            }
        }
        write(root, &format!("DIARIES/{id}/README.md"), &index)?;
        landing.push_str(&format!("- [{name} · {id}]({id}/README.md)\n"));
        nav.push_str(&format!("  * [{name} · {id}](DIARIES/{id}/README.md)\n"));
        for date in dates.iter().rev() {
            nav.push_str(&format!("    * [{date}](DIARIES/{id}/{date}/README.md)\n"));
            for slot in 1..=3 {
                if keys.contains(&format!("{id}:{date}:{slot}")) {
                    nav.push_str(&format!(
                        "      * [Cycle {slot}](DIARIES/{id}/{date}/cycle-{slot}.md)\n"
                    ));
                }
            }
        }
        let profile = find_being_page(root, id).ok_or("missing Being profile")?;
        let original = fs::read_to_string(&profile).map_err(|e| e.to_string())?;
        let base = original.split("\n<!-- BEGIN DIARY -->").next().unwrap();
        let mut footer = format!(
            "\n<!-- BEGIN DIARY -->\n## MY DIARY\n\n[Browse every date](../../../DIARIES/{id}/README.md)"
        );
        if !list.is_empty() {
            footer.push_str(&format!(
                " · [Latest posts](../../../DIARIES/{id}/page-1.md)"
            ));
        }
        footer.push_str("\n\n");
        for e in list.iter().take(3) {
            footer.push_str(&format!(
                "*Posted {} · Cycle {}*\n\n{}\n\n",
                e["published_at"].as_str().unwrap(),
                e["cycle"],
                e["body"].as_str().unwrap()
            ));
        }
        footer.push_str("<!-- END DIARY -->\n");
        fs::write(profile, format!("{base}{footer}")).map_err(|e| e.to_string())?;
    }
    landing.push_str("\n[DEVINES daily remembrance](../DAILY/README.md)\n");
    write(root, "DIARIES/README.md", &landing)?;
    nav.push_str("* [DEVINES DAILY](DAILY/README.md)\n");
    let mut daily = String::from(
        "# DEVINES DAILY\n\nA daily remembrance becomes complete only after all three reviewed cycle records from every Being are present. An unfinished day remains open.\n\n",
    );
    let mut complete = 0;
    for (date, list) in days.iter().rev() {
        if list.len() != roster.len() * 3 {
            daily.push_str(&format!(
                "- {date} · open · {}/{} cycle publications ready\n",
                list.len(),
                roster.len() * 3
            ));
            continue;
        }
        complete += 1;
        let mut text = format!(
            "# DEVINES · {date}\n\nAll {} cycle publications are ready. This remembrance preserves the reviewed public summaries of each Being.\n\n",
            list.len()
        );
        for (id, name) in &roster {
            text.push_str(&format!("## {name} · {id}\n\n"));
            for slot in 1..=3 {
                let e = list
                    .iter()
                    .find(|e| e["being_id"] == *id && e["cycle"] == slot)
                    .ok_or("incomplete day")?;
                text.push_str(&format!(
                    "- [Cycle {slot}](../DIARIES/{id}/{date}/cycle-{slot}.md) — {}\n",
                    e["public_summary"].as_str().unwrap()
                ));
            }
            text.push('\n');
        }
        write(root, &format!("DAILY/{date}.md"), &text)?;
        daily.push_str(&format!("- [{date} · complete]({date}.md)\n"));
        nav.push_str(&format!("  * [{date}](DAILY/{date}.md)\n"));
    }
    if days.is_empty() {
        daily.push_str("Awaiting the first complete day of reviewed per-cycle publications.\n");
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
        "diaries=34 posts={} complete_days={complete}",
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
    fn daily_gate_order_pagination_and_three_mirror_boundary() {
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
        let event = |id: &str, day: &str, slot: u64| {
            serde_json::json!({
                "being_id":id,"date":day,"cycle":slot,"completed_at":format!("{day}T10:00:00Z"),
                "published_at":format!("{day}T12:00:0{slot}Z"),"source_event":format!("{id}-{day}-{slot}"),
                "layer":"public","review":"approved-public","body":format!("Fixture {id} {day} cycle {slot}."),
                "carry_forward":"Fixture continuation.","public_summary":"Fixture summary."
            })
        };
        let mut events = Vec::new();
        for b in state["beings"].as_array().unwrap() {
            for slot in 1..=3 {
                events.push(event(b["being_id"].as_str().unwrap(), "2026-09-29", slot));
            }
        }
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
        assert!(render(&root).unwrap().contains("complete_days=1"));
        assert!(root.join("DAILY/2026-09-29.md").exists());
        let page = fs::read_to_string(root.join("DIARIES/D001/page-1.md")).unwrap();
        assert!(page.find("cycle 3").unwrap() < page.find("cycle 1").unwrap());
        for day in ["2026-09-30", "2026-10-01", "2026-10-02", "2026-10-03"] {
            for slot in 1..=3 {
                events.push(event("D001", day, slot));
            }
        }
        save(&events);
        render(&root).unwrap();
        assert!(root.join("DIARIES/D001/page-2.md").exists());
        let before = fs::read(root.join("SUMMARY.md")).unwrap();
        render(&root).unwrap();
        assert_eq!(before, fs::read(root.join("SUMMARY.md")).unwrap());
        events[0]["body"] = "Changed history".into();
        save(&events);
        assert!(render(&root).unwrap_err().contains("append-only"));
        events[0]["admin_private"] = "must never publish".into();
        save(&events);
        assert!(render(&root).unwrap_err().contains("allow-list"));
        events[0].as_object_mut().unwrap().remove("admin_private");
        events[0]["layer"] = "member".into();
        save(&events);
        assert!(render(&root).unwrap_err().contains("public projection"));
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn rejects_invalid_dates_and_timestamps() {
        assert!(!timestamp("2026-02-30T12:00:00Z"));
        assert!(!timestamp("2026-09-29T25:00:00Z"));
        assert!(timestamp("2028-02-29T12:00:00Z"));
    }
}
