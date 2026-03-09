mod app;
mod domain;
mod features;
mod infrastructure;
mod ui;

use adw::prelude::*;

fn main() {
    let app = app::build_app();
    std::process::exit(app.run().into());
}