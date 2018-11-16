use ofx::*;
use std::ffi::CStr;

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
			Action::Describe(mut effect_descriptor) => {
				let mut effect_properties = effect_descriptor.properties_mut()?;

				effect_properties.set::<Label, _>("ofx_rs_simple_plugin")?;
				effect_properties.set::<ShortLabel, _>("simple_plugin")?;
				effect_properties.set::<LongLabel, _>("longer_description")?;
				effect_properties.set::<image_effect_plugin::Grouping, _>("ofx_rs")?;

				//effect_properties.set::<image_effect::SupportsMultipleClipDepths, _>(true)?;

				effect_properties.set_at::<image_effect::SupportedPixelDepths, _>(
					0,
					CStr::from_bytes_with_nul(kOfxBitDepthByte)?.to_str()?,
				)?;
				effect_properties.set_at::<image_effect::SupportedPixelDepths, _>(
					1,
					CStr::from_bytes_with_nul(kOfxBitDepthShort)?.to_str()?,
				)?;
				effect_properties.set_at::<image_effect::SupportedPixelDepths, _>(
					2,
					CStr::from_bytes_with_nul(kOfxBitDepthFloat)?.to_str()?,
				)?;

				effect_properties.set_at::<image_effect::SupportedContexts, _>(
					0,
					CStr::from_bytes_with_nul(kOfxImageEffectContextFilter)?.to_str()?,
				)?;
				effect_properties.set_at::<image_effect::SupportedContexts, _>(
					1,
					CStr::from_bytes_with_nul(kOfxImageEffectContextFilter)?.to_str()?,
				)?;

				Ok(eOfxStatus_OK)
			}
			_ => Ok(eOfxStatus_OK),
		}
	}
}
