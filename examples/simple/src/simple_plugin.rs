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

pub trait HasLabel: Writable {
	fn set_label<S>(&mut self, value: S) -> Result<()>
	where
		S: Into<String>,
	{
		self.set::<Label, _>(value)
	}
}

impl<'a> HasLabel for PropertySetHandle<'a> {}

impl Execute for SimplePlugin {
	fn execute<'a>(&'a mut self, action: &'a mut Action) -> Result<Int> {
		match *action {
			Action::Describe(mut effect) => {
				let mut effect_properties = effect.properties_mut()?;

				effect_properties.set::<image_effect_plugin::Grouping, _>("Ofx-rs")?;
				//effect_properties.set::<Label, _>("Ofx-rs simple_plugin sample")?;
				effect_properties.set::<Label, _>("Ofx-rs simple_plugin sample")?;
				effect_properties.set::<ShortLabel, _>("Ofx-rs simple_plugin")?;
				effect_properties.set::<LongLabel, _>("Ofx-rs simple_plugin in examples")?;

				// TODO: implement host interface
				// effect_properties.set::<image_effect::SupportsMultipleClipDepths, _>(true)?;

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
