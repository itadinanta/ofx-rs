## OpenFX bindings

Replaces the client Support C++ layer, using the raw ofx* ABI via bindgen only.

01/2019 status: Not ready for production use but basic functionality implemented, available via `crates.io`. 

### Design goals

- data and type safe
- ergonomic API for Image Effect plugin writers
- one dll/so crate can contain multiple plugins
- each plugin in its own Rust module
- centralised plugin registry per crate

### Cargo.toml

```
[dependencies]
ofx = "0.1"
```

### Example code


The example at https://github.com/itadinanta/ofx-rs/examples/basic is an almost line-by-line translation of the `basic.cpp` (https://github.com/NatronGitHub/openfx/tree/4fc7b53bc9ad86bb323e971d553b8482916a62d9/Examples/Basic) example in OpenFX.

Tested in Linux only using (Natron)[https://natron.fr/] as the host application. See example in`test_in_natron.sh`. Requires configuration of Natron OFX plugin paths.

### Example plugin skeleton

`lib.rs`

```rust
extern crate ofx;

mod simple_plugin;

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