pub mod gui;
pub mod logic;
pub mod db;
pub mod model;
pub mod controller;

pub use gui::App;

pub fn run_analyser() -> App {
    App::default()
}
