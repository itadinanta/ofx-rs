#![feature(plugin)]

use action::*;
use enums::*;
use handle::*;
use ofx_sys::*;
use property::*;
use result::*;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fmt;
use std::fmt::Display;
use std::rc::Rc;
use suites::*;
use types::*;

pub struct ApiVersion(pub Int);
pub struct PluginVersion(pub UnsignedInt, pub UnsignedInt);

#[macro_export]
macro_rules! static_str (
	($name:expr) => { unsafe { CStr::from_bytes_with_nul_unchecked($name).as_ptr() } }
);

#[derive(Debug)]
struct EnumIndex<T>
where
	T: std::cmp::Eq + std::hash::Hash + Clone,
{
	map: HashMap<String, T>,
	inverse_map: HashMap<T, String>,
}

impl<T> EnumIndex<T>
where
	T: std::cmp::Eq + std::hash::Hash + Clone,
{
	pub fn new() -> EnumIndex<T> {
		EnumIndex {
			map: HashMap::new(),
			inverse_map: HashMap::new(),
		}
	}

	pub fn insert(&mut self, key_bytes: &[u8], value: T) {
		if let Ok(cstr) = CStr::from_bytes_with_nul(key_bytes) {
			if let Ok(key) = cstr.to_str() {
				self.map.insert(key.to_owned(), value.clone());
				self.inverse_map.insert(value, key.to_owned());
			}
		} else {
			error!("Was unable to add {:?} key, this is a bug", key_bytes)
		}
	}

	pub fn find(&self, c_key: &[u8]) -> Option<T> {
		let cstr = CString::new(c_key).ok()?;
		let key = cstr.into_string().ok()?;
		self.map.get(&key).cloned()
	}
}

#[derive(Debug)]
pub enum RawMessage {
	SetHost {
		host: OfxHost,
	},
	MainEntry {
		action: CharPtr,
		handle: VoidPtr,
		in_args: OfxPropertySetHandle,
		out_args: OfxPropertySetHandle,
	},
}

pub trait Dispatch {
	fn dispatch(&mut self, message: RawMessage) -> Result<Int> {
		OK
	}
}

pub trait Plugin: Dispatch + MapAction + Execute {
	fn suites(&self) -> &Suites;
}

pub struct PluginDescriptor {
	plugin_id: CString,
	module_name: String,
	plugin_index: usize,
	host: Option<OfxHost>,
	suites: Option<Suites>,
	cached_handle: Option<ImageEffectHandle>,
	instance: Box<Execute>,
	global_action_index: EnumIndex<GlobalAction>,
	image_effect_action_index: EnumIndex<ImageEffectAction>,
	ofx_plugin: OfxPlugin, // need an owned copy for the lifetime of the plugin
}

pub struct PluginContext {
	host: HostHandle,
	suites: Suites,
}

impl PluginContext {
	pub fn get_host(&self) -> HostHandle {
		self.host.clone()
	}
}

impl Display for PluginDescriptor {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"module:{} id:{:?} index:{}",
			self.module_name, self.plugin_id, self.plugin_index
		)
	}
}

impl MapAction for PluginDescriptor {
	fn map_action(
		&self,
		action: CharPtr,
		handle: VoidPtr,
		in_args: OfxPropertySetHandle,
		out_args: OfxPropertySetHandle,
	) -> Result<Action> {
		let name = unsafe { CStr::from_ptr(action) }.to_bytes();
		if let Some(action) = self.image_effect_action_index.find(name) {
			info!("Image effect action match {:?}", action);
			match action {
				ImageEffectAction::DescribeInContext => Ok(Action::DescribeInContext(
					self.new_image_effect_raw(handle)?,
					self.typed_properties(DescribeInContextInArgs::new, in_args)?,
				)),
				ImageEffectAction::GetRegionOfDefinition => Ok(Action::GetRegionOfDefinition(
					self.new_image_effect_raw(handle)?,
					self.typed_properties(GetRegionOfDefinitionInArgs::new, in_args)?,
					self.typed_properties(GetRegionOfDefinitionOutArgs::new, out_args)?,
				)),
				ImageEffectAction::GetRegionsOfInterest => Ok(Action::GetRegionsOfInterest(
					self.new_image_effect_raw(handle)?,
					self.typed_properties(GetRegionsOfInterestInArgs::new, in_args)?,
					self.typed_properties(GetRegionsOfInterestOutArgs::new, out_args)?,
				)),
				ImageEffectAction::IsIdentity => Ok(Action::IsIdentity(
					self.new_image_effect_raw(handle)?,
					self.typed_properties(IsIdentityInArgs::new, in_args)?,
					self.typed_properties(IsIdentityOutArgs::new, out_args)?,
				)),
				ImageEffectAction::GetClipPreferences => Ok(Action::GetClipPreferences(
					self.new_image_effect_raw(handle)?,
					self.typed_properties(GetClipPreferencesOutArgs::new, out_args)?,
				)),
				ImageEffectAction::GetTimeDomain => Ok(Action::GetTimeDomain(
					self.new_image_effect_raw(handle)?,
					self.typed_properties(GetTimeDomainOutArgs::new, out_args)?,
				)),
				ImageEffectAction::BeginSequenceRender => Ok(Action::BeginSequenceRender(
					self.new_image_effect_raw(handle)?,
					self.typed_properties(BeginSequenceRenderInArgs::new, in_args)?,
				)),
				ImageEffectAction::Render => Ok(Action::Render(
					self.new_image_effect_raw(handle)?,
					self.typed_properties(RenderInArgs::new, in_args)?,
				)),
				ImageEffectAction::EndSequenceRender => Ok(Action::EndSequenceRender(
					self.new_image_effect_raw(handle)?,
					self.typed_properties(EndSequenceRenderInArgs::new, in_args)?,
				)),

				_ => Err(Error::InvalidAction),
			}
		} else if let Some(action) = self.global_action_index.find(name) {
			info!("Global action {:?}", action);
			match action {
				GlobalAction::Load => Ok(Action::Load),
				GlobalAction::Unload => Ok(Action::Unload),
				GlobalAction::Describe => Ok(Action::Describe(self.new_image_effect_raw(handle)?)),
				GlobalAction::CreateInstance => {
					Ok(Action::CreateInstance(self.new_image_effect_raw(handle)?))
				}
				GlobalAction::BeginInstanceChanged => Ok(Action::BeginInstanceChanged(
					self.new_image_effect_raw(handle)?,
					self.typed_properties(BeginInstanceChangedInArgs::new, in_args)?,
				)),
				GlobalAction::InstanceChanged => Ok(Action::InstanceChanged(
					self.new_image_effect_raw(handle)?,
					self.typed_properties(InstanceChangedInArgs::new, in_args)?,
				)),

				GlobalAction::EndInstanceChanged => Ok(Action::EndInstanceChanged(
					self.new_image_effect_raw(handle)?,
					self.typed_properties(EndInstanceChangedInArgs::new, in_args)?,
				)),
				GlobalAction::DestroyInstance => {
					Ok(Action::DestroyInstance(self.new_image_effect_raw(handle)?))
				}
				_ => Err(Error::InvalidAction),
			}
		} else {
			info!("No action matching {:?}", unsafe { CStr::from_ptr(action) });
			Err(Error::InvalidAction)
		}
	}
}

impl Filter for PluginDescriptor {
	fn before_execute(&mut self, action: &Action) -> Result<Int> {
		match action {
			Action::Load => self.load(),
			Action::Unload => self.unload(),
			Action::Describe(ref handle) => self.describe(handle.clone()),
			_ => OK,
		}?;

		OK
	}

	fn after_execute(&mut self, context: &PluginContext, action: &mut Action) -> Result<Int> {
		match action {
			Action::DestroyInstance(ref mut effect) => effect.drop_instance_data(),
			_ => Ok(()),
		}?;

		OK
	}
}

impl Dispatch for PluginDescriptor {
	fn dispatch(&mut self, message: RawMessage) -> Result<Int> {
		match message {
			RawMessage::SetHost { host } => {
				self.host = Some(host);
				OK
			}
			RawMessage::MainEntry {
				action,
				handle,
				in_args,
				out_args,
			} => {
				let mut mapped_action = self.map_action(action, handle, in_args, out_args)?;

				debug!("Mapped action found: {:?}", mapped_action);
				self.before_execute(&mapped_action)?;

				if let (Some(host), Some(suites)) = (self.host, self.suites.clone()) {
					let plugin_context = PluginContext {
						host: HostHandle::new(host.host, suites.property()),
						suites: suites,
					};
					let status = self.execute(&plugin_context, &mut mapped_action);
					// TODO: do we call after even if execute fails?
					self.after_execute(&plugin_context, &mut mapped_action)?;
					status
				} else {
					OK
				}
			}
		}
	}
}

impl Execute for PluginDescriptor {
	fn execute(&mut self, context: &PluginContext, action: &mut Action) -> Result<Int> {
		let result = self.instance.execute(context, action);
		info!(
			"Executed {:?} of {} -> {:?}",
			action, self.module_name, result
		);
		result
	}
}

impl PluginDescriptor {
	pub(crate) fn new(
		plugin_index: usize,
		module_name: &'static str,
		name: &'static str,
		api_version: ApiVersion,
		plugin_version: PluginVersion,
		instance: Box<Execute>,
		set_host: SetHost,
		main_entry: MainEntry,
	) -> PluginDescriptor {
		let plugin_id = CString::new(name).unwrap();

		let ofx_plugin = OfxPlugin {
			pluginApi: static_str!(kOfxImageEffectPluginApi),
			apiVersion: api_version.0,
			pluginVersionMajor: plugin_version.0,
			pluginVersionMinor: plugin_version.1,
			pluginIdentifier: plugin_id.as_ptr(),
			setHost: Some(set_host),
			mainEntry: Some(main_entry),
		};

		let mut global_action_index = EnumIndex::new();
		let mut image_effect_action_index = EnumIndex::new();
		use ofx_sys::*;
		macro_rules! global_add {
			($id:ident) => {
				debug!(
					"kOfxAction{} GlobalAction::{}",
					stringify!($id),
					stringify!($id)
					);
				global_action_index.insert(concat_idents!(kOfxAction, $id), GlobalAction::$id)
			};
		}
		macro_rules! image_effect_add {
			($id:ident) => {
				debug!(
					"kOfxImageEffectAction{} ImageEffectAction::{}",
					stringify!($id),
					stringify!($id)
					);
				image_effect_action_index.insert(
					concat_idents!(kOfxImageEffectAction, $id),
					ImageEffectAction::$id,
					)
			};
		}

		global_add!(Load);
		global_add!(Describe);
		global_add!(Unload);
		global_add!(PurgeCaches);
		global_add!(SyncPrivateData);
		global_add!(CreateInstance);
		global_add!(DestroyInstance);
		global_add!(BeginInstanceChanged);
		global_add!(InstanceChanged);
		global_add!(EndInstanceChanged);
		global_add!(BeginInstanceEdit);
		global_add!(EndInstanceEdit);
		global_add!(Dialog);

		image_effect_add!(GetRegionOfDefinition);
		image_effect_add!(GetRegionsOfInterest);
		image_effect_add!(GetTimeDomain);
		image_effect_add!(GetFramesNeeded);
		image_effect_add!(GetClipPreferences);
		image_effect_add!(IsIdentity);
		image_effect_add!(BeginSequenceRender);
		image_effect_add!(Render);
		image_effect_add!(EndSequenceRender);
		image_effect_add!(DescribeInContext);
		image_effect_add!(GetInverseDistortion);
		image_effect_add!(InvokeHelp);
		image_effect_add!(InvokeAbout);
		image_effect_add!(VegasKeyframeUplift);

		PluginDescriptor {
			plugin_index,
			module_name: module_name.to_owned(),
			plugin_id,
			instance,
			host: None,
			suites: None,
			cached_handle: None,
			global_action_index,
			image_effect_action_index,
			ofx_plugin,
		}
	}

	fn suites(&self) -> Result<&Suites> {
		self.suites.as_ref().ok_or(Error::SuiteNotInitialized)
	}

	fn new_image_effect_raw(&self, ptr: VoidPtr) -> Result<ImageEffectHandle> {
		self.new_image_effect(unsafe { ptr as OfxImageEffectHandle })
	}

	fn new_image_effect(&self, handle: OfxImageEffectHandle) -> Result<ImageEffectHandle> {
		let suites = self.suites()?;
		let property_suite = suites.property();
		let image_effect_suite = suites.image_effect();
		let parameter_suite = suites.parameter();
		Ok(ImageEffectHandle::new(
			handle,
			property_suite,
			image_effect_suite,
			parameter_suite,
		))
	}

	fn typed_properties<T, F>(&self, constructor: F, handle: OfxPropertySetHandle) -> Result<T>
	where
		F: Fn(OfxPropertySetHandle, Rc<OfxPropertySuiteV1>) -> T,
	{
		let property_suite = self.suites()?.property();
		Ok(constructor(handle, property_suite))
	}

	fn load(&mut self) -> Result<Int> {
		let host = self.host.ok_or(Error::HostNotReady)?;
		let fetch_suite = host.fetchSuite.ok_or(Error::HostNotReady)?;

		const V1: Int = 1;
		const V2: Int = 2;

		info!("Fetching suites");
		macro_rules! fetch_suite {
			($suite_name:ident, $suite_version:ident) => {
				unsafe {
					let suiteptr = fetch_suite(
						host.host as OfxPropertySetHandle,
						CStr::from_bytes_with_nul_unchecked(concat_idents!(
							kOfx,
							$suite_name,
							Suite
						))
						.as_ptr(),
						$suite_version,
						);
					if suiteptr.is_null() {
						error!("Failed to load {}", stringify!($suite_name));
						None
					} else {
						info!(
							"Found suite '{}' at {:?}",
							stringify!($suite_name),
							suiteptr
						);
						unsafe {
							Some(*unsafe {
								suiteptr
									as *const concat_idents!(
										Ofx,
										$suite_name,
										Suite,
										$suite_version
									)
							})
							}
						}
					}
			};
		};

		self.suites = Some(Suites::new(
			fetch_suite!(ImageEffect, V1).ok_or(Error::InvalidSuite)?,
			fetch_suite!(Property, V1).ok_or(Error::InvalidSuite)?,
			fetch_suite!(Parameter, V1).ok_or(Error::InvalidSuite)?,
			fetch_suite!(Memory, V1).ok_or(Error::InvalidSuite)?,
			fetch_suite!(MultiThread, V1).ok_or(Error::InvalidSuite)?,
			fetch_suite!(Message, V1).ok_or(Error::InvalidSuite)?,
			fetch_suite!(Message, V2),
			fetch_suite!(Progress, V1).ok_or(Error::InvalidSuite)?,
			fetch_suite!(Progress, V2),
			fetch_suite!(TimeLine, V1).ok_or(Error::InvalidSuite)?,
			fetch_suite!(ParametricParameter, V1),
			fetch_suite!(ImageEffectOpenGLRender, V1),
		));
		info!("Loaded plugin");
		OK
	}

	fn unload(&mut self) -> Result<Int> {
		OK
	}

	fn cache_handle(&mut self, handle: ImageEffectHandle) {
		self.cached_handle = Some(handle);
	}

	fn describe(&mut self, handle: ImageEffectHandle) -> Result<Int> {
		info!("Caching plugin instance handle {:?}", handle);
		self.cache_handle(handle);
		OK
	}

	pub fn ofx_plugin(&self) -> &OfxPlugin {
		&self.ofx_plugin
	}
}

impl Plugin for PluginDescriptor {
	fn suites(&self) -> &Suites {
		&self.suites.as_ref().unwrap()
	}
}
