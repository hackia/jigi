// helpers.rs
use rocket_dyn_templates::tera;
use serde_json::{Value, to_value};
use std::collections::HashMap;

pub fn num_format(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let n = args.get("n").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let s = format!("{:>.*}", 0, n); // entier
    let with_spaces = s
        .chars()
        .rev()
        .enumerate()
        .map(|(i, c)| {
            if i != 0 && i % 3 == 0 {
                format!(" {c}")
            } else {
                c.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("")
        .chars()
        .rev()
        .collect::<String>();
    Ok(Value::from(with_spaces))
}

pub fn money_eur(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let n = args.get("n").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let s = format!("{:.2}", n);
    let (intp, decp) = s.split_once('.').unwrap_or((s.as_str(), "00"));
    let spaced = intp
        .chars()
        .rev()
        .enumerate()
        .map(|(i, c)| {
            if i != 0 && i % 3 == 0 {
                format!(" {}", c)
            } else {
                c.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("")
        .chars()
        .rev()
        .collect::<String>();
    Ok(to_value(format!("{spaced},{decp} €")).unwrap())
}

pub fn percent(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let n = args.get("n").and_then(|v| v.as_f64()).unwrap_or(0.0);
    Ok(to_value(format!("{:.0}%", n * 100.0)).unwrap())
}

pub fn safe_url(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let url = args.get("url").and_then(|v| v.as_str()).unwrap_or("");
    let ok = url.starts_with('/') || url.starts_with("https://") || url.starts_with("http://");
    Ok(to_value(if ok { url } else { "#" }).unwrap())
}

pub fn external_rel(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let url = args.get("url").and_then(|v| v.as_str()).unwrap_or("");
    let is_ext = url.starts_with("http");
    Ok(to_value(if is_ext {
        "noopener noreferrer external"
    } else {
        ""
    })
    .unwrap())
}

pub fn csrf_input(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let token = args.get("token").and_then(|v| v.as_str()).unwrap_or("");
    Ok(to_value(format!(
        r#"<input type="hidden" name="csrf" value="{token}">"#
    ))
    .unwrap())
}

pub fn active_link(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let current = args.get("current").and_then(|v| v.as_str()).unwrap_or("");
    let href = args.get("href").and_then(|v| v.as_str()).unwrap_or("");
    let class = if current == href {
        "aria-current=\"page\""
    } else {
        ""
    };
    Ok(to_value(class).unwrap())
}

pub fn paginate(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let total = args.get("total").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let page = args.get("page").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
    let per = args.get("per").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
    let pages = total.div_ceil(per);
    let has_prev = page > 1;
    let has_next = page < pages.max(1);
    Ok(to_value(
        serde_json::json!({"pages":pages,"page":page,"per":per,"prev":has_prev,"next":has_next}),
    )
    .unwrap())
}
