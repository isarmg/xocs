//! Run the actual frontend contracts, read models and Markdown policy natively,
//! without compiling a browser application or maintaining test-only replicas.
#![cfg(test)]
// UI-consumed DTO fields are intentionally not all read by these unit cases.
#![allow(dead_code)]
#[path = "../src/contracts.rs"]
mod contracts;
#[path = "../src/markdown.rs"]
mod markdown;
#[path = "../src/models.rs"]
mod models;
