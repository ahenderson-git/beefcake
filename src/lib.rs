#![warn(clippy::all, rust_2018_idioms)]

pub mod analyser;
mod gui;
mod theme;
mod utils;

pub use gui::BeefcakeApp;
