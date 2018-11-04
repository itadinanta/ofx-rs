use action::*;
use ofx_sys::*;
use plugin::*;
use result::*;
use std::collections::HashMap;
use types::*;

pub struct Registry {
	plugins: Vec<PluginDescriptor>,
	plugin_modules: HashMap<String, usize>,
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
		let plugin_index = self.plugins.len();

		self.plugin_modules
			.insert(module_name.to_owned(), plugin_index as usize);

		let plugin = PluginDescriptor::new(
			plugin_index,
			module_name,
			name,
			api_version,
			plugin_version,
			instance,
			set_host,
			main_entry,
		);

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
		&self.plugins[index as usize].ofx_plugin()
	}

	pub fn dispatch(&mut self, plugin_module: &str, message: RawMessage) -> Result<Int> {
		info!("{}:{:?}", plugin_module, message);
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

static mut _GLOBAL_REGISTRY: Option<Registry> = None;

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
		if _GLOBAL_REGISTRY.is_none() {
			use log4rs::append::console::*;
			use log4rs::config::*;

			let config = Config::builder()
				.appender(Appender::builder().build(
					"stdout".to_string(),
					Box::new(ConsoleAppender::builder().build()),
				))
				.logger(Logger::builder().build("ofx".to_string(), log::LevelFilter::Debug))
				.build(
					Root::builder()
						.appender("stdout".to_string())
						.build(log::LevelFilter::Error),
				);
			log4rs::init_config(config.unwrap()).unwrap();

			let mut registry = Registry::new();
			init_function(&mut registry);
			for plugin in &registry.plugins {
				info!("Registered plugin {}", plugin);
			}
			_GLOBAL_REGISTRY = Some(registry);
		}
	}
}

fn get_registry_mut() -> &'static mut Registry {
	unsafe { _GLOBAL_REGISTRY.as_mut().unwrap() }
}

pub fn get_registry() -> &'static Registry {
	unsafe { _GLOBAL_REGISTRY.as_ref().unwrap() }
}

#[macro_export]
macro_rules! plugin_module {
	($name:expr, $api_version:expr, $plugin_version:expr, $factory:expr) => {
		pub fn name() -> &'static str {
			$name
		}

		pub fn module_name() -> &'static str {
			module_path!()//.split("::").last().as_ref().unwrap()
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
