extern crate ofx;

mod simple_plugin;
mod tests;

use ofx::*;

register_modules!(simple_plugin);
