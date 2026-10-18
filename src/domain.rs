use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Developer,
    Ai,
    Cybercrime,
}

impl Category {
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Developer => "developer",
            Self::Ai => "ai",
            Self::Cybercrime => "cybercrime",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Developer => "Developer",
            Self::Ai => "AI",
            Self::Cybercrime => "Cybercrime",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawItem {
    pub title: String,
    pub summary: Option<String>,
    pub url: String,
    pub source: String,
    pub published_at: Option<DateTime<Utc>>,
}
