pub mod gui;
pub mod logic;

pub use gui::App;

pub fn run_analyser() -> App {
    App::default()
}
