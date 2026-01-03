pub mod controller;
pub mod db;
pub mod gui;
pub mod logic;
pub mod model;

pub use gui::App;

pub fn run_analyser() -> App {
    App::default()
}
