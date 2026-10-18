use crate::{config::CategoryConfig, domain::Category};

const CATEGORY_ORDER: [Category; 3] = [Category::Developer, Category::Ai, Category::Cybercrime];

pub fn classify(title: &str, summary: &str, config: &CategoryConfig) -> Vec<Category> {
    let text = normalized_tokens(&format!("{title} {summary}"));
    CATEGORY_ORDER
        .into_iter()
        .filter(|category| {
            config.categories.get(category.slug()).is_some_and(|rule| {
                rule.keywords
                    .iter()
                    .any(|keyword| contains_phrase(&text, keyword))
            })
        })
        .collect()
}

fn contains_phrase(text: &[String], phrase: &str) -> bool {
    let keyword = normalized_tokens(phrase);
    !keyword.is_empty()
        && text
            .windows(keyword.len())
            .any(|window| window == keyword.as_slice())
}

fn normalized_tokens(value: &str) -> Vec<String> {
    value
        .to_lowercase()
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use crate::{config::CategoryConfig, domain::Category};

    use super::classify;

    #[derive(Debug, Deserialize)]
    struct ClassificationCases {
        cases: Vec<ClassificationCase>,
    }

    #[derive(Debug, Deserialize)]
    struct ClassificationCase {
        title: String,
        summary: String,
        categories: Vec<String>,
    }

    #[test]
    fn classifies_reviewed_real_headlines_and_ambiguous_cases() {
        let config: CategoryConfig = toml::from_str(include_str!("../../config/categories.toml"))
            .expect("category configuration should parse");
        let cases: ClassificationCases = toml::from_str(include_str!(
            "../../tests/fixtures/classification_cases.toml"
        ))
        .expect("classification cases should parse");

        for case in cases.cases {
            let actual = classify(&case.title, &case.summary, &config)
                .into_iter()
                .map(Category::slug)
                .collect::<Vec<_>>();
            assert_eq!(actual, case.categories, "headline: {}", case.title);
        }
    }
}
