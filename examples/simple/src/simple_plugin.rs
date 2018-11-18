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
	fn execute<'a>(&'a mut self, action: &'a mut Action) -> Result<Int> {
		match *action {
			Action::Describe(mut effect) => {
				let mut effect_properties = effect.properties_mut()?;

				effect_properties.set_image_effect_plugin_grouping("Ofx-rs")?;

				effect_properties.set_label("Ofx-rs simple_plugin sample")?;
				effect_properties.set_short_label("Ofx-rs simple_plugin")?;
				effect_properties.set_long_label("Ofx-rs simple_plugin in examples")?;

				// TODO: implement host interface
				// effect_properties.set::<image_effect::SupportsMultipleClipDepths, _>(true)?;

				effect_properties
					.set_at::<image_effect::SupportedPixelDepths>(0, kOfxBitDepthByte)?;
				effect_properties
					.set_at::<image_effect::SupportedPixelDepths>(1, kOfxBitDepthShort)?;
				effect_properties
					.set_at::<image_effect::SupportedPixelDepths>(2, kOfxBitDepthFloat)?;

				effect_properties
					.set_at::<image_effect::SupportedContexts>(0, kOfxImageEffectContextFilter)?;
				effect_properties
					.set_at::<image_effect::SupportedContexts>(1, kOfxImageEffectContextFilter)?;

				Ok(eOfxStatus_OK)
			}
			_ => Ok(eOfxStatus_OK),
		}
	}
}
