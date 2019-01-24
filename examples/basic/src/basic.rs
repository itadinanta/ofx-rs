use ofx::*;
use std::sync::{Arc, Mutex};

plugin_module!(
	"net.itadinanta.ofx-rs.basic",
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
#[allow(unused)]
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

struct TileProcessor<'a, T, M>
where
	T: PixelFormat,
	M: PixelFormatAlpha,
{
	instance: ImageEffectHandle,
	scale: RGBAColourD,
	src: ImageDescriptor<'a, T>,
	dst: ImageTileMut<'a, T>,
	mask: Option<ImageDescriptor<'a, M>>,
	render_window: RectI,
}

// Members of the TileProcessor are either:
// - shared + read only
// - shareable handles and memory blocks from OFX (for which we can spray and pray)
// - owned by the tileprocessor
// so we can assume it can be processed across multiple threads even if rustc says no
//
unsafe impl<'a, T, M> Send for TileProcessor<'a, T, M>
where
	T: PixelFormat + ScaleMix,
	M: PixelFormatAlpha,
{
}

impl<'a, T, M> TileProcessor<'a, T, M>
where
	T: PixelFormat + ScaleMix,
	M: PixelFormatAlpha,
{
	fn new(
		instance: ImageEffectHandle,
		r_scale: Double,
		g_scale: Double,
		b_scale: Double,
		a_scale: Double,
		src: ImageDescriptor<'a, T>,
		dst: ImageTileMut<'a, T>,
		mask: Option<ImageDescriptor<'a, M>>,
		render_window: RectI,
	) -> Self {
		let scale = RGBAColourD {
			r: r_scale,
			g: g_scale,
			b: b_scale,
			a: a_scale,
		};
		TileProcessor {
			instance,
			scale,
			src,
			dst,
			mask,
			render_window,
		}
	}
}

struct TileDispatch<'a, T, M>
where
	T: PixelFormat + ScaleMix,
	M: PixelFormatAlpha,
{
	tiles: Arc<Mutex<Vec<TileProcessor<'a, T, M>>>>,
}

impl<'a, T, M> TileDispatch<'a, T, M>
where
	T: PixelFormat + ScaleMix,
	M: PixelFormatAlpha,
{
	fn new(tiles: Vec<TileProcessor<'a, T, M>>) -> Self {
		TileDispatch {
			tiles: Arc::new(Mutex::new(tiles)),
		}
	}

	fn pop(&mut self) -> Option<TileProcessor<'a, T, M>> {
		if let Ok(ref mut tiles) = self.tiles.lock() {
			tiles.pop()
		} else {
			None
		}
	}
}

impl<'a, T, M> Runnable for TileDispatch<'a, T, M>
where
	T: PixelFormat + ScaleMix,
	M: PixelFormatAlpha,
{
	fn run(&mut self, _thread_index: UnsignedInt, _thread_max: UnsignedInt) {
		while let Some(mut tile) = self.pop() {
			if tile.do_processing().is_err() {
				break;
			};
		}
	}
}

trait ProcessRGBA<'a, T, M> {
	fn do_processing(&'a mut self) -> Result<()>;
}

impl<'a, T, M> ProcessRGBA<'a, T, M> for TileProcessor<'a, T, M>
where
	T: PixelFormat + ScaleMix,
	M: PixelFormatAlpha,
{
	fn do_processing(&'a mut self) -> Result<()> {
		let scale = self.scale;
		let proc_window = self.render_window;
		for y in self.dst.y1.max(proc_window.y1)..self.dst.y2.min(proc_window.y2) {
			let dst_row = self
				.dst
				.row_range_as_slice(proc_window.x1, proc_window.x2, y);

			let src_row = self
				.src
				.row_range_as_slice(proc_window.x1, proc_window.x2, y);

			if self.instance.abort()? {
				break;
			}

			let src_mask = self
				.mask
				.as_ref()
				.map(|mask| mask.row_range_as_slice(proc_window.x1, proc_window.x2, y));

			match src_mask {
				None => {
					for (dst, src) in dst_row.iter_mut().zip(src_row.iter()) {
						*dst = src.scaled(&scale);
					}
				}
				Some(src_mask) => {
					for ((dst, src), mask) in dst_row.iter_mut().zip(src_row.iter()).zip(src_mask) {
						let mask0 = mask.to_f32();
						*dst = src.mix(&src.scaled(&scale), mask0);
					}
				}
			}
		}

		Ok(())
	}
}

const PARAM_MAIN_NAME: &str = "Main";
const PARAM_SCALE_NAME: &str = "scale";
const PARAM_SCALE_R_NAME: &str = "scaleR";
const PARAM_SCALE_G_NAME: &str = "scaleG";
const PARAM_SCALE_B_NAME: &str = "scaleB";
const PARAM_SCALE_A_NAME: &str = "scaleA";
const PARAM_SCALE_COMPONENTS_NAME: &str = "scaleComponents";
const PARAM_COMPONENT_SCALES_NAME: &str = "componentScales";

impl Execute for SimplePlugin {
	#[allow(clippy::float_cmp)]
	fn execute(&mut self, plugin_context: &PluginContext, action: &mut Action) -> Result<Int> {
		use Action::*;
		match *action {
			Render(ref mut effect, ref in_args) => {
				let time = in_args.get_time()?;
				// TODO: what happens if render_window < full size?
				let render_window = in_args.get_render_window()?;
				let instance_data: &mut MyInstanceData = effect.get_instance_data()?;

				let source_image = instance_data.source_clip.get_image(time)?;
				let output_image = instance_data.output_clip.get_image_mut(time)?;
				let mask_image = match instance_data.mask_clip {
					None => None,
					Some(ref mask_clip) => {
						if instance_data.is_general_effect && mask_clip.get_connected()? {
							Some(mask_clip.get_image(time)?)
						} else {
							None
						}
					}
				};

				let (sv, sr, sg, sb, sa) = instance_data.get_scale_components(time)?;
				let (r_scale, g_scale, b_scale, a_scale) = (sv * sr, sv * sg, sv * sb, sv * sa);
				let mut output_image = output_image.borrow_mut();
				let num_threads = plugin_context.num_threads()?;
				let num_tiles = num_threads as usize;
				macro_rules! tiles {
					($rgba_format:ty, $mask_format:ty) => {{
						output_image
							.get_tiles_mut::<$rgba_format>(num_tiles)?
							.into_iter()
							.map(|tile| {
								let src = source_image.get_descriptor::<$rgba_format>().unwrap();
								let mask = mask_image
									.as_ref()
									.and_then(|mask| mask.get_descriptor::<$mask_format>().ok());
								TileProcessor::new(
									effect.clone(),
									r_scale,
									g_scale,
									b_scale,
									a_scale,
									src,
									tile,
									mask,
									render_window,
								)
							})
						}};
				}

				macro_rules! process_tiles {
					($rgba_format:ty, $mask_format:ty) => {{
						let mut queue =
							TileDispatch::new(tiles!($rgba_format, $mask_format).collect());
						plugin_context.run_in_threads(num_threads, &mut queue)?;
						}};
				}
				match (
					output_image.get_pixel_depth()?,
					output_image.get_components()?,
				) {
					(BitDepth::Float, ImageComponent::RGBA) => process_tiles!(RGBAColourF, f32),
					(BitDepth::Byte, ImageComponent::RGBA) => process_tiles!(RGBAColourB, u8),
					(BitDepth::Short, ImageComponent::RGBA) => process_tiles!(RGBAColourS, u16),
					(BitDepth::Float, ImageComponent::Alpha) => process_tiles!(f32, f32),
					(BitDepth::Byte, ImageComponent::Alpha) => process_tiles!(u8, u8),
					(BitDepth::Short, ImageComponent::Alpha) => process_tiles!(u16, u16),
					(_, _) => return FAILED,
				}

				if effect.abort()? {
					FAILED
				} else {
					OK
				}
			}

			IsIdentity(ref mut effect, ref in_args, ref mut out_args) => {
				let time = in_args.get_time()?;
				let _render_window = in_args.get_render_window()?;
				let instance_data: &MyInstanceData = effect.get_instance_data()?;

				let (scale_value, sr, sg, sb, sa) = instance_data.get_scale_components(time)?;

				if scale_value == 1. && sr == 1. && sg == 1. && sb == 1. && sa == 1. {
					out_args.set_name(&image_effect_simple_source_clip_name())?;
					OK
				} else {
					REPLY_DEFAULT
				}
			}

			InstanceChanged(ref mut effect, ref in_args) => {
				if in_args.get_change_reason()? == Change::UserEdited {
					let obj_changed = in_args.get_name()?;
					let expected = match in_args.get_type()? {
						Type::Clip => Some(image_effect_simple_source_clip_name()),
						Type::Parameter => Some(PARAM_SCALE_COMPONENTS_NAME.to_owned()),
						_ => None,
					};

					if expected == Some(obj_changed) {
						Self::set_per_component_scale_enabledness(effect)?;
						OK
					} else {
						REPLY_DEFAULT
					}
				} else {
					REPLY_DEFAULT
				}
			}

			GetRegionOfDefinition(ref mut effect, ref in_args, ref mut out_args) => {
				let time = in_args.get_time()?;
				let rod = effect
					.get_instance_data::<MyInstanceData>()?
					.source_clip
					.get_region_of_definition(time)?;
				out_args.set_effect_region_of_definition(rod)?;

				OK
			}

			GetRegionsOfInterest(ref mut effect, ref in_args, ref mut out_args) => {
				let roi = in_args.get_region_of_interest()?;

				out_args.set_raw(image_clip_prop_roi!(clip_source!()), &roi)?;

				if effect
					.get_instance_data::<MyInstanceData>()?
					.is_general_effect
					&& effect.get_clip(clip_mask!())?.get_connected()?
				{
					out_args.set_raw(image_clip_prop_roi!(clip_mask!()), &roi)?;
				}

				OK
			}

			GetTimeDomain(ref mut effect, ref mut out_args) => {
				let my_data: &MyInstanceData = effect.get_instance_data()?;
				out_args.set_frame_range(my_data.source_clip.get_frame_range()?)?;

				OK
			}

			GetClipPreferences(ref mut effect, ref mut out_args) => {
				let my_data: &MyInstanceData = effect.get_instance_data()?;
				let bit_depth = my_data.source_clip.get_pixel_depth()?;
				let image_component = my_data.source_clip.get_components()?;
				let output_component = match image_component {
					ImageComponent::RGBA | ImageComponent::RGB => ImageComponent::RGBA,
					_ => ImageComponent::Alpha,
				};
				out_args.set_raw(
					image_clip_prop_components!(clip_output!()),
					output_component.to_bytes(),
				)?;

				if self.host_supports_multiple_clip_depths {
					out_args
						.set_raw(image_clip_prop_depth!(clip_output!()), bit_depth.to_bytes())?;
				}

				if my_data.is_general_effect {
					let is_mask_connected = my_data
						.mask_clip
						.as_ref()
						.and_then(|mask| mask.get_connected().ok())
						.unwrap_or_default();

					if is_mask_connected {
						out_args.set_raw(
							image_clip_prop_components!(clip_mask!()),
							ImageComponent::Alpha.to_bytes(),
						)?;
						if self.host_supports_multiple_clip_depths {
							out_args.set_raw(
								image_clip_prop_depth!(clip_mask!()),
								bit_depth.to_bytes(),
							)?;
						}
					}
				}

				OK
			}

			CreateInstance(ref mut effect) => {
				let mut effect_props: ImageEffectProperties = effect.properties()?;
				let mut param_set = effect.parameter_set()?;

				let is_general_effect = effect_props.get_context()?.is_general();
				let per_component_scale_param = param_set.parameter(PARAM_SCALE_COMPONENTS_NAME)?;

				let source_clip = effect.get_simple_input_clip()?;
				let output_clip = effect.get_output_clip()?;
				let mask_clip = if is_general_effect {
					Some(effect.get_clip(clip_mask!())?)
				} else {
					None
				};

				let scale_param = param_set.parameter(PARAM_SCALE_NAME)?;
				let scale_r_param = param_set.parameter(PARAM_SCALE_R_NAME)?;
				let scale_g_param = param_set.parameter(PARAM_SCALE_G_NAME)?;
				let scale_b_param = param_set.parameter(PARAM_SCALE_B_NAME)?;
				let scale_a_param = param_set.parameter(PARAM_SCALE_A_NAME)?;

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

			DestroyInstance(ref mut _effect) => OK,

			DescribeInContext(ref mut effect, ref in_args) => {
				let mut output_clip = effect.new_output_clip()?;
				output_clip
					.set_supported_components(&[ImageComponent::RGBA, ImageComponent::Alpha])?;

				let mut input_clip = effect.new_simple_input_clip()?;
				input_clip
					.set_supported_components(&[ImageComponent::RGBA, ImageComponent::Alpha])?;

				if in_args.get_context()?.is_general() {
					let mut mask = effect.new_clip(clip_mask!())?;
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
					PARAM_SCALE_NAME,
					"scale",
					PARAM_SCALE_NAME,
					"Scales all component in the image",
					None,
				)?;

				let mut param_props =
					param_set.param_define_boolean(PARAM_SCALE_COMPONENTS_NAME)?;
				param_props.set_default(false)?;
				param_props.set_hint("Enables scale on individual components")?;
				param_props.set_script_name(PARAM_SCALE_COMPONENTS_NAME)?;
				param_props.set_label("Scale Individual Components")?;

				let mut param_props = param_set.param_define_group(PARAM_COMPONENT_SCALES_NAME)?;
				param_props.set_hint("Scales on the individual component")?;
				param_props.set_label("Components")?;

				define_scale_param(
					&mut param_set,
					PARAM_SCALE_R_NAME,
					"red",
					PARAM_SCALE_R_NAME,
					"Scales the red component of the image",
					Some(PARAM_COMPONENT_SCALES_NAME),
				)?;
				define_scale_param(
					&mut param_set,
					PARAM_SCALE_G_NAME,
					"green",
					PARAM_SCALE_G_NAME,
					"Scales the green component of the image",
					Some(PARAM_COMPONENT_SCALES_NAME),
				)?;
				define_scale_param(
					&mut param_set,
					PARAM_SCALE_B_NAME,
					"blue",
					PARAM_SCALE_B_NAME,
					"Scales the blue component of the image",
					Some(PARAM_COMPONENT_SCALES_NAME),
				)?;
				define_scale_param(
					&mut param_set,
					PARAM_SCALE_A_NAME,
					"alpha",
					PARAM_SCALE_A_NAME,
					"Scales the alpha component of the image",
					Some(PARAM_COMPONENT_SCALES_NAME),
				)?;

				param_set
					.param_define_page(PARAM_MAIN_NAME)?
					.set_children(&[
						PARAM_SCALE_NAME,
						PARAM_SCALE_COMPONENTS_NAME,
						PARAM_SCALE_R_NAME,
						PARAM_SCALE_G_NAME,
						PARAM_SCALE_B_NAME,
						PARAM_SCALE_A_NAME,
					])?;

				OK
			}

			Describe(ref mut effect) => {
				self.host_supports_multiple_clip_depths = plugin_context
					.get_host()
					.get_supports_multiple_clip_depths()?;

				let mut effect_properties: EffectDescriptorProperties = effect.properties()?;
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
		let instance_data: &mut MyInstanceData = effect.get_instance_data()?;
		let input_clip = effect.get_simple_input_clip()?;
		let is_input_rgb = input_clip.get_connected()? && input_clip.get_components()?.is_rgb();
		instance_data
			.per_component_scale_param
			.set_enabled(is_input_rgb)?;
		let per_component_scale =
			is_input_rgb && instance_data.per_component_scale_param.get_value()?;
		for scale_param in &mut [
			&mut instance_data.scale_r_param,
			&mut instance_data.scale_g_param,
			&mut instance_data.scale_b_param,
			&mut instance_data.scale_a_param,
		] {
			scale_param.set_enabled(per_component_scale)?;
			instance_data
				.scale_param
				.set_enabled(!per_component_scale)?
		}

		Ok(())
	}
}

impl MyInstanceData {
	fn get_scale_components(&self, time: Time) -> Result<(f64, f64, f64, f64, f64)> {
		let scale_value = self.scale_param.get_value_at_time(time)?;
		let per_component_scale = self.per_component_scale_param.get_value_at_time(time)?;
		if per_component_scale && self.source_clip.get_components()?.is_rgb() {
			Ok((
				scale_value,
				self.scale_r_param.get_value_at_time(time)?,
				self.scale_g_param.get_value_at_time(time)?,
				self.scale_b_param.get_value_at_time(time)?,
				self.scale_a_param.get_value_at_time(time)?,
			))
		} else {
			Ok((scale_value, 1., 1., 1., 1.))
		}
	}
}
