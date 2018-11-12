use ofx::*;

plugin_module!(
	"net.itadinanta.ofx-rs.simple_plugin_1",
	ApiVersion(1),
	PluginVersion(1, 0),
	SimplePlugin::new
);

struct SimplePlugin {}

impl SimplePlugin {
	pub fn new() -> SimplePlugin {
		SimplePlugin {}
	}
}

impl Execute for SimplePlugin {
	fn execute(&mut self, action: Action) -> Result<Int> {
		match action {
			Action::Describe(handle) => Ok(eOfxStatus_OK),
			_ => Ok(eOfxStatus_OK),
		}
	}
}
