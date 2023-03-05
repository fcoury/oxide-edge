use bson::doc;
use serde_json::json;

use super::CommandResult;

pub fn run() -> CommandResult {
    let banner = r#"
 ▄▄█▀▀██            ██      ▀██          ▀██▀▀█▄   ▀██▀▀█▄
▄█▀    ██  ▄▄▄ ▄▄▄ ▄▄▄    ▄▄ ██    ▄▄▄▄   ██   ██   ██   ██
██      ██  ▀█▄▄▀   ██  ▄▀  ▀██  ▄█▄▄▄██  ██    ██  ██▀▀▀█▄
▀█▄     ██   ▄█▄    ██  █▄   ██  ██       ██    ██  ██    ██
 ▀▀█▄▄▄█▀  ▄█  ██▄ ▄██▄ ▀█▄▄▀██▄  ▀█▄▄▄▀ ▄██▄▄▄█▀  ▄██▄▄▄█▀

OxideDB v0.1.0      by Felipe Coury <felipe.coury@gmail.com>
"#;

    let log = banner
        .lines()
        .into_iter()
        .map(|line| {
            let json_line = json!({
                "t": { "$date": format!("{}", chrono::Utc::now().to_rfc3339()) },
                "s": "I",
                "c": "STORAGE",
                "id": 22297,
                "ctx": "initandlisten",
                "msg": line,
                "tags": ["startupWarnings"]
            });
            serde_json::to_string(&json_line).unwrap()
        })
        .collect::<Vec<String>>();

    let doc = doc! {
        "totalLinesWritten": 3,
        "log": log,
        "ok": 1.0
    };

    Ok(doc)
}
