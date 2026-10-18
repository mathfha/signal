# PRD: TechPulse

TechPulse aggregates technology news from international outlets and filters it into three categories: **Developer**, **AI**, and **Cybercrime**.

## Problem
Technology news is spread across many outlets and mixed with other coverage, such as politics and sports. Readers who want developer, AI, or cybercrime news have to visit each outlet and filter stories manually.

## Target users
- Students and developers who want quick updates on the IT industry.
- Initially, a single user: the project author. There are no accounts or login.

## News sources
| Source | Notes |
|---|---|
| BBC News | Official RSS feeds are available, including a Technology section. |
| Reuters | No public technology RSS endpoint has been verified; leave unconfigured until an authorized feed is available. |
| AP News | The discovered `/index.rss` endpoint requires client credentials; leave unconfigured until a public feed is verified. |
| Al Jazeera | Official all-news RSS feed configured. |
| The Guardian | Official RSS URL pattern supports a Technology section feed. |
| CNN | Documented feeds are stale; the latest Technology feed item found was from August 2024, so it is not configured. |
| The New York Times | Official Technology RSS feed configured; full article text is behind a paywall. TechPulse stores metadata and a link only. |
| DW | Official English all-news RSS feed configured. |

Feed URLs belong in `config/sources.toml` and must be verified manually before use. Do not invent feed URLs.

## Categories
| Category | Coverage |
|---|---|
| Developer | Programming languages, frameworks, open source, tools, DevOps, and software engineering |
| AI | Machine learning, LLMs, generative AI, AI regulation, and AI companies |
| Cybercrime | Ransomware, data breaches, phishing, malware, vulnerabilities, and hacker arrests |

A story may belong to more than one category, such as a story about AI used for phishing. Do not display stories that match no category.

## Functional requirements
- [ ] FR-01: Fetch feeds from all configured sources on a regular schedule.
- [ ] FR-02: Normalize each item into one format: title, summary, URL, source, and publication time.
- [ ] FR-03: Remove duplicates with the same normalized URL or a highly similar title.
- [x] FR-04: Classify stories as Developer, AI, or Cybercrime using keyword rules.
- [x] FR-05: Show the latest stories, ordered by publication time.
- [x] FR-06: Filter stories by category and source.
- [x] FR-07: Show each story's source and a link to the original article.
- [x] FR-08: An error from one source must not stop processing other sources.

## Non-functional requirements
- Legal and ethical handling: store metadata only (title, short summary, link, source, and publication time), never full article text. Always link to the source.
- Be polite to sources: use a clear User-Agent, send `ETag` and `If-Modified-Since` headers, and fetch each source no more often than every 15 minutes.
- Reliability: use a timeout and a limited number of retries for each fetch.
- Performance: render the home page from the local database in under 500 ms.
- Privacy: do not store user data.

## Acceptance criteria
- Given a valid BBC feed, when the fetcher runs, new items are stored in the database without duplicates.
- Given a story about ransomware, when it is classified, it belongs to the Cybercrime category.
- Given that one source times out, when the fetcher runs, other sources are still processed and the error is logged.
- Given that a user opens the AI filter, only stories in the AI category are shown.

## Out of scope for version 1
- Accounts, login, and bookmarks.
- Scraping full article text from HTML pages.
- Automatic translation.
- AI summaries, notifications, and a mobile app.

## Open questions
1. Reuters and AP: remain unconfigured until an authorized public feed or API is identified; do not substitute Google News without reviewing its terms.
2. Database: is SQLite enough for now?
3. Where should the app be deployed, or should it run locally for now?

## Resolved product decisions
- Display stack: Askama for server-rendered pages and HTML fragments, with HTMX
  for partial page interactions. No separate frontend application in v1.
- Interface language: English.
