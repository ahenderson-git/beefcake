pub mod gui;
pub mod logic;
pub mod db;

pub use gui::App;

pub fn run_analyser() -> App {
    App::default()
}
