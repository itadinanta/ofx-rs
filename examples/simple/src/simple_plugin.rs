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
	fn execute(&mut self, mut action: Action) -> Result<Int> {
		println!("We are here");
		match action {
			Action::Describe(ref mut effect_descriptor) => {
				effect_descriptor.set::<Label, _>("ofx_rs_simple_plugin")?;
				effect_descriptor.set::<ShortLabel, _>("simple plugin")?;
				effect_descriptor.set::<LongLabel, _>(
					"This is a longer desciptor for the ofx_rs_simple_plugin",
				)?;

				Ok(eOfxStatus_OK)
			}
			_ => Ok(eOfxStatus_OK),
		}
	}
}
