use std::sync::Arc;

use askama::Template;
use axum::{
    body::Body,
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use tokio::{net::TcpListener, task::JoinSet};
use tracing::warn;

use crate::{
    config::{AppConfig, CategoryConfig},
    domain::RawItem,
    pipeline::classify::classify,
    sources::rss,
};

const STYLE_CSS: &str = include_str!("../../static/style.css");
const HTMX_JS: &str = include_str!("../../static/htmx.min.js");

#[derive(Clone, Debug)]
pub struct AppState {
    snapshot: Arc<FeedSnapshot>,
}

#[derive(Clone, Debug, Default)]
struct FeedSnapshot {
    stories: Vec<Story>,
    sources: Vec<SourceOption>,
    feed_states: Vec<FeedState>,
    last_successful_fetch: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Story {
    title: String,
    summary: Option<String>,
    url: String,
    source_id: String,
    source: String,
    published_at: DateTime<Utc>,
    published_at_iso: String,
    exact_time: String,
    relative_time: String,
    categories: Vec<CategoryLabel>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CategoryLabel {
    slug: String,
    name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SourceOption {
    id: String,
    name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FeedState {
    source_name: String,
    last_error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CategoryLink {
    name: String,
    href: String,
    active: bool,
}

#[derive(Clone, Debug, Deserialize, Default)]
struct FilterQuery {
    category: Option<String>,
    source: Option<String>,
}

#[derive(Clone, Debug)]
struct NewsPanel {
    stories: Vec<Story>,
    category_links: Vec<CategoryLink>,
    all_stories_url: String,
    sources: Vec<SourceOption>,
    selected_category: String,
    selected_source: String,
    has_fetch_time: bool,
    fetch_time: String,
    fetch_time_iso: String,
    feed_errors: Vec<String>,
    empty_message: String,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    page_title: &'a str,
    panel: NewsPanel,
}

#[derive(Template)]
#[template(path = "_story_list.html")]
struct StoryListTemplate {
    panel: NewsPanel,
}

impl AppState {
    pub async fn load(config: AppConfig, category_config: CategoryConfig) -> Self {
        let sources = config
            .sources
            .iter()
            .map(|(id, feed)| SourceOption {
                id: id.clone(),
                name: feed.name.clone(),
            })
            .collect();
        let mut stories = Vec::new();
        let mut feed_states = Vec::new();
        let mut last_successful_fetch = None;
        let mut fetches = JoinSet::new();

        for (source_id, source) in config.sources {
            fetches.spawn(async move {
                let result = rss::fetch(&source.url, &source.name).await;
                (source_id, source.name, result)
            });
        }

        while let Some(result) = fetches.join_next().await {
            match result {
                Ok((source_id, source_name, result)) => match result {
                    Ok(items) => {
                        let fetch_time = Utc::now();
                        last_successful_fetch = Some(
                            last_successful_fetch
                                .map(|current: DateTime<Utc>| current.max(fetch_time))
                                .unwrap_or(fetch_time),
                        );
                        feed_states.push(FeedState {
                            source_name,
                            last_error: None,
                        });
                        stories.extend(items.into_iter().filter_map(|item| {
                            story_from_item(item, &source_id, &category_config)
                        }));
                    }
                    Err(error) => {
                        warn!(source = %source_name, source_id, %error, "feed fetch failed");
                        feed_states.push(FeedState {
                            source_name,
                            last_error: Some(error.to_string()),
                        });
                    }
                },
                Err(error) => warn!(%error, "feed fetch task failed"),
            }
        }

        sort_newest_first(&mut stories);

        Self {
            snapshot: Arc::new(FeedSnapshot {
                stories,
                sources,
                feed_states,
                last_successful_fetch,
            }),
        }
    }

    #[cfg(test)]
    fn from_snapshot(snapshot: FeedSnapshot) -> Self {
        Self {
            snapshot: Arc::new(snapshot),
        }
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/static/style.css", get(style))
        .route("/static/htmx.min.js", get(htmx))
        .with_state(state)
}

async fn health() -> StatusCode {
    StatusCode::OK
}

pub async fn serve(state: AppState, address: &str) -> anyhow::Result<()> {
    let listener = TcpListener::bind(address).await?;
    tracing::info!(%address, "web server listening");
    axum::serve(listener, router(state)).await?;
    Ok(())
}

async fn index(
    State(state): State<AppState>,
    Query(query): Query<FilterQuery>,
    headers: HeaderMap,
) -> Response {
    let panel = build_panel(&state.snapshot, &query);
    if is_htmx_request(&headers) {
        return render_response(StoryListTemplate { panel });
    }

    render_response(IndexTemplate {
        page_title: "Signal",
        panel,
    })
}

fn is_htmx_request(headers: &HeaderMap) -> bool {
    headers
        .get("HX-Request")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("true"))
}

fn render_response<T: Template>(template: T) -> Response {
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(error) => {
            warn!(%error, "could not render page template");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "The page could not be rendered.",
            )
                .into_response()
        }
    }
}

async fn style() -> Response<Body> {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        STYLE_CSS,
    )
        .into_response()
}

async fn htmx() -> Response<Body> {
    (
        [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        HTMX_JS,
    )
        .into_response()
}

fn build_panel(snapshot: &FeedSnapshot, query: &FilterQuery) -> NewsPanel {
    let category = query
        .category
        .as_deref()
        .filter(|value| matches!(*value, "developer" | "ai" | "cybercrime"));
    let source = query
        .source
        .as_deref()
        .filter(|value| snapshot.sources.iter().any(|item| item.id == *value));
    let stories = snapshot
        .stories
        .iter()
        .filter(|story| {
            category.is_none_or(|slug| story.categories.iter().any(|item| item.slug == slug))
                && source.is_none_or(|id| story.source_id == id)
        })
        .cloned()
        .collect::<Vec<_>>();
    let category_links = category_options()
        .into_iter()
        .map(|(slug, name)| CategoryLink {
            name: name.to_owned(),
            href: filter_url(Some(slug), source),
            active: category == Some(slug),
        })
        .collect();
    let feed_errors: Vec<String> = snapshot
        .feed_states
        .iter()
        .filter_map(|feed| {
            feed.last_error
                .as_ref()
                .map(|error| format!("{}: {error}", feed.source_name))
        })
        .collect();
    let fetch_time = snapshot
        .last_successful_fetch
        .map(format_timestamp)
        .unwrap_or_default();
    let fetch_time_iso = snapshot
        .last_successful_fetch
        .map(|time| time.to_rfc3339())
        .unwrap_or_default();
    let empty_message = if !feed_errors.is_empty() && snapshot.stories.is_empty() {
        "No stories could be loaded because every feed failed. Try again later.".to_owned()
    } else if category.is_some() || source.is_some() {
        "No stories match these filters.".to_owned()
    } else {
        "No stories are available yet. Check back after the next feed update.".to_owned()
    };

    NewsPanel {
        stories,
        category_links,
        all_stories_url: filter_url(None, source),
        sources: snapshot.sources.clone(),
        selected_category: category.unwrap_or_default().to_owned(),
        selected_source: source.unwrap_or_default().to_owned(),
        has_fetch_time: !fetch_time.is_empty(),
        fetch_time,
        fetch_time_iso,
        feed_errors,
        empty_message,
    }
}

fn category_options() -> [(&'static str, &'static str); 3] {
    [
        ("developer", "Developer"),
        ("ai", "AI"),
        ("cybercrime", "Cybercrime"),
    ]
}

fn filter_url(category: Option<&str>, source: Option<&str>) -> String {
    let mut params = url::form_urlencoded::Serializer::new(String::from("/"));
    if let Some(category) = category {
        params.append_pair("category", category);
    }
    if let Some(source) = source {
        params.append_pair("source", source);
    }
    params.finish()
}

fn story_from_item(
    item: RawItem,
    source_id: &str,
    category_config: &CategoryConfig,
) -> Option<Story> {
    let published_at = item.published_at?;
    let summary = item.summary.as_deref().unwrap_or_default();
    let categories = classify(&item.title, summary, category_config)
        .into_iter()
        .map(|category| CategoryLabel {
            slug: category.slug().to_owned(),
            name: category.name().to_owned(),
        })
        .collect::<Vec<_>>();
    if categories.is_empty() {
        return None;
    }

    Some(Story {
        title: item.title,
        summary: item.summary.map(|summary| truncate_summary(&summary, 280)),
        url: item.url,
        source_id: source_id.to_owned(),
        source: item.source,
        published_at_iso: published_at.to_rfc3339(),
        exact_time: format_timestamp(published_at),
        relative_time: relative_time(published_at, Utc::now()),
        published_at,
        categories,
    })
}

fn truncate_summary(summary: &str, max_characters: usize) -> String {
    let mut characters = summary.chars();
    let shortened: String = characters.by_ref().take(max_characters).collect();
    if characters.next().is_some() {
        format!("{shortened}…")
    } else {
        shortened
    }
}

fn format_timestamp(time: DateTime<Utc>) -> String {
    time.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

fn sort_newest_first(stories: &mut [Story]) {
    stories.sort_by_key(|story| std::cmp::Reverse(story.published_at));
}

fn relative_time(published_at: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let difference = now.signed_duration_since(published_at);
    if difference < Duration::minutes(1) {
        "just now".to_owned()
    } else if difference < Duration::hours(1) {
        format!("{}m ago", difference.num_minutes())
    } else if difference < Duration::days(1) {
        format!("{}h ago", difference.num_hours())
    } else if difference < Duration::days(7) {
        format!("{}d ago", difference.num_days())
    } else {
        format!("{}w ago", difference.num_weeks())
    }
}

#[cfg(test)]
mod tests {
    use axum::{body::to_bytes, http::Request};
    use chrono::{TimeZone, Utc};
    use tower::ServiceExt;

    use super::{router, AppState, CategoryLabel, FeedSnapshot, FeedState, SourceOption, Story};

    fn fixture_state() -> AppState {
        let published_at = Utc.with_ymd_and_hms(2026, 10, 18, 9, 30, 0).unwrap();
        AppState::from_snapshot(FeedSnapshot {
            stories: vec![
                Story {
                    title: "Rust compiler update".to_owned(),
                    summary: Some("New developer tools".to_owned()),
                    url: "https://example.test/rust".to_owned(),
                    source_id: "bbc".to_owned(),
                    source: "bbc".to_owned(),
                    published_at,
                    published_at_iso: "2026-10-18T09:30:00+00:00".to_owned(),
                    exact_time: "2026-10-18 09:30:00 UTC".to_owned(),
                    relative_time: "1h ago".to_owned(),
                    categories: vec![CategoryLabel {
                        slug: "developer".to_owned(),
                        name: "Developer".to_owned(),
                    }],
                },
                Story {
                    title: "AI model breach".to_owned(),
                    summary: Some("A cybercrime incident".to_owned()),
                    url: "https://example.test/ai".to_owned(),
                    source_id: "dw".to_owned(),
                    source: "dw".to_owned(),
                    published_at,
                    published_at_iso: "2026-10-18T09:30:00+00:00".to_owned(),
                    exact_time: "2026-10-18 09:30:00 UTC".to_owned(),
                    relative_time: "1h ago".to_owned(),
                    categories: vec![
                        CategoryLabel {
                            slug: "ai".to_owned(),
                            name: "AI".to_owned(),
                        },
                        CategoryLabel {
                            slug: "cybercrime".to_owned(),
                            name: "Cybercrime".to_owned(),
                        },
                    ],
                },
            ],
            sources: vec![
                SourceOption {
                    id: "bbc".to_owned(),
                    name: "BBC News".to_owned(),
                },
                SourceOption {
                    id: "dw".to_owned(),
                    name: "DW News".to_owned(),
                },
            ],
            feed_states: vec![FeedState {
                source_name: "AP News".to_owned(),
                last_error: Some("feed unavailable".to_owned()),
            }],
            last_successful_fetch: Some(published_at),
        })
    }

    #[tokio::test]
    async fn htmx_request_returns_only_the_story_list_partial() {
        let response = router(fixture_state())
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header("HX-Request", "true")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();

        assert!(html.contains("id=\"story-list\""));
        assert!(!html.contains("<!doctype html>"));
        assert!(!html.contains("<header"));
    }

    #[tokio::test]
    async fn ordinary_request_returns_the_full_page() {
        let response = router(fixture_state())
            .oneshot(
                Request::builder()
                    .uri("/")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();

        assert!(html.contains("<!doctype html>"));
        assert!(html.contains("<header"));
    }

    #[tokio::test]
    async fn health_endpoint_returns_success() {
        let response = router(fixture_state())
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn category_filter_returns_only_matching_stories() {
        let response = router(fixture_state())
            .oneshot(
                Request::builder()
                    .uri("/?category=ai")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();

        assert!(html.contains("AI model breach"));
        assert!(!html.contains("Rust compiler update"));
    }

    #[tokio::test]
    async fn source_filter_returns_only_matching_stories() {
        let response = router(fixture_state())
            .oneshot(
                Request::builder()
                    .uri("/?source=dw")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();

        assert!(html.contains("AI model breach"));
        assert!(!html.contains("Rust compiler update"));
    }

    #[tokio::test]
    async fn empty_snapshot_shows_useful_empty_and_feed_error_messages() {
        let state = AppState::from_snapshot(FeedSnapshot {
            stories: Vec::new(),
            sources: Vec::new(),
            feed_states: vec![FeedState {
                source_name: "AP News".to_owned(),
                last_error: Some("feed unavailable".to_owned()),
            }],
            last_successful_fetch: None,
        });
        let response = router(state)
            .oneshot(
                Request::builder()
                    .uri("/")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();

        assert!(html.contains("No stories could be loaded"));
        assert!(html.contains("AP News: feed unavailable"));
        assert!(html.contains("No feed has completed successfully yet."));
    }

    #[test]
    fn stories_are_sorted_newest_first() {
        let old = Utc.with_ymd_and_hms(2026, 10, 17, 9, 30, 0).unwrap();
        let new = Utc.with_ymd_and_hms(2026, 10, 18, 9, 30, 0).unwrap();
        let mut stories = vec![story_at("Old story", old), story_at("New story", new)];

        super::sort_newest_first(&mut stories);

        assert_eq!(stories[0].title, "New story");
    }

    fn story_at(title: &str, published_at: chrono::DateTime<Utc>) -> Story {
        Story {
            title: title.to_owned(),
            summary: None,
            url: "https://example.test/story".to_owned(),
            source_id: "bbc".to_owned(),
            source: "BBC News".to_owned(),
            published_at,
            published_at_iso: published_at.to_rfc3339(),
            exact_time: super::format_timestamp(published_at),
            relative_time: "1d ago".to_owned(),
            categories: Vec::new(),
        }
    }
}
