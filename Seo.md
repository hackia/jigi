# SEO Guide: Practical Usage and Best Practices

## TL;DR Checklist

- Title: 50–60 chars, unique, includes main keyword.
- Description: 120–160 chars, compelling summary with a call to action.
- Canonical URL: Absolute URL for indexable pages.
- Language: Use a valid BCP 47 code (e.g., `en`, `fr`, `en-US`).
- Open Graph:
  - `og:type`: `website` for home/landing, `article` for posts.
  - `og:image`: Absolute URL, 1200×630 px, ≤ 5 MB, JPG/PNG/WebP.
- Twitter: Use `summary_large_image` unless you specifically want a small card.
- JSON-LD: Add a relevant schema (`WebSite`, `Article`, etc.) with ISO8601 dates.
- Keywords: Optional; if used, 3–8 concise terms, comma-separated.
- Slug: URL-safe string for your page identity.
- Updated: ISO8601 datetime for last-modified.
- Validate: Test in Google Rich Results, Open Graph Debugger, and Twitter Validator.

## Field Reference and Recommendations

- title
  - Purpose: Page title shown in SERP and social previews.
    - Recommendations: 50–60 characters; place important words early; keep brand suffix consistent.

- description
  - Purpose: SERP snippet; can affect CTR.
  - Recommendations: 120–160 characters; be clear and actionable.
- keywords (optional)
  - Purpose: Legacy meta; low SEO impact.
  - Recommendations: Use sparingly (3–8). Don't stuff.

- author (optional)
  - Purpose: Attribution; useful for articles.
  - Recommendations: Plain human-readable name.
- canonical_url (optional but recommended)
  - Purpose: Deduplicate content; signals the preferred URL
  - Recommendations: Use an absolute URL (e.g., https://example.com/path).
- lang (optional)
  - Purpose: Indicate page language.
  - Recommendations: BCP 47 codes (e.g., `en`, `fr`, `pt-BR`).
- updated (optional)
  - Purpose: Last-modified for search engines and users.
  - Recommendations: ISO8601 format, e.g., `2025-05-20T12:30:00Z`.
- og_image (optional but strongly recommended for sharing)
  - Purpose: Social preview image.
  - Recommendations: Absolute URL; 1200×630 px; include brand-safe margins.
- og_type (optional)
  - Purpose: Open Graph type; affects how the platforms parses content.
  - Values: `website`, `article`, `book`, `profile`, etc.
  - Default strategy: `website` site-wide, `article` for posts.
- twitter_card (optional)
  - Purpose: Twitter presentation.
  - Recommended: `summary_large_image`.
- json_ld (optional)
  - Purpose: Structured data for rich results.
  - Recommendations: Valid JSON; match visible content; keep current.
- content_type (optional)
  - Purpose: Internal or template logic (e.g., `work`, `author`, `season`, `event`).
- slug (optional)
  - Purpose: URL identity for the page.
  - Recommendations: Lowercase, hyphen-separated, stable.