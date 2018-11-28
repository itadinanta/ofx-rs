#[macro_use]
extern crate log;
extern crate log4rs;
extern crate ofx;

mod simple_plugin;
mod tests;

use ofx::*;

register_modules!(simple_plugin);
