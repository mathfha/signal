use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("could not read configuration at {path}: {source}")]
    ConfigRead {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("could not parse configuration: {0}")]
    ConfigParse(#[from] toml::de::Error),
    #[error("invalid configuration: {0}")]
    ConfigValidation(String),
    #[error("could not fetch feed {url}: {source}")]
    FeedRequest {
        url: String,
        #[source]
        source: reqwest::Error,
    },
    #[error("could not parse RSS feed from {url}: {source}")]
    FeedParse {
        url: String,
        #[source]
        source: feed_rs::parser::ParseFeedError,
    },
}
