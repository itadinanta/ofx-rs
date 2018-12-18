use boolinator::*;
use ofx::*;

plugin_module!(
	"net.itadinanta.ofx-rs.simple_plugin_1",
	ApiVersion(1),
	PluginVersion(1, 0),
	SimplePlugin::new
);

#[derive(Default)]
struct SimplePlugin {
	host_supports_multiple_clip_depths: Bool,
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
	#[allow(clippy::float_cmp)]
	fn execute(&mut self, plugin_context: &PluginContext, action: &mut Action) -> Result<Int> {
		match *action {
			Action::IsIdentity(ref mut effect, ref in_args, ref mut out_args) => {
				let time = in_args.get_time()?;
				let render_window = in_args.get_render_window()?;
				let instance_data = effect.get_instance_data::<MyInstanceData>()?;

				let scale_value = instance_data.scale_param.get_value_at_time(time)?;

				let (sr, sg, sb, sa) = if instance_data.source_clip.get_components()?.is_rgb() {
					(
						instance_data.scale_r_param.get_value_at_time(time)?,
						instance_data.scale_g_param.get_value_at_time(time)?,
						instance_data.scale_b_param.get_value_at_time(time)?,
						instance_data.scale_a_param.get_value_at_time(time)?,
					)
				} else {
					(1., 1., 1., 1.)
				};
				if scale_value == 1. && sr == 1. && sg == 1. && sb == 1. && sa == 1. {
					out_args.set_name_raw(kOfxImageEffectSimpleSourceClipName)?;
					OK
				} else {
					REPLY_DEFAULT
				}
			}

			Action::GetRegionOfDefinition(ref mut effect, ref in_args, ref mut out_args) => {
				let time = in_args.get_time()?;
				let rod = effect
					.get_instance_data::<MyInstanceData>()?
					.source_clip
					.get_region_of_definition(time)?;
				out_args.set_region_of_definition(rod)?;

				OK
			}

			Action::GetRegionsOfInterest(ref mut effect, ref in_args, ref mut out_args) => {
				let roi = in_args.get_region_of_interest()?;

				out_args.set_raw("OfxImageClipPropRoI_Source", &roi)?;

				if effect
					.get_instance_data::<MyInstanceData>()?
					.is_general_effect
					&& effect.get_clip("Mask")?.get_connected()?
				{
					out_args.set_raw("OfxImageClipPropRoI_Mask", &roi)?;
				}

				OK
			}

			Action::GetClipPreferences(ref mut effect, ref mut out_args) => {
				let my_data = effect.get_instance_data::<MyInstanceData>()?;
				let bit_depth = my_data.source_clip.get_pixel_depth()?;
				let image_component = my_data.source_clip.get_components()?;
				let output_component = match image_component {
					ImageComponent::RGBA | ImageComponent::RGB => ImageComponent::RGBA,
					_ => ImageComponent::Alpha,
				};
				out_args.set_raw(
					"OfxImageClipPropComponents_Output",
					output_component.to_bytes(),
				)?;

				if self.host_supports_multiple_clip_depths {
					out_args.set_raw("OfxImageClipPropDepth_Output", bit_depth.to_bytes())?;
				}

				if my_data.is_general_effect {
					let is_mask_connected = my_data
						.mask_clip
						.as_ref()
						.and_then(|mask| mask.get_connected().ok())
						.unwrap_or_default();

					if is_mask_connected {
						out_args.set_raw(
							"OfxImageClipPropComponents_Mask",
							ImageComponent::Alpha.to_bytes(),
						)?;
						if self.host_supports_multiple_clip_depths {
							out_args.set_raw("OfxImageClipPropDepth_Mask", bit_depth.to_bytes())?;
						}
					}
				}

				OK
			}

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

				Self::set_per_component_scale_enabledness(effect)?;

				OK
			}

			Action::DestroyInstance(ref mut _effect) => OK,

			Action::DescribeInContext(ref mut effect, context) => {
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
				self.host_supports_multiple_clip_depths = plugin_context
					.get_host()
					.get_supports_multiple_clip_depths()?;

				let mut effect_properties = effect.properties()?;
				effect_properties.set_grouping("Ofx-rs")?;

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

			_ => REPLY_DEFAULT,
		}
	}
}

impl SimplePlugin {
	fn set_per_component_scale_enabledness(effect: &mut ImageEffectHandle) -> Result<()> {
		let per_component_scale = {
			let per_component_scale_selected = effect
				.get_instance_data::<MyInstanceData>()?
				.per_component_scale_param
				.get_value()?;
			let input_clip = effect.get_simple_input_clip()?;
			let is_rgb = input_clip.get_connected()?
				&& input_clip.get_components()? != ImageComponent::Alpha;
			per_component_scale_selected && is_rgb
		};

		let instance_data = effect.get_instance_data::<MyInstanceData>()?;
		instance_data
			.scale_r_param
			.set_enabled(per_component_scale)?;
		instance_data
			.scale_g_param
			.set_enabled(per_component_scale)?;
		instance_data
			.scale_b_param
			.set_enabled(per_component_scale)?;
		instance_data
			.scale_a_param
			.set_enabled(per_component_scale)?;

		// 		does this work too?
		//		for parameter in &["scaleR", "scaleG", "scaleB", "scaleA"] {
		//			effect
		//				.parameter_set()?
		//				.parameter::<Double>(parameter)?
		//				.set_enabled(per_component_scale)?;
		//		}

		Ok(())
	}
}
