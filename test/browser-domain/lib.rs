//! Run the actual frontend contracts, read models and Markdown policy natively,
//! without compiling a browser application or maintaining test-only replicas.
#![cfg(test)]
// UI-consumed DTO fields are intentionally not all read by these unit cases.
#![allow(dead_code)]
#[path = "../../src/browser/src/contracts.rs"]
mod contracts;
#[path = "../../src/browser/src/editor/insertion.rs"]
mod editor_insertion;
#[path = "../../src/browser/src/markdown.rs"]
mod markdown;
#[path = "../../src/browser/src/models.rs"]
mod models;
