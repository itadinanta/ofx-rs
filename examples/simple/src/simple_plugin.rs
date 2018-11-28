use ofx::*;

plugin_module!(
	"net.itadinanta.ofx-rs.simple_plugin_1",
	ApiVersion(1),
	PluginVersion(1, 0),
	SimplePlugin::new
);

#[derive(Default)]
struct SimplePlugin {
	host_supports_multiple_clip_depths: Option<Bool>,
}

impl SimplePlugin {
	pub fn new() -> SimplePlugin {
		SimplePlugin::default()
	}
}

impl Execute for SimplePlugin {
	fn execute(&mut self, context: &PluginContext, action: &mut Action) -> Result<Int> {
		match *action {
			Action::DescribeInContext(effect, context) => {
				info!("DescribeInContext {:?} {:?}", effect, context);

				UNIMPLEMENTED
			}

			Action::Describe(effect) => {
				info!("Describe {:?}", effect);

				self.host_supports_multiple_clip_depths =
					Some(context.get_host().get_supports_multiple_clip_depths()?);

				let mut effect_properties = effect.properties()?;
				effect_properties.set_image_effect_plugin_grouping("Ofx-rs")?;

				effect_properties.set_label("Ofx-rs simple_plugin sample")?;
				effect_properties.set_short_label("Ofx-rs simple_plugin")?;
				effect_properties.set_long_label("Ofx-rs simple_plugin in examples")?;

				effect_properties.set_supported_pixel_depths(&[
					BitDepth::Byte,
					BitDepth::Short,
					BitDepth::Float,
				])?;
				effect_properties.set_supported_contexts(&[
					ImageEffectContext::Filter,
					ImageEffectContext::General,
				])?;

				OK
			}
			_ => OK,
		}
	}
}
