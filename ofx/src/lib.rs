#![allow(unused)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![feature(concat_idents)]

extern crate ofx_sys;

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fmt;
use std::fmt::Display;
use std::marker::PhantomData;

mod result;
mod types;
mod prop;

pub use ofx_sys::*;
pub use result::*;
pub use types::*;

pub struct Suites {
	effect: &'static OfxImageEffectSuiteV1,
	prop: &'static OfxPropertySuiteV1,
	param: &'static OfxParameterSuiteV1,
	memory: &'static OfxMemorySuiteV1,
	thread: &'static OfxMultiThreadSuiteV1,
	message: &'static OfxMessageSuiteV1,
	message_v2: Option<&'static OfxMessageSuiteV2>,
	progress: &'static OfxProgressSuiteV1,
	progress_v2: Option<&'static OfxProgressSuiteV2>,
	time_line: &'static OfxTimeLineSuiteV1,
	parametric_parameter: Option<&'static OfxParametricParameterSuiteV1>,
	opengl_render: Option<&'static OfxImageEffectOpenGLRenderSuiteV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum GlobalAction {
	Load,
	Describe,
	Unload,
	PurgeCaches,
	SyncPrivateData,
	CreateInstance,
	DestroyInstance,
	InstanceChanged,
	BeginInstanceChanged,
	EndInstanceChanged,
	BeginInstanceEdit,
	EndInstanceEdit,
	//	DescribeInteract,
	//	CreateInstanceInteract,
	//	DestroyInstanceInteract,
	Dialog,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ImageEffectAction {
	GetRegionOfDefinition,
	GetRegionsOfInterest,
	GetTimeDomain,
	GetFramesNeeded,
	GetClipPreferences,
	IsIdentity,
	Render,
	BeginSequenceRender,
	EndSequenceRender,
	DescribeInContext,
	GetInverseDistortion,
	InvokeHelp,
	InvokeAbout,
	VegasKeyframeUplift,
}

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
			println!("Was unable to add {:?} key, this is a bug", key_bytes)
		}
	}

	pub fn find(&self, c_key: &[u8]) -> Option<T> {
		let cstr = CString::new(c_key).ok()?;
		let key = cstr.into_string().ok()?;
		self.map.get(&key).cloned()
	}
}

#[derive(Clone, Copy, Debug)]
pub struct GenericPluginHandle<'a> {
	inner: VoidPtr,
	_lifetime: PhantomData<&'a Void>,
}



#[derive(Clone, Copy, Debug)]
pub struct ImageEffectHandle<'a> {
	inner: OfxImageEffectHandle,
	_lifetime: PhantomData<&'a OfxImageEffectHandle>,
}

impl<'a> ImageEffectHandle<'a> {
	pub fn new(ptr: VoidPtr) -> ImageEffectHandle<'a> {
		ImageEffectHandle {
			inner: unsafe { ptr as OfxImageEffectHandle },
			_lifetime: PhantomData,
		}
	}
}

#[derive(Clone, Copy)]
pub struct ImageEffectInstanceHandle<'a> {
	inner: VoidPtr,
	_lifetime: PhantomData<&'a Void>,
}

#[derive(Debug)]
pub enum Action<'a> {
	Load,
	Unload,
	Describe(ImageEffectHandle<'a>),
	GenericGlobal(GlobalAction, GenericPluginHandle<'a>),
	GenericImageEffect(ImageEffectAction, ImageEffectHandle<'a>),
}

pub trait Dispatch {
	fn dispatch(&mut self, message: RawMessage) -> Result<Int> {
		Ok(eOfxStatus_OK)
	}
}

pub trait Execute {
	fn execute(&mut self, action: Action) -> Result<Int> {
		Ok(eOfxStatus_OK)
	}
}

pub trait MapAction {
	fn map_action<'a>(
		&self,
		action: CharPtr,
		handle: VoidPtr,
		in_args: OfxPropertySetHandle,
		out_args: OfxPropertySetHandle,
	) -> Result<Action<'a>>;
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
	cached_handle: Option<ImageEffectHandle<'static>>,
	instance: Box<Execute>,
	global_action_index: EnumIndex<GlobalAction>,
	image_effect_action_index: EnumIndex<ImageEffectAction>,
	ofx_plugin: OfxPlugin, // need an owned copy for the lifetime of the plugin
}

impl Display for PluginDescriptor {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"{:?} {} {}",
			self.plugin_id, self.module_name, self.plugin_index
		)
	}
}

impl MapAction for PluginDescriptor {
	fn map_action<'a>(
		&self,
		action: CharPtr,
		handle: VoidPtr,
		in_args: OfxPropertySetHandle,
		out_args: OfxPropertySetHandle,
	) -> Result<Action<'a>> {
		let name = unsafe { CStr::from_ptr(action) }.to_bytes();
		if let Some(action) = self.image_effect_action_index.find(name) {
			println!("Image effect action match {:?}", action);
			match action {
				_ => Err(Error::InvalidAction),
			}
		} else if let Some(action) = self.global_action_index.find(name) {
			println!("Global action {:?}", action);
			match action {
				GlobalAction::Load => Ok(Action::Load),
				GlobalAction::Unload => Ok(Action::Unload),
				GlobalAction::Describe => Ok(Action::Describe(ImageEffectHandle::new(handle))),
				_ => Err(Error::InvalidAction),
			}
		} else {
			println!("No action matching");
			Err(Error::InvalidAction)
		}
	}
}

impl Execute for PluginDescriptor {}

impl Dispatch for PluginDescriptor {
	fn dispatch(&mut self, message: RawMessage) -> Result<Int> {
		match message {
			RawMessage::SetHost { host } => {
				self.host = Some(host.clone());
				Ok(0)
			}
			RawMessage::MainEntry {
				action,
				handle,
				in_args,
				out_args,
			} => {
				let mapped_action = self.map_action(action, handle, in_args, out_args);
				println!("Mapped action found: {:?}", mapped_action);
				match mapped_action {
					Ok(Action::Load) => self.load(),
					Ok(Action::Unload) => self.unload(),
					Ok(Action::Describe(handle)) => self.describe(handle),
					//Ok(Action::DescribeInContext(handle)) => self.describe(handle),
					Ok(a) => self.execute(a),
					Err(e) => Err(e),
				}
			}
		}
	}
}

impl PluginDescriptor {
	fn load(&mut self) -> Result<Int> {
		let host = self.host.ok_or(Error::HostNotReady)?;
		let fetchSuite = host.fetchSuite.ok_or(Error::HostNotReady)?;

		const V1: Int = 1;
		const V2: Int = 2;

		println!("Fetching suites");
		macro_rules! fetch_suite {
			($suite_name:ident, $suite_version:ident) => {
				unsafe {
					let suiteptr = fetchSuite(
						host.host as OfxPropertySetHandle,
						CStr::from_bytes_with_nul_unchecked(concat_idents!(
							kOfx,
							$suite_name,
							Suite
						))
						.as_ptr(),
						$suite_version,
						);
					if suiteptr == std::ptr::null() {
						println!("Failed to load {}", stringify!($suite_name));
						None
					} else {
						println!("Found {} at {:?}", stringify!($suite_name), suiteptr);
						unsafe {
							Some(&*unsafe {
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

		let suites = Suites {
			effect: fetch_suite!(ImageEffect, V1).ok_or(Error::InvalidSuite)?,
			prop: fetch_suite!(Property, V1).ok_or(Error::InvalidSuite)?,
			param: fetch_suite!(Parameter, V1).ok_or(Error::InvalidSuite)?,
			memory: fetch_suite!(Memory, V1).ok_or(Error::InvalidSuite)?,
			thread: fetch_suite!(MultiThread, V1).ok_or(Error::InvalidSuite)?,
			message: fetch_suite!(Message, V1).ok_or(Error::InvalidSuite)?,
			message_v2: fetch_suite!(Message, V2),
			progress: fetch_suite!(Progress, V1).ok_or(Error::InvalidSuite)?,
			progress_v2: fetch_suite!(Progress, V2),

			time_line: fetch_suite!(TimeLine, V1).ok_or(Error::InvalidSuite)?,
			parametric_parameter: fetch_suite!(ParametricParameter, V1),
			opengl_render: fetch_suite!(ImageEffectOpenGLRender, V1),
		};
		self.suites = Some(suites);
		println!("Loaded plugin");
		Ok(eOfxStatus_OK)
	}

	fn unload(&mut self) -> Result<Int> {
		Ok(eOfxStatus_OK)
	}

	fn cache_handle(&mut self, handle: ImageEffectHandle<'static>) {
		self.cached_handle = Some(handle);
	}

	fn describe(&mut self, handle: ImageEffectHandle<'static>) -> Result<Int> {
		println!("Caching plugin instance handle {:?}", handle);
		self.cache_handle(handle);
		Ok(eOfxStatus_OK)
	}
}

impl Plugin for PluginDescriptor {
	fn suites(&self) -> &Suites {
		&self.suites.as_ref().unwrap()
	}
}

pub struct ApiVersion(pub Int);
pub struct PluginVersion(pub UnsignedInt, pub UnsignedInt);

pub struct Registry {
	plugins: Vec<PluginDescriptor>,
	plugin_modules: HashMap<String, usize>,
}

#[derive(Debug)]
pub enum RawMessage<'a> {
	SetHost {
		host: &'a OfxHost,
	},
	MainEntry {
		action: CharPtr,
		handle: VoidPtr,
		in_args: OfxPropertySetHandle,
		out_args: OfxPropertySetHandle,
	},
}

impl Registry {
	pub fn new() -> Registry {
		Registry {
			plugin_modules: HashMap::new(),
			plugins: Vec::new(),
		}
	}

	pub fn add(
		&mut self,
		module_name: &'static str,
		name: &'static str,
		api_version: ApiVersion,
		plugin_version: PluginVersion,
		instance: Box<Execute>,
		set_host: SetHost,
		main_entry: MainEntry,
	) -> usize {
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

		let plugin_index = self.plugins.len();
		let module_name = module_name.to_owned();
		self.plugin_modules
			.insert(module_name.clone(), plugin_index as usize);

		let mut global_action_index = EnumIndex::new();
		let mut image_effect_action_index = EnumIndex::new();
		use ofx_sys::*;
		macro_rules! global_add {
			($id:ident) => {
				println!(
					"{} {}",
					stringify!(concat_idents!(kOfxAction, $id)),
					stringify!(GlobalAction::$id)
					);
				global_action_index.insert(concat_idents!(kOfxAction, $id), GlobalAction::$id)
			};
		}
		global_add!(Load);
		global_add!(Describe);
		global_add!(Unload);
		global_add!(PurgeCaches);
		global_add!(SyncPrivateData);
		global_add!(CreateInstance);
		global_add!(DestroyInstance);
		global_add!(InstanceChanged);
		global_add!(BeginInstanceChanged);
		global_add!(EndInstanceChanged);
		global_add!(BeginInstanceEdit);
		global_add!(EndInstanceEdit);
		//global_add!(DescribeInteract);
		//global_add!(CreateInstanceInteract);
		//global_add!(DestroyInstanceInteract);
		global_add!(Dialog);
		println!("{:?}", global_action_index);
		macro_rules! image_effect_add {
			($id:ident) => {
				image_effect_action_index.insert(
					concat_idents!(kOfxImageEffectAction, $id),
					ImageEffectAction::$id,
					)
			};
		}
		image_effect_add!(GetRegionOfDefinition);
		image_effect_add!(GetRegionsOfInterest);
		image_effect_add!(GetTimeDomain);
		image_effect_add!(GetFramesNeeded);
		image_effect_add!(GetClipPreferences);
		image_effect_add!(IsIdentity);
		image_effect_add!(Render);
		image_effect_add!(BeginSequenceRender);
		image_effect_add!(EndSequenceRender);
		image_effect_add!(DescribeInContext);
		image_effect_add!(GetInverseDistortion);
		image_effect_add!(InvokeHelp);
		image_effect_add!(InvokeAbout);
		image_effect_add!(VegasKeyframeUplift);

		let plugin = PluginDescriptor {
			plugin_index,
			module_name,
			plugin_id,
			instance,
			host: None,
			suites: None,
			cached_handle: None,
			global_action_index,
			image_effect_action_index,
			ofx_plugin,
		};

		self.plugins.push(plugin);
		plugin_index
	}

	pub fn count(&self) -> Int {
		self.plugins.len() as Int
	}

	pub fn get_plugin_mut(&mut self, index: usize) -> &mut PluginDescriptor {
		&mut self.plugins[index as usize]
	}

	pub fn get_plugin(&self, index: usize) -> &PluginDescriptor {
		&self.plugins[index as usize]
	}

	pub fn ofx_plugin(&'static self, index: Int) -> &'static OfxPlugin {
		&self.plugins[index as usize].ofx_plugin
	}

	pub fn dispatch(&mut self, plugin_module: &str, message: RawMessage) -> Result<Int> {
		println!("{}:{:?}", plugin_module, message);
		let found_plugin = self.plugin_modules.get(plugin_module).cloned();
		if let Some(plugin_index) = found_plugin {
			let plugin = self.get_plugin_mut(plugin_index);
			plugin.dispatch(message)
		} else {
			Err(Error::PluginNotFound)
		}
	}
}

pub fn set_host_for_plugin(plugin_module: &str, host: *mut OfxHost) {
	unsafe {
		get_registry_mut()
			.dispatch(plugin_module, RawMessage::SetHost { host: &*host })
			.ok();
	}
}

static mut global_registry: Option<Registry> = None;

pub fn main_entry_for_plugin(
	plugin_module: &str,
	action: CharPtr,
	handle: VoidPtr,
	in_args: OfxPropertySetHandle,
	out_args: OfxPropertySetHandle,
) -> Int {
	unsafe {
		get_registry_mut()
			.dispatch(
				plugin_module,
				RawMessage::MainEntry {
					action,
					handle,
					in_args,
					out_args,
				},
			)
			.ok()
			.unwrap_or(-1)
	}
}

pub fn init_registry<F>(init_function: F)
where
	F: Fn(&mut Registry),
{
	unsafe {
		if global_registry.is_none() {
			let mut registry = Registry::new();
			init_function(&mut registry);
			global_registry = Some(registry);
		}
	}
}

fn get_registry_mut() -> &'static mut Registry {
	unsafe { global_registry.as_mut().unwrap() }
}

pub fn get_registry() -> &'static Registry {
	unsafe { global_registry.as_ref().unwrap() }
}

#[macro_export]
macro_rules! static_str (
	($name:expr) => { unsafe { CStr::from_bytes_with_nul_unchecked($name).as_ptr() } }
);

#[macro_export]
macro_rules! plugin_module {
	($name:expr, $api_version:expr, $plugin_version:expr, $factory:expr) => {
		pub fn name() -> &'static str {
			$name
		}

		pub fn module_name() -> &'static str {
			module_path!().split("::").last().as_ref().unwrap()
		}

		pub fn new_instance() -> Box<Execute> {
			Box::new($factory())
		}

		pub fn api_version() -> ApiVersion {
			$api_version
		}

		pub fn plugin_version() -> PluginVersion {
			$plugin_version
		}

		pub extern "C" fn set_host(host: *mut ofx::OfxHost) {
			ofx::set_host_for_plugin(module_name(), host)
		}

		pub extern "C" fn main_entry(
			action: ofx::CharPtr,
			handle: ofx::VoidPtr,
			in_args: ofx::OfxPropertySetHandle,
			out_args: ofx::OfxPropertySetHandle,
		) -> super::Int {
			ofx::main_entry_for_plugin(module_name(), action, handle, in_args, out_args)
		}
	};
}

#[macro_export]
macro_rules! register_plugin {
	($registry:ident, $module:ident) => {
		$registry.add(
			$module::module_name(),
			$module::name(),
			$module::api_version(),
			$module::plugin_version(),
			$module::new_instance(),
			$module::set_host,
			$module::main_entry,
			);
	};
}

#[macro_export]
macro_rules! build_plugin_registry {
	($init_callback:ident) => {
		fn init() {
			init_registry($init_callback);
		}

		#[no_mangle]
		pub extern "C" fn OfxGetNumberOfPlugins() -> Int {
			init();
			get_registry().count()
		}

		#[no_mangle]
		pub extern "C" fn OfxGetPlugin(nth: Int) -> *const OfxPlugin {
			init();
			get_registry().ofx_plugin(nth) as *const OfxPlugin
		}

		pub fn describe_plugins() -> Vec<String> {
			unsafe {
				let n = OfxGetNumberOfPlugins();
				for i in 0..n {
					OfxGetPlugin(i);
				}
				(0..n)
					.map(|i| {
						let plugin = get_registry().get_plugin(i as usize);
						format!("{}", plugin)
					})
					.collect()
			}
		}
	};
}

#[macro_export]
macro_rules! register_modules {
	( $ ($module:ident), *) => {
		fn register_plugins(registry: &mut ofx::Registry) {
			$(register_plugin!(registry, $module);
			)*
		}

		build_plugin_registry!(register_plugins);
	};
}
