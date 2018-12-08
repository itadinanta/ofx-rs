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

struct MyInstanceData {
	is_general_effect: bool,

	source_clip: ImageClipHandle,
	mask_clip: Option<ImageClipHandle>,
	output_clip: ImageClipHandle,

	scale_param: ParamHandle<Double>,

	per_component_scale_param: ParamHandle<Bool>,

	scale_r_param: ParamHandle<Double>,
	scale_g_param: ParamHandle<Double>,
	scale_b_param: ParamHandle<Double>,
	scale_a_param: ParamHandle<Double>,
}

impl Execute for SimplePlugin {
	fn execute(&mut self, plugin_context: &PluginContext, action: &mut Action) -> Result<Int> {
		match *action {
			Action::CreateInstance(ref mut effect) => {
				let mut effect_props = effect.properties()?;
				let mut param_set = effect.parameter_set()?;

				let is_general_effect =
					ImageEffectContext::General == effect_props.get_context()?;

				let per_component_scale_param = param_set.parameter("scale")?;

				let source_clip = effect.get_simple_input_clip()?;
				let output_clip = effect.get_output_clip()?;
				let mask_clip = if is_general_effect {
					Some(effect.get_clip("Mask")?)
				} else {
					None
				};

				let scale_param = param_set.parameter("scale")?;
				let scale_r_param = param_set.parameter("scaleR")?;
				let scale_g_param = param_set.parameter("scaleG")?;
				let scale_b_param = param_set.parameter("scaleB")?;
				let scale_a_param = param_set.parameter("scaleA")?;

				effect.set_instance_data(MyInstanceData {
					is_general_effect,
					source_clip,
					mask_clip,
					output_clip,
					per_component_scale_param,
					scale_param,
					scale_r_param,
					scale_g_param,
					scale_b_param,
					scale_a_param,
				})?;

				Self::set_per_component_scale_enabledness(effect);

				UNIMPLEMENTED
			}

			Action::DestroyInstance(ref mut effect) => {
				effect.drop_instance_data()?;

				OK
			}

			Action::DescribeInContext(ref mut effect, context) => {
				info!("DescribeInContext {:?} {:?}", effect, context);

				let mut output_clip = effect.new_output_clip()?;
				output_clip
					.set_supported_components(&[ImageComponent::RGBA, ImageComponent::Alpha])?;

				let mut input_clip = effect.new_simple_input_clip()?;
				input_clip
					.set_supported_components(&[ImageComponent::RGBA, ImageComponent::Alpha])?;

				if context == ImageEffectContext::General {
					let mut mask = effect.new_clip("Mask")?;
					mask.set_supported_components(&[ImageComponent::Alpha])?;
					mask.set_optional(true)?;
				}

				fn define_scale_param(
					param_set: &mut ParamSetHandle,
					name: &str,
					label: &'static str,
					script_name: &'static str,
					hint: &'static str,
					parent: Option<&'static str>,
				) -> Result<()> {
					let mut param_props = param_set.param_define_double(name)?;

					param_props.set_double_type(ParamDoubleType::Scale)?;
					param_props.set_label(label)?;
					param_props.set_default(1.0)?;
					param_props.set_display_min(1.0)?;
					param_props.set_display_min(1.0)?;
					param_props.set_display_max(100.0)?;
					param_props.set_hint(hint)?;
					param_props.set_script_name(script_name)?;

					if let Some(parent) = parent {
						param_props.set_parent(parent)?;
					}

					Ok(())
				}

				let mut param_set = effect.parameter_set()?;
				define_scale_param(
					&mut param_set,
					"scale",
					"scale",
					"scale",
					"Scales all component in the image",
					None,
				)?;

				let mut param_props = param_set.param_define_boolean("scaleComponents")?;
				param_props.set_default(false)?;
				param_props.set_hint("Enables scale on individual components")?;
				param_props.set_script_name("scaleComponents")?;
				param_props.set_label("Scale Individual Components")?;

				let mut param_props = param_set.param_define_boolean("componentScales")?;
				param_props.set_hint("Scales on the individual component")?;
				param_props.set_label("Components")?;

				define_scale_param(
					&mut param_set,
					"scaleR",
					"red",
					"scaleR",
					"Scales the red component of the image",
					Some("componentScales"),
				)?;
				define_scale_param(
					&mut param_set,
					"scaleG",
					"green",
					"scaleG",
					"Scales the green component of the image",
					Some("componentScales"),
				)?;
				define_scale_param(
					&mut param_set,
					"scaleB",
					"blue",
					"scaleB",
					"Scales the blue component of the image",
					Some("componentScales"),
				)?;
				define_scale_param(
					&mut param_set,
					"scaleA",
					"alpha",
					"scaleA",
					"Scales the alpha component of the image",
					Some("componentScales"),
				)?;

				let mut param_props = param_set.param_define_page("Main")?;
				param_props.set_children(&[
					"scale",
					"scaleComponents",
					"scaleR",
					"scaleG",
					"scaleB",
					"scaleA",
				])?;

				OK
			}

			Action::Describe(ref mut effect) => {
				info!("Describe {:?}", effect);

				self.host_supports_multiple_clip_depths = Some(
					plugin_context
						.get_host()
						.get_supports_multiple_clip_depths()?,
				);

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

impl SimplePlugin {
	fn set_param_enabledness<T>(
		effect: &mut ImageEffectHandle,
		name: &str,
		enabled: bool,
	) -> Result<()> where T: ParamHandleValue{
		let mut parameter = effect.parameter_set()?.parameter::<T>(name)?;
		parameter.set_enabled(enabled)?;
		//let instance_data = effect.get_instance_data::<MyInstanceData>()?;
		//instance_data.per_component_scale_param.get_value();
		Ok(())
	}

	fn set_per_component_scale_enabledness(effect: &mut ImageEffectHandle) -> Result<()> {
		let source_clip = effect.get_simple_input_clip()?.clone();
		let mut instance_data = effect.get_instance_data::<MyInstanceData>()?;
		//instance_data.per_component_scale_param;
		let is_connected = source_clip.get_connected()?;
		//source_clip.get_components();
		//instance_data.per_component_scale_param.get_value();
		//Self::set_param_enabledness(effect, "scaleR", per_component_scale);
		Ok(())
	}
}
