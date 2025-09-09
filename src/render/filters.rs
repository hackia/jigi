use chrono::{DateTime, Utc};
use comrak::{ComrakOptions, markdown_to_html};
use rocket_dyn_templates::tera;
use rocket_dyn_templates::tera::Result as TeraResult;
use rocket_dyn_templates::tera::Value;
use rocket_dyn_templates::tera::to_value;

pub fn markdown(value: &Value, _: &std::collections::HashMap<String, Value>) -> TeraResult<Value> {
    let s = value.as_str().unwrap_or_default();
    let mut opt = ComrakOptions::default();
    opt.extension.table = true;
    opt.extension.strikethrough = true;
    let html = markdown_to_html(s, &opt);
    Ok(to_value(html).unwrap())
}

pub fn md_excerpt(
    value: &Value,
    args: &std::collections::HashMap<String, Value>,
) -> TeraResult<Value> {
    let words = args.get("words").and_then(|v| v.as_u64()).unwrap_or(40) as usize;
    let s = value.as_str().unwrap_or_default();
    let text = s
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .to_string();
    let out = text
        .split_whitespace()
        .take(words)
        .collect::<Vec<_>>()
        .join(" ");
    Ok(to_value(out).unwrap())
}

pub fn date(value: &Value, args: &std::collections::HashMap<String, Value>) -> TeraResult<Value> {
    let fmt = args
        .get("fmt")
        .and_then(|v| v.as_str())
        .unwrap_or("%Y-%m-%d");
    let s = value.as_str().unwrap_or_default();
    let dt: DateTime<Utc> = s.parse().unwrap_or_else(|_| Utc::now());
    Ok(to_value(dt.format(fmt).to_string()).unwrap())
}

pub fn ago(value: &Value, _args: &std::collections::HashMap<String, Value>) -> TeraResult<Value> {
    let s = value.as_str().unwrap_or_default();
    let then: DateTime<Utc> = s.parse().unwrap_or_else(|_| Utc::now());
    let sec = (Utc::now() - then).num_seconds();
    let human = if sec < 60 {
        format!("il y a {}s", sec)
    } else if sec < 3600 {
        format!("il y a {}min", sec / 60)
    } else if sec < 86400 {
        format!("il y a {}h", sec / 3600)
    } else {
        format!("il y a {}j", sec / 86400)
    };
    Ok(to_value(human).unwrap())
}

pub fn truncate(
    value: &Value,
    args: &std::collections::HashMap<String, Value>,
) -> TeraResult<Value> {
    let n = args.get("n").and_then(|v| v.as_u64()).unwrap_or(160) as usize;
    let s = value.as_str().unwrap_or_default();
    let out = if s.len() > n {
        format!("{}…", &s[..n])
    } else {
        s.to_string()
    };
    Ok(to_value(out).unwrap())
}

pub fn slugify(
    value: &Value,
    _args: &std::collections::HashMap<String, Value>,
) -> TeraResult<Value> {
    let s = value.as_str().unwrap_or_default();
    Ok(tera::to_value(slug::slugify(s)).unwrap())
}

pub fn nl2br(value: &Value, _args: &std::collections::HashMap<String, Value>) -> TeraResult<Value> {
    let s = value.as_str().unwrap_or_default().replace('\n', "<br/>");
    Ok(to_value(s).unwrap())
}

pub fn json_pp(
    value: &Value,
    _args: &std::collections::HashMap<String, Value>,
) -> TeraResult<Value> {
    Ok(Value::String(serde_json::to_string_pretty(value).unwrap()))
}
