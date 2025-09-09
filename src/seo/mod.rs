use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Seo {
    /// Title (~60 chars)
    pub title: String,
    /// Description (~160 chars)
    pub description: String,
    pub keywords: Vec<String>,
    pub author: Option<String>,

    pub canonical_url: Option<String>,
    pub lang: Option<String>,    // ex: "fr"
    pub updated: Option<String>, // ISO8601

    // Social
    pub og_image: Option<String>,
    pub og_type: Option<String>,      // "website" | "article" | "book"…
    pub twitter_card: Option<String>, // "summary_large_image"

    // (Optionnel) Pour générer du JSON-LD
    pub json_ld: Option<String>,

    // (Optionnel) Jardin
    pub content_type: Option<String>, // "work" | "author" | "season" | "event"
    pub slug: Option<String>,
}

// ---------- Builders (accept Into<String>) ----------
impl Seo {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_title<S: Into<String>>(&mut self, t: S) -> &mut Self {
        self.title = t.into();
        self
    }
    pub fn with_desc<S: Into<String>>(&mut self, d: S) -> &mut Self {
        self.description = d.into();
        self
    }
    pub fn with_keywords<I, S>(&mut self, ks: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.keywords = ks.into_iter().map(|s| s.into()).collect();
        self
    }
    pub fn with_author<S: Into<String>>(&mut self, a: S) -> &mut Self {
        self.author = Some(a.into());
        self
    }
    pub fn with_lang<S: Into<String>>(&mut self, l: S) -> &mut Self {
        self.lang = Some(l.into());
        self
    }
    pub fn with_updated<S: Into<String>>(&mut self, u: S) -> &mut Self {
        self.updated = Some(u.into());
        self
    }
    pub fn with_json_ld<S: Into<String>>(&mut self, j: S) -> &mut Self {
        self.json_ld = Some(j.into());
        self
    }
    pub fn with_content_type<S: Into<String>>(&mut self, c: S) -> &mut Self {
        self.content_type = Some(c.into());
        self
    }
    pub fn with_canonical<S: Into<String>>(&mut self, url: S) -> &mut Self {
        self.canonical_url = Some(url.into());
        self
    }
    pub fn with_slug<S: Into<String>>(&mut self, s: S) -> &mut Self {
        self.slug = Some(s.into());
        self
    }
    pub fn with_og_image<S: Into<String>>(&mut self, img: S) -> &mut Self {
        self.og_image = Some(img.into());
        self
    }
    pub fn with_og_type<S: Into<String>>(&mut self, t: S) -> &mut Self {
        self.og_type = Some(t.into());
        self
    }
    pub fn with_twitter_card<S: Into<String>>(&mut self, c: S) -> &mut Self {
        self.twitter_card = Some(c.into());
        self
    }
    pub fn twitter_summary(&mut self) -> &mut Self {
        self.twitter_card = Some("summary".to_string());
        self
    }

    /// Merge `other` over `self` (other wins when set).
    pub fn merged_with(mut self, other: &Seo) -> Self {
        macro_rules! take_if_some {
            ($field:ident) => {
                if other.$field.is_some() {
                    self.$field = other.$field.clone();
                }
            };
        }
        if !other.title.is_empty() {
            self.title = other.title.clone();
        }
        if !other.description.is_empty() {
            self.description = other.description.clone();
        }
        if !other.keywords.is_empty() {
            self.keywords = other.keywords.clone();
        }
        take_if_some!(author);
        take_if_some!(canonical_url);
        take_if_some!(lang);
        take_if_some!(updated);
        take_if_some!(og_image);
        take_if_some!(og_type);
        take_if_some!(twitter_card);
        take_if_some!(json_ld);
        take_if_some!(content_type);
        take_if_some!(slug);
        self
    }

    /// Provide sane fallbacks (site-wide defaults).
    pub fn with_defaults(mut self, site: &SiteSeoDefaults) -> Self {
        if self.title.is_empty() {
            self.title = site.site_title.clone();
        }
        if self.description.is_empty() {
            self.description = site.site_desc.clone();
        }
        if self.lang.is_none() {
            self.lang = Some(site.lang.clone());
        }
        if self.og_type.is_none() {
            self.og_type = Some("website".into());
        }
        if self.twitter_card.is_none() {
            self.twitter_card = Some("summary_large_image".into());
        }
        self
    }

    /// Convert to a Tera JSON context fragment.
    pub fn to_ctx(&self) -> serde_json::Value {
        serde_json::json!({
            "title": self.title,
            "description": self.description,
            "keywords": self.keywords,
            "author": self.author,
            "canonical_url": self.canonical_url,
            "lang": self.lang,
            "updated": self.updated,
            "og_image": self.og_image,
            "og_type": self.og_type,
            "twitter_card": self.twitter_card,
            "json_ld": self.json_ld,
            "content_type": self.content_type,
            "slug": self.slug
        })
    }

    /// Render a ready-to-inject `<head>` HTML string (useful for non-Tera engines too).
    pub fn render_head(&self, site_name: impl Into<Cow<'static, str>>) -> String {
        let site_name = site_name.into();
        let kw = if self.keywords.is_empty() {
            String::new()
        } else {
            self.keywords.join(", ")
        };
        let mut out = String::new();
        use std::fmt::Write;

        // basics
        let _ = writeln!(out, r#"<meta charset="utf-8">"#);
        if let Some(lang) = &self.lang {
            let _ = writeln!(
                out,
                r#"<meta http-equiv="content-language" content="{lang}">"#
            );
        }
        let _ = writeln!(
            out,
            r#"<meta name="viewport" content="width=device-width, initial-scale=1">"#
        );
        if !self.description.is_empty() {
            let _ = writeln!(
                out,
                r#"<meta name="description" content="{}">"#,
                html_escape(&self.description)
            );
        }
        if !kw.is_empty() {
            let _ = writeln!(
                out,
                r#"<meta name="keywords" content="{}">"#,
                html_escape(&kw)
            );
        }
        if let Some(a) = &self.author {
            let _ = writeln!(out, r#"<meta name="author" content="{}">"#, html_escape(a));
        }
        if let Some(c) = &self.canonical_url {
            let _ = writeln!(out, r#"<link rel="canonical" href="{}">"#, html_escape(c));
        }

        // Open Graph
        let _ = writeln!(
            out,
            r#"<meta property="og:title" content="{}">"#,
            html_escape(&self.title)
        );
        if !self.description.is_empty() {
            let _ = writeln!(
                out,
                r#"<meta property="og:description" content="{}">"#,
                html_escape(&self.description)
            );
        }
        if let Some(t) = &self.og_type {
            let _ = writeln!(
                out,
                r#"<meta property="og:type" content="{}">"#,
                html_escape(t)
            );
        }
        if let Some(img) = &self.og_image {
            let _ = writeln!(
                out,
                r#"<meta property="og:image" content="{}">"#,
                html_escape(img)
            );
        }
        let _ = writeln!(
            out,
            r#"<meta property="og:site_name" content="{}">"#,
            html_escape(&site_name)
        );

        // Twitter
        if let Some(card) = &self.twitter_card {
            let _ = writeln!(
                out,
                r#"<meta name="twitter:card" content="{}">"#,
                html_escape(card)
            );
        }
        let _ = writeln!(
            out,
            r#"<meta name="twitter:title" content="{}">"#,
            html_escape(&self.title)
        );
        if !self.description.is_empty() {
            let _ = writeln!(
                out,
                r#"<meta name="twitter:description" content="{}">"#,
                html_escape(&self.description)
            );
        }
        if let Some(img) = &self.og_image {
            let _ = writeln!(
                out,
                r#"<meta name="twitter:image" content="{}">"#,
                html_escape(img)
            );
        }

        // JSON-LD
        if let Some(ld) = &self.json_ld {
            let _ = writeln!(out, r#"<script type="application/ld+json">{}</script>"#, ld);
        }

        out
    }
}

/// Small HTML escaper for meta attributes.
fn html_escape(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            '&' => "&amp;".chars().collect::<Vec<_>>(),
            '<' => "&lt;".chars().collect(),
            '>' => "&gt;".chars().collect(),
            '"' => "&quot;".chars().collect(),
            '\'' => "&#39;".chars().collect(),
            _ => vec![c],
        })
        .collect()
}

#[derive(Debug, Clone)]
pub struct SiteSeoDefaults {
    pub site_title: String,
    pub site_desc: String,
    pub lang: String,
}
impl SiteSeoDefaults {
    pub fn new<S: Into<String>>(title: S, desc: S, lang: S) -> Self {
        Self {
            site_title: title.into(),
            site_desc: desc.into(),
            lang: lang.into(),
        }
    }
}
