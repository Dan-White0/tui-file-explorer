use std::{env, io};

mod components;
use components::App;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::new(env::current_dir().unwrap()).run(&mut terminal);
    ratatui::restore();
    app_result
}
