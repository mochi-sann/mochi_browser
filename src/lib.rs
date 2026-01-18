#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub mod html_tokenizer;
pub mod http;

pub use app::TemplateApp;
