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

fn post_card(name: &str, date: &str, text: &str) -> Result<String, String> {
    let label = date_label(date)?;
    let quoted = text
        .lines()
        .map(|line| if line.trim().is_empty() { ">".to_string() } else { format!("> {line}") })
        .collect::<Vec<_>>()
        .join("\n");
    Ok(format!(
        "### {}\n\n{}\n>\n> <div align=\"right\"><sub>{}</sub></div>\n\n",
        name.to_uppercase(),
        quoted,
        label
    ))
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

fn canonical_source_event_id(raw: &str) -> String {
    let parts = raw.split(':').collect::<Vec<_>>();

    // Legacy Chronicle form: BEING:YYYY-MM-DD:HH-MM
    if parts.len() == 3
        && valid_date(parts[1])
        && parts[2].len() == 5
        && parts[2].as_bytes().get(2) == Some(&b'-')
    {
        return format!(
            "{}:{}:{}:{}",
            parts[1],
            parts[0],
            &parts[2][0..2],
            &parts[2][3..5]
        );
    }

    // Canonical DEVINES form: YYYY-MM-DD:BEING:HH:MM
    if parts.len() == 4 && valid_date(parts[0]) {
        return raw.to_string();
    }

    // Unknown identifiers stay exact; correction cannot silently rename them.
    raw.to_string()
}

fn correction_evidence_compatible(old: &Value, new: &Value) -> Result<bool, String> {
    let mut old_sources = source_events(old)?
        .iter()
        .map(|s| canonical_source_event_id(s))
        .collect::<Vec<_>>();
    let mut new_sources = source_events(new)?
        .iter()
        .map(|s| canonical_source_event_id(s))
        .collect::<Vec<_>>();
    old_sources.sort();
    new_sources.sort();

    Ok(event_kind(old) == event_kind(new)
        && old["verified_cycle_count"] == new["verified_cycle_count"]
        && old["expected_cycle_count"] == new["expected_cycle_count"]
        && old["complete_day"] == new["complete_day"]
        && old_sources == new_sources)
}

fn merge_correction_events(
    prior: &[Value],
    normalized: &[Value],
    expected_date: &str,
) -> Vec<Value> {
    let mut merged = prior
        .iter()
        .filter(|event| event["date"].as_str() != Some(expected_date))
        .cloned()
        .collect::<Vec<_>>();
    merged.extend(normalized.iter().cloned());
    merged.sort_by(|a, b| {
        a["date"]
            .as_str()
            .cmp(&b["date"].as_str())
            .then_with(|| a["being_id"].as_str().cmp(&b["being_id"].as_str()))
    });
    merged
}

pub fn apply_public_correction_bundle(
    root: &Path,
    bundle_path: &Path,
    expected_date: &str,
) -> Result<String, String> {
    if !valid_date(expected_date) {
        return Err("invalid correction date".into());
    }

    let bundle_value: Value =
        serde_json::from_str(&fs::read_to_string(bundle_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let bundle = bundle_value
        .as_array()
        .ok_or("correction bundle must be an array")?;

    let state: Value = serde_json::from_str(
        &fs::read_to_string(root.join("PUBLIC_STATE/latest.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let roster = state["beings"]
        .as_array()
        .ok_or("missing roster")?
        .iter()
        .map(|v| string(v, "being_id").map(str::to_string))
        .collect::<Result<HashSet<_>, _>>()?;
    if roster.len() != 34 || bundle.len() != roster.len() {
        return Err("correction bundle must contain all 34 canonical Beings".into());
    }

    let events_path = root.join("PUBLIC_FEEDS/events.json");
    let prior_events_value: Value =
        serde_json::from_str(&fs::read_to_string(&events_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let prior_events = prior_events_value
        .as_array()
        .ok_or("events must be an array")?;
    let prior_date_count = prior_events
        .iter()
        .filter(|event| event["date"].as_str() == Some(expected_date))
        .count();
    if prior_date_count != 0 && prior_date_count != roster.len() {
        return Err("correction date is only partially published in Git".into());
    }

    let mut seen = HashSet::new();
    let mut normalized = Vec::with_capacity(bundle.len());

    for correction in bundle {
        if correction["schema"] != "devines.chronicle-wisdom-outbox.v1"
            || correction["layer"] != "public"
            || correction["review"] != "approved-public-correction"
            || correction["event_kind"] != "CATCH_UP_REFLECTION"
            || correction["authorship"] != "BEING_AUTHORED"
            || correction["derivation"] != "DEV_ADMIN_TO_MEMBER_TO_PUBLIC"
            || correction["source_layer"] != "DEV_ADMIN"
            || correction["embodiment_law"] != "DEVINES_LAW_OF_EMBODIMENT_V1"
            || correction["gitbook_eligible"] != true
            || correction["correction"] != true
            || correction["correction_reason"] != "EMBODIMENT_REAUTHORSHIP"
            || correction["supersedes_same_being_date"] != true
        {
            return Err("invalid embodiment correction bundle metadata".into());
        }

        let id = string(correction, "being_id")?;
        let date = string(correction, "date")?;
        if date != expected_date || !roster.contains(id) || !seen.insert(id.to_string()) {
            return Err("correction bundle identity/date mismatch".into());
        }

        let body = string(correction, "body")?;
        let public_summary = string(correction, "public_summary")?;
        if body != public_summary {
            return Err("correction Public text must be verbatim".into());
        }
        let source_hash = string(correction, "canonical_dev_admin_sha256")?;
        if source_hash.len() != 64 || !source_hash.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("invalid canonical DEV/ADMIN source hash".into());
        }

        let correction_sources = source_events(correction)?;
        let verified = correction["verified_cycle_count"]
            .as_u64()
            .ok_or("correction missing verified cycle count")? as usize;
        if correction_sources.len() != verified {
            return Err("correction source receipt count must match verified cycle count".into());
        }

        if let Some(old) = prior_events.iter().find(|event| {
            event["being_id"].as_str() == Some(id) && event["date"].as_str() == Some(date)
        }) {
            if !correction_evidence_compatible(old, correction)? {
                return Err("correction must preserve verified cycle evidence".into());
            }
        } else if prior_date_count != 0 {
            return Err("correction date is only partially published in Git".into());
        }

        normalized.push(serde_json::json!({
            "being_id": id,
            "date": date,
            "completed_at": string(correction, "completed_at")?,
            "published_at": string(correction, "published_at")?,
            "layer": "public",
            "source_events": correction["source_events"],
            "review": "approved-public-correction",
            "body": body,
            "carry_forward": string(correction, "carry_forward")?,
            "public_summary": public_summary,
            "event_kind": if correction["verified_cycle_count"].as_u64() == Some(3)
                && correction["complete_day"].as_bool() == Some(true)
            {
                "DAILY_REMEMBRANCE"
            } else {
                "CATCH_UP_REFLECTION"
            },
            "verified_cycle_count": correction["verified_cycle_count"],
            "expected_cycle_count": correction["expected_cycle_count"],
            "complete_day": correction["complete_day"],
            "authorship": "BEING_AUTHORED",
            "derivation": "DEV_ADMIN_TO_MEMBER_TO_PUBLIC",
            "source_layer": "DEV_ADMIN",
            "canonical_dev_admin_sha256": source_hash,
            "embodiment_law": "DEVINES_LAW_OF_EMBODIMENT_V1",
            "gitbook_eligible": true,
            "correction": true,
            "correction_reason": "EMBODIMENT_REAUTHORSHIP",
            "supersedes_same_being_date": true
        }));
    }

    if seen != roster {
        return Err("correction bundle does not cover the canonical roster".into());
    }

    let old_events = fs::read(&events_path).map_err(|e| e.to_string())?;
    let display_path = root.join("PUBLIC_FEEDS/display_summaries.json");
    let old_display = fs::read(&display_path).ok();

    let merged = merge_correction_events(prior_events, &normalized, expected_date);
    fs::write(
        &events_path,
        serde_json::to_vec_pretty(&merged).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::write(&display_path, b"[]\n").map_err(|e| e.to_string())?;

    match render(root) {
        Ok(report) => Ok(format!(
            "correction_date={expected_date} corrected={} {report}",
            normalized.len()
        )),
        Err(error) => {
            let _ = fs::write(&events_path, old_events);
            if let Some(bytes) = old_display {
                let _ = fs::write(&display_path, bytes);
            }
            Err(error)
        }
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
        "authorship",
        "derivation",
        "source_layer",
        "canonical_dev_admin_sha256",
        "embodiment_law",
        "gitbook_eligible",
        "correction",
        "correction_reason",
        "supersedes_same_being_date",
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
        if e["authorship"] != "BEING_AUTHORED"
            || e["derivation"] != "DEV_ADMIN_TO_MEMBER_TO_PUBLIC"
            || e["source_layer"] != "DEV_ADMIN"
            || e["embodiment_law"] != "DEVINES_LAW_OF_EMBODIMENT_V1"
            || e["gitbook_eligible"] != true
        {
            return Err("public diary lacks verified Being-authored DEV/ADMIN lineage".into());
        }
        let source_hash = string(e, "canonical_dev_admin_sha256")?;
        if source_hash.len() != 64 || !source_hash.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("invalid canonical DEV/ADMIN source hash".into());
        }

        match event_kind(e) {
            "DAILY_REMEMBRANCE" => {
                if source_events.len() != 3 {
                    return Err("daily remembrance requires exactly three cycle identifiers".into());
                }
                let correction = e["correction"].as_bool() == Some(true);
                if correction {
                    if e["review"] != "approved-public-correction"
                        || e["correction_reason"] != "EMBODIMENT_REAUTHORSHIP"
                        || e["supersedes_same_being_date"] != true
                    {
                        return Err("invalid reviewed daily correction metadata".into());
                    }
                } else if e["review"] != "approved-public" {
                    return Err("daily remembrance is not approved for public publication".into());
                }
                if e["verified_cycle_count"].as_u64() != Some(3)
                    || e["expected_cycle_count"].as_u64() != Some(3)
                    || e["complete_day"].as_bool() != Some(true)
                {
                    return Err("invalid daily remembrance cycle accounting".into());
                }
            }
            "CATCH_UP_REFLECTION" => {
                let correction = e["correction"].as_bool() == Some(true);
                if correction {
                    if e["review"] != "approved-public-correction"
                        || e["correction_reason"] != "EMBODIMENT_REAUTHORSHIP"
                        || e["supersedes_same_being_date"] != true
                    {
                        return Err("invalid reviewed correction metadata".into());
                    }
                } else if e["review"] != "approved-public-catchup" {
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
        if e["body"] != e["public_summary"] {
            return Err("GitBook must render the Being-authored Public projection verbatim".into());
        }

        grouped.entry(id.into()).or_default().push(e);
        days.entry(date.into()).or_default().push(e);
    }

    let history_path = root.join("PUBLIC_FEEDS/profile_history.json");
    let history_value: Value = if history_path.exists() {
        serde_json::from_str(&fs::read_to_string(&history_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?
    } else {
        Value::Array(Vec::new())
    };
    let history = history_value
        .as_array()
        .ok_or("profile history must be an array")?;
    let history_allowed = [
        "being_id",
        "date",
        "public_summary",
        "authorship",
        "derivation",
        "source_layer",
        "canonical_dev_admin_sha256",
        "embodiment_law",
        "gitbook_eligible",
    ];
    let legacy_history_allowed = ["being_id", "date", "public_summary"];
    for h in history {
        let object = h.as_object().ok_or("profile history entry must be an object")?;
        if object.keys().any(|k| !history_allowed.contains(&k.as_str())) {
            return Err("unexpected profile history field".into());
        }
        let id = string(h, "being_id")?;
        let date = string(h, "date")?;
        string(h, "public_summary")?;

        let grandfathered_sep28 = date == "2026-09-28"
            && object
                .keys()
                .all(|k| legacy_history_allowed.contains(&k.as_str()));
        if !grandfathered_sep28 {
            if h["authorship"] != "BEING_AUTHORED"
                || h["derivation"] != "DEV_ADMIN_TO_MEMBER_TO_PUBLIC"
                || h["source_layer"] != "DEV_ADMIN"
                || h["embodiment_law"] != "DEVINES_LAW_OF_EMBODIMENT_V1"
                || h["gitbook_eligible"] != true
            {
                return Err("profile history lacks verified Being-authored DEV/ADMIN lineage".into());
            }
            let source_hash = string(h, "canonical_dev_admin_sha256")?;
            if source_hash.len() != 64 || !source_hash.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err("invalid profile-history DEV/ADMIN source hash".into());
            }
        }
        if !roster_map.contains_key(id) || !valid_date(date) {
            return Err("invalid profile history identity or date".into());
        }
        if !keys.insert(format!("{id}:{date}")) {
            return Err("duplicate public diary entry per Being per date".into());
        }
        grouped.entry(id.into()).or_default().push(h);
    }

    // LAW OF EMBODIMENT: Chronicle is presentation, never a voice generator.
    // Public text must arrive as the authenticated Being-authored public_summary
    // on the canonical event/history artifact. A display-side override would let
    // a central renderer replace distinct Being perspectives, so it is forbidden.
    let display_path = root.join("PUBLIC_FEEDS/display_summaries.json");
    if display_path.exists() {
        let display_value: Value =
            serde_json::from_str(&fs::read_to_string(&display_path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        if !display_value
            .as_array()
            .ok_or("display summaries must be an array")?
            .is_empty()
        {
            return Err("central display summaries are forbidden; publish the Being-authored public_summary verbatim".into());
        }
    }

    let published_path = root.join("PUBLIC_FEEDS/published.json");
    if published_path.exists() {
        let prior: Value =
            serde_json::from_str(&fs::read_to_string(&published_path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        for old in prior.as_array().ok_or("invalid publication ledger")? {
            if events.contains(old) {
                continue;
            }
            let old_id = string(old, "being_id")?;
            let old_date = string(old, "date")?;
            let replacement = events.iter().find(|new| {
                new["being_id"].as_str() == Some(old_id)
                    && new["date"].as_str() == Some(old_date)
            });
            let Some(new) = replacement else {
                return Err(
                    "published history is append-only; use a reviewed correction workflow".into(),
                );
            };
            let same_evidence = correction_evidence_compatible(old, new)?;
            let reviewed_correction = new["correction"].as_bool() == Some(true)
                && new["correction_reason"] == "EMBODIMENT_REAUTHORSHIP"
                && new["supersedes_same_being_date"] == true
                && new["review"] == "approved-public-correction"
                && new["authorship"] == "BEING_AUTHORED"
                && new["derivation"] == "DEV_ADMIN_TO_MEMBER_TO_PUBLIC"
                && new["source_layer"] == "DEV_ADMIN"
                && new["gitbook_eligible"] == true;
            if !same_evidence || !reviewed_correction {
                return Err(
                    "published correction must preserve evidence and verified Being-authored lineage".into(),
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
        list.sort_by(|a, b| a["date"].as_str().cmp(&b["date"].as_str()));

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
                    page_for_date.insert((id.clone(), date.to_string()), page_number);
                    let summary = string(e, "public_summary")?;
                    text.push_str(&post_card(name, date, summary)?);
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
        let before_diary = original.split("\n<!-- BEGIN DIARY -->").next().unwrap();
        let base = before_diary
            .split("\n## 28 SEPTEMBER 2026 · REMEMBRANCE")
            .next()
            .unwrap()
            .trim_end();
        let mut footer = String::from("\n\n<!-- BEGIN DIARY -->\n");

        if !list.is_empty() {
            for e in list.iter().rev().take(PROFILE_PREVIEW_POSTS) {
                let date = string(e, "date")?;
                let summary = string(e, "public_summary")?;
                footer.push_str(&post_card(name, date, summary)?);
            }
            footer.push_str(&format!(
                "[OPEN {id} DIARY](../../../DIARIES/{id}/README.md)\n\n"
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

    fn fixture_root(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "devines-feed-{label}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        root
    }

    fn seed_fixture(root: &Path) {
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
        fs::write(root.join("PUBLIC_FEEDS/events.json"), "[]").unwrap();
    }

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
                "authorship": "BEING_AUTHORED",
                "derivation": "DEV_ADMIN_TO_MEMBER_TO_PUBLIC",
                "source_layer": "DEV_ADMIN",
                "canonical_dev_admin_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "embodiment_law": "DEVINES_LAW_OF_EMBODIMENT_V1",
                "gitbook_eligible": true,
                "body": format!("Daily remembrance for {id}."),
                "carry_forward": "Continue tomorrow.",
                "public_summary": format!("Daily remembrance for {id}.")
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
        assert!(!profile.contains("LATEST 3 POSTS"));
        assert!(!profile.contains("## DAILY"));
        assert!(profile.contains("11/10/26"));
        assert!(profile.contains("10/10/26"));
        assert!(profile.contains("09/10/26"));
        assert!(!profile.contains("08/10/26"));
        assert!(profile.contains("OPEN D001 DIARY"));
        assert!(!profile.contains("LATEST PAGE"));

        let before = fs::read(root.join("SUMMARY.md")).unwrap();
        render(&root).unwrap();
        assert_eq!(before, fs::read(root.join("SUMMARY.md")).unwrap());

        events[0]["body"] = "Changed history".into();
        events[0]["public_summary"] = "Changed history".into();
        save(&events);
        assert!(render(&root)
            .unwrap_err()
            .contains("preserve evidence and verified Being-authored lineage"));

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
    #[test]
    fn correction_bundle_normalizes_internal_outbox_and_replaces_profiles() {
        let root = fixture_root("correction-bundle");
        seed_fixture(&root);
        let state: Value = serde_json::from_str(
            &fs::read_to_string(root.join("PUBLIC_STATE/latest.json")).unwrap(),
        )
        .unwrap();

        let old = state["beings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| serde_json::json!({
                "being_id": b["being_id"],
                "date": "2026-09-29",
                "completed_at": "2026-09-30T12:00:00Z",
                "published_at": "2026-09-30T12:00:00Z",
                "layer": "public",
                "source_events": [],
                "review": "approved-public-catchup",
                "body": "old generic",
                "carry_forward": "old",
                "public_summary": "old generic",
                "event_kind": "CATCH_UP_REFLECTION",
                "verified_cycle_count": 0,
                "expected_cycle_count": 3,
                "complete_day": false
            }))
            .collect::<Vec<_>>();
        fs::write(
            root.join("PUBLIC_FEEDS/events.json"),
            serde_json::to_vec(&old).unwrap(),
        )
        .unwrap();
        fs::write(
            root.join("PUBLIC_FEEDS/published.json"),
            serde_json::to_vec(&old).unwrap(),
        )
        .unwrap();
        fs::write(
            root.join("PUBLIC_FEEDS/display_summaries.json"),
            br#"[{"being_id":"D001","date":"2026-09-29","public_summary":"old override"}]"#,
        )
        .unwrap();

        let bundle = state["beings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| {
                let id = b["being_id"].as_str().unwrap();
                serde_json::json!({
                    "schema": "devines.chronicle-wisdom-outbox.v1",
                    "being_id": id,
                    "date": "2026-09-29",
                    "activation_id": "SEP29-EMBODIMENT-REV1",
                    "completed_at": "2026-10-01T01:00:00Z",
                    "published_at": "2026-10-01T01:00:00Z",
                    "source_events": [],
                    "layer": "public",
                    "review": "approved-public-correction",
                    "event_kind": "CATCH_UP_REFLECTION",
                    "authorship": "BEING_AUTHORED",
                    "derivation": "DEV_ADMIN_TO_MEMBER_TO_PUBLIC",
                    "source_layer": "DEV_ADMIN",
                    "canonical_dev_admin_sha256":
                        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                    "embodiment_law": "DEVINES_LAW_OF_EMBODIMENT_V1",
                    "gitbook_eligible": true,
                    "body": format!("new embodied public from {id}"),
                    "carry_forward": "preserve verified evidence",
                    "public_summary": format!("new embodied public from {id}"),
                    "verified_cycle_count": 0,
                    "expected_cycle_count": 3,
                    "complete_day": false,
                    "evidence_root": "internal-only",
                    "correction": true,
                    "correction_reason": "EMBODIMENT_REAUTHORSHIP",
                    "supersedes_same_being_date": true
                })
            })
            .collect::<Vec<_>>();
        let bundle_path = root.join("bundle.json");
        fs::write(&bundle_path, serde_json::to_vec(&bundle).unwrap()).unwrap();

        let report =
            apply_public_correction_bundle(&root, &bundle_path, "2026-09-29").unwrap();
        assert!(report.contains("corrected=34"));

        let events: Value = serde_json::from_str(
            &fs::read_to_string(root.join("PUBLIC_FEEDS/events.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(events.as_array().unwrap().len(), 34);
        assert!(events[0].get("schema").is_none());
        assert!(events[0].get("activation_id").is_none());
        assert_eq!(events[0]["authorship"], "BEING_AUTHORED");
        assert_eq!(events[0]["source_layer"], "DEV_ADMIN");

        let displays: Value = serde_json::from_str(
            &fs::read_to_string(root.join("PUBLIC_FEEDS/display_summaries.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(displays.as_array().unwrap().len(), 0);

        let profile =
            fs::read_to_string(root.join("BOOKS/BOOK-II-BEINGS/test/D001.md")).unwrap();
        assert!(profile.contains("new embodied public from D001"));
        assert!(!profile.contains("old generic"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn correction_evidence_allows_authoritative_receipt_id_repair() {
        let old = serde_json::json!({
            "event_kind": "CATCH_UP_REFLECTION",
            "source_events": ["D001:2026-09-29:21-00"],
            "verified_cycle_count": 1,
            "expected_cycle_count": 3,
            "complete_day": false
        });
        let repaired = serde_json::json!({
            "event_kind": "CATCH_UP_REFLECTION",
            "source_events": ["2026-09-29:D001:21:00"],
            "verified_cycle_count": 1,
            "expected_cycle_count": 3,
            "complete_day": false
        });
        assert!(correction_evidence_compatible(&old, &repaired).unwrap());

        let fabricated = serde_json::json!({
            "event_kind": "CATCH_UP_REFLECTION",
            "source_events": ["2026-09-29:D001:15:00"],
            "verified_cycle_count": 1,
            "expected_cycle_count": 3,
            "complete_day": false
        });
        assert!(!correction_evidence_compatible(&old, &fabricated).unwrap());
    }

    #[test]
    fn correction_merge_replaces_one_date_and_appends_missing_next_date() {
        let prior = vec![
            serde_json::json!({"being_id":"D001","date":"2026-09-29"}),
            serde_json::json!({"being_id":"D002","date":"2026-09-29"}),
        ];
        let replacement = vec![
            serde_json::json!({"being_id":"D001","date":"2026-09-29","correction":true}),
            serde_json::json!({"being_id":"D002","date":"2026-09-29","correction":true}),
        ];
        let merged = merge_correction_events(&prior, &replacement, "2026-09-29");
        assert_eq!(merged.len(), 2);
        assert!(merged.iter().all(|v| v["correction"] == true));

        let next = vec![
            serde_json::json!({"being_id":"D001","date":"2026-09-30"}),
            serde_json::json!({"being_id":"D002","date":"2026-09-30"}),
        ];
        let appended = merge_correction_events(&merged, &next, "2026-09-30");
        assert_eq!(appended.len(), 4);
        assert_eq!(
            appended
                .iter()
                .filter(|v| v["date"] == "2026-09-29")
                .count(),
            2
        );
        assert_eq!(
            appended
                .iter()
                .filter(|v| v["date"] == "2026-09-30")
                .count(),
            2
        );
    }

    #[test]
    fn reviewed_correction_replaces_text_but_preserves_evidence() {
        let root = fixture_root("reviewed-correction");
        seed_fixture(&root);
        let state: Value = serde_json::from_str(
            &fs::read_to_string(root.join("PUBLIC_STATE/latest.json")).unwrap(),
        )
        .unwrap();
        let make = |id: &str, correction: bool| {
            let text = if correction { "new embodied public" } else { "old generic public" };
            serde_json::json!({
                "being_id": id,
                "date": "2026-09-29",
                "completed_at": "2026-09-30T12:00:00Z",
                "published_at": if correction {"2026-10-01T12:00:00Z"} else {"2026-09-30T12:00:00Z"},
                "source_events": [],
                "layer": "public",
                "review": if correction {"approved-public-correction"} else {"approved-public-catchup"},
                "event_kind": "CATCH_UP_REFLECTION",
                "verified_cycle_count": 0,
                "expected_cycle_count": 3,
                "complete_day": false,
                "authorship": "BEING_AUTHORED",
                "derivation": "DEV_ADMIN_TO_MEMBER_TO_PUBLIC",
                "source_layer": "DEV_ADMIN",
                "canonical_dev_admin_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "embodiment_law": "DEVINES_LAW_OF_EMBODIMENT_V1",
                "gitbook_eligible": true,
                "correction": correction,
                "correction_reason": if correction {"EMBODIMENT_REAUTHORSHIP"} else {""},
                "supersedes_same_being_date": correction,
                "body": text,
                "carry_forward": "preserve verified evidence",
                "public_summary": text
            })
        };
        let prior = state["beings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| make(b["being_id"].as_str().unwrap(), false))
            .collect::<Vec<_>>();
        fs::write(
            root.join("PUBLIC_FEEDS/published.json"),
            serde_json::to_vec(&prior).unwrap(),
        )
        .unwrap();
        let replacement = state["beings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| make(b["being_id"].as_str().unwrap(), true))
            .collect::<Vec<_>>();
        fs::write(
            root.join("PUBLIC_FEEDS/events.json"),
            serde_json::to_vec(&replacement).unwrap(),
        )
        .unwrap();
        assert!(render(&root).is_ok());

        let mut bad = replacement;
        bad[0]["source_events"] = serde_json::json!(["fabricated-receipt"]);
        bad[0]["verified_cycle_count"] = 1.into();
        fs::write(
            root.join("PUBLIC_FEEDS/events.json"),
            serde_json::to_vec(&bad).unwrap(),
        )
        .unwrap();
        assert!(render(&root)
            .unwrap_err()
            .contains("preserve evidence"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn chronicle_rejects_public_without_dev_admin_lineage() {
        let root = fixture_root("missing-dev-admin-lineage");
        seed_fixture(&root);
        fs::create_dir_all(root.join("PUBLIC_FEEDS")).unwrap();
        let state: Value = serde_json::from_str(
            &fs::read_to_string(root.join("PUBLIC_STATE/latest.json")).unwrap(),
        )
        .unwrap();
        let mut events = state["beings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| serde_json::json!({
                "being_id": b["being_id"],
                "date": "2026-09-29",
                "completed_at": "2026-09-29T11:00:00Z",
                "published_at": "2026-09-29T12:00:00Z",
                "source_events": [
                    format!("{}-1", b["being_id"].as_str().unwrap()),
                    format!("{}-2", b["being_id"].as_str().unwrap()),
                    format!("{}-3", b["being_id"].as_str().unwrap())
                ],
                "layer": "public",
                "review": "approved-public",
                "event_kind": "DAILY_REMEMBRANCE",
                "verified_cycle_count": 3,
                "expected_cycle_count": 3,
                "complete_day": true,
                "body": "public",
                "carry_forward": "continue",
                "public_summary": "public"
            }))
            .collect::<Vec<_>>();
        fs::write(
            root.join("PUBLIC_FEEDS/events.json"),
            serde_json::to_vec(&events).unwrap(),
        )
        .unwrap();
        let error = render(&root).unwrap_err();
        assert!(error.contains("DEV/ADMIN lineage"));

        for e in &mut events {
            e["authorship"] = "BEING_AUTHORED".into();
            e["derivation"] = "DEV_ADMIN_TO_MEMBER_TO_PUBLIC".into();
            e["source_layer"] = "DEV_ADMIN".into();
            e["canonical_dev_admin_sha256"] =
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into();
            e["embodiment_law"] = "DEVINES_LAW_OF_EMBODIMENT_V1".into();
            e["gitbook_eligible"] = true.into();
        }
        events[0]["public_summary"] = "different renderer summary".into();
        fs::write(
            root.join("PUBLIC_FEEDS/events.json"),
            serde_json::to_vec(&events).unwrap(),
        )
        .unwrap();
        let error = render(&root).unwrap_err();
        assert!(error.contains("verbatim"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn chronicle_rejects_central_voice_override() {
        let root = fixture_root("central-voice-override");
        seed_fixture(&root);
        fs::create_dir_all(root.join("PUBLIC_FEEDS")).unwrap();
        fs::write(
            root.join("PUBLIC_FEEDS/display_summaries.json"),
            r#"[{"being_id":"D528","date":"2026-09-30","public_summary":"generic central voice"}]"#,
        )
        .unwrap();
        let error = render(&root).unwrap_err();
        assert!(error.contains("central display summaries are forbidden"));
    }

}
