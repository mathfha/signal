use std::time::Duration;

use feed_rs::{model::Entry, parser};
use reqwest::Client;
use tracing::warn;

use crate::{domain::RawItem, error::AppError};

const FETCH_TIMEOUT: Duration = Duration::from_secs(15);
const USER_AGENT: &str = "TechPulse/0.1 (personal RSS reader)";

pub async fn fetch(url: &str, source_name: &str) -> Result<Vec<RawItem>, AppError> {
    let client = Client::builder()
        .timeout(FETCH_TIMEOUT)
        .user_agent(USER_AGENT)
        .build()
        .map_err(|source| AppError::FeedRequest {
            url: url.to_owned(),
            source,
        })?;
    let response = client
        .get(url)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|source| AppError::FeedRequest {
            url: url.to_owned(),
            source,
        })?;
    let body = response
        .bytes()
        .await
        .map_err(|source| AppError::FeedRequest {
            url: url.to_owned(),
            source,
        })?;

    parse(&body, url, source_name)
}

pub fn parse(bytes: &[u8], feed_url: &str, source_name: &str) -> Result<Vec<RawItem>, AppError> {
    let feed = parser::parse(bytes).map_err(|source| AppError::FeedParse {
        url: feed_url.to_owned(),
        source,
    })?;

    let mut items = Vec::with_capacity(feed.entries.len());
    for entry in feed.entries {
        match item_from_entry(entry, source_name) {
            Some(item) => items.push(item),
            None => warn!(feed_url, "skipping RSS item without a title or link"),
        }
    }

    Ok(items)
}

fn item_from_entry(entry: Entry, source_name: &str) -> Option<RawItem> {
    let title = entry.title?.content.trim().to_owned();
    if title.is_empty() {
        return None;
    }

    let url = entry
        .links
        .iter()
        .find(|link| link.rel.as_deref().is_none_or(|rel| rel == "alternate"))
        .or_else(|| entry.links.first())?
        .href
        .clone();
    if url.trim().is_empty() {
        return None;
    }

    let summary = entry.summary.map(|summary| summary.content);
    let published_at = entry.published.or(entry.updated);

    Some(RawItem {
        title,
        summary,
        url,
        source: source_name.to_owned(),
        published_at,
    })
}

#[cfg(test)]
mod tests {
    use super::parse;

    const FEED_URL: &str = "https://example.test/feed.xml";

    #[test]
    fn parses_rss_items_from_fixture() {
        let bytes = include_bytes!("../../tests/fixtures/bbc_technology.xml");

        let items = parse(bytes, FEED_URL, "BBC News").expect("fixture should parse");

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "Rust makes systems programming safer");
        assert_eq!(items[0].url, "https://example.test/rust-safety");
        assert_eq!(items[0].source, "BBC News");
        assert_eq!(items[0].summary.as_deref(), Some("A short RSS summary."));
        assert!(items[0].published_at.is_some());
    }

    #[test]
    fn skips_items_without_a_link() {
        let feed = br#"<?xml version="1.0" encoding="UTF-8"?>
            <rss version="2.0"><channel><title>Test</title><link>https://example.test</link>
            <description>Test feed</description><item><title>Missing link</title></item>
            </channel></rss>"#;

        let items = parse(feed, FEED_URL, "Test source").expect("feed should parse");

        assert!(items.is_empty());
    }
}
