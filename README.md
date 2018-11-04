## OpenFX bindings

Replaces the client Support C++ layer, using the raw ofx* ABI via bindgen only.

### Design goals

- data and type safe
- ergonomic API for Image Effect plugin writers
- one dll/so crate can contain multiple plugins
- each plugin in its own Rust module
- centralised plugin registry per crate

### Example plugin shell

`lib.rs`:

```rust
extern crate ofx;

mod simple_plugin;
mod tests;

use ofx::*;

register_modules!(simple_plugin);
```

`simple_plugin.rs`

```rust
use ofx::*;

// plugin declaration
plugin_module!(
	"net.itadinanta.ofx-rs.simple_plugin_1",
	ApiVersion(1),
	PluginVersion(1, 0),
	SimplePlugin::new
);

// custom plugin data goes here
struct SimplePlugin {}

impl SimplePlugin {
	pub fn new() -> SimplePlugin {
		SimplePlugin {}
	}
}

impl Execute for SimplePlugin {
// plugin logic goes here
}


```