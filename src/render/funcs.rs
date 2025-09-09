use once_cell::sync::Lazy;
use rocket_dyn_templates::tera;
use std::{collections::HashMap, fs};

static MANIFEST: Lazy<HashMap<String, String>> = Lazy::new(|| {
    fs::read_to_string("public/public/manifest.json")
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
});

pub fn asset(args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let mapped = MANIFEST
        .get(path)
        .cloned()
        .unwrap_or_else(|| path.to_string());
    Ok(tera::to_value(format!("/public/{}", mapped)).unwrap())
}

pub fn asset_tag(args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
    let kind = args.get("kind").and_then(|v| v.as_str()).unwrap_or("css");
    let path = asset(args)?.as_str().unwrap().to_string();
    let tag = match kind {
        "css" => format!(r#"<link rel="stylesheet" href="{}">"#, path),
        "js" => format!(r#"<script type="module" src="{}"></script>"#, path),
        _ => "".into(),
    };
    Ok(tera::to_value(tag).unwrap())
}

pub fn img_srcset(args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
    // ex: path="img/cover.jpg", widths=[480,768,1200]
    let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
    let widths: Vec<i64> = args
        .get("widths")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or(vec![480, 768, 1200]);
    let srcset = widths
        .into_iter()
        .map(|w| {
            format!(
                "/public/{} {}w",
                path.replace(
                    '.',
                    &format!(".{}.{}", w, path.split('.').next_back().unwrap_or("jpg"))
                ),
                w
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    Ok(tera::to_value(srcset).unwrap())
}

pub fn picture(_args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
    // à implémenter si tu veux du <picture> clé en main
    Ok(tera::to_value("").unwrap())
}

pub fn canonical(args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
    let base = std::env::var("APP_PUBLIC_URL").unwrap_or_else(|_| "http://localhost:8000".into());
    let p = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
    Ok(tera::to_value(format!("{base}{p}")).unwrap())
}
