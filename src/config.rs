use std::{collections::BTreeMap, fs, path::Path};

use serde::Deserialize;
use url::Url;

use crate::error::AppError;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub sources: BTreeMap<String, FeedConfig>,
}

#[derive(Debug, Deserialize)]
pub struct FeedConfig {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CategoryConfig {
    pub categories: BTreeMap<String, CategoryRule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CategoryRule {
    pub name: String,
    pub keywords: Vec<String>,
}

impl CategoryConfig {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, AppError> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path).map_err(|source| AppError::ConfigRead {
            path: path.display().to_string(),
            source,
        })?;
        let config: Self = toml::from_str(&contents)?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), AppError> {
        let expected = [
            ("developer", "Developer"),
            ("ai", "AI"),
            ("cybercrime", "Cybercrime"),
        ];
        if self.categories.len() != expected.len() {
            return Err(AppError::ConfigValidation(
                "category configuration must define Developer, AI, and Cybercrime".to_owned(),
            ));
        }

        for (slug, name) in expected {
            let Some(rule) = self.categories.get(slug) else {
                return Err(AppError::ConfigValidation(format!(
                    "category configuration is missing {name}"
                )));
            };
            if rule.name != name || rule.keywords.is_empty() {
                return Err(AppError::ConfigValidation(format!(
                    "category {name} must use its fixed name and have at least one keyword"
                )));
            }
            if rule
                .keywords
                .iter()
                .any(|keyword| keyword.trim().is_empty())
            {
                return Err(AppError::ConfigValidation(format!(
                    "category {name} contains an empty keyword"
                )));
            }
        }

        Ok(())
    }
}

impl AppConfig {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, AppError> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path).map_err(|source| AppError::ConfigRead {
            path: path.display().to_string(),
            source,
        })?;
        let config: Self = toml::from_str(&contents)?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), AppError> {
        if self.sources.is_empty() {
            return Err(AppError::ConfigValidation(
                "at least one feed source must be configured".to_owned(),
            ));
        }

        for (id, source) in &self.sources {
            if id.trim().is_empty() || source.name.trim().is_empty() {
                return Err(AppError::ConfigValidation(
                    "source IDs and names must not be empty".to_owned(),
                ));
            }

            let parsed_url = Url::parse(&source.url).map_err(|error| {
                AppError::ConfigValidation(format!("source {id} has an invalid URL: {error}"))
            })?;
            if !matches!(parsed_url.scheme(), "http" | "https") {
                return Err(AppError::ConfigValidation(format!(
                    "source {id} URL must use HTTP or HTTPS"
                )));
            }
        }

        Ok(())
    }
}
