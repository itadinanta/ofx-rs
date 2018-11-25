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
	fn execute(&mut self, context: &PluginContext, action: &mut Action) -> Result<Int> {
		match *action {
			Action::Describe(effect) => {
				context.get_host().get_supports_multiple_clip_depths();
				let mut effect_properties = effect.properties()?;
				effect_properties.set_image_effect_plugin_grouping("Ofx-rs")?;

				effect_properties.set_label("Ofx-rs simple_plugin sample")?;
				effect_properties.set_short_label("Ofx-rs simple_plugin")?;
				effect_properties.set_long_label("Ofx-rs simple_plugin in examples")?;

				// TODO: implement host interface
				// effect_properties.set::<image_effect::SupportsMultipleClipDepths, _>(true)?;

				effect_properties.set_supported_pixel_depths(&[
					BitDepth::Byte,
					BitDepth::Short,
					BitDepth::Float,
				])?;
				effect_properties.set_supported_contexts(&[
					ImageEffectContext::Filter,
					ImageEffectContext::General,
				])?;

				Ok(eOfxStatus_OK)
			}
			_ => Ok(eOfxStatus_OK),
		}
	}
}
