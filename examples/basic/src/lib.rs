// Copyright (c) 2026 Nico Orru <https://itadinanta.net> - All rights reserved.
extern crate log;
extern crate log4rs;
extern crate ofx;

mod basic;
mod tests;

use ofx::*;

register_modules!(basic);
