# ADR 0001: SQLite and server-rendered HTML for v1

- Status: accepted
- Date: 2026-10-18

## Context
This is a single-user project with a small dataset. The main goals are to learn Rust and get a working result quickly.

## Options considered
1. SQLite, Askama templates, and HTMX
2. Postgres, a JSON API, and a React frontend
3. In-memory storage without a database

## Decision
Choose option 1: one binary with SQLite, server-rendered HTML from Askama
templates, and HTMX for partial page interactions. There is no separate
frontend application or build step.

## Consequences
- Setup is minimal, and testing is straightforward.
- Askama renders both full pages and HTML fragments. HTMX requests those fragments
  from the Axum routes for interactions such as category and source filters.
- If a separate frontend is needed later, JSON routes can be added to `web`
  without changing the pipeline.
- Moving to Postgres will require replacing the `storage` implementation, so keep SQL in one module.
