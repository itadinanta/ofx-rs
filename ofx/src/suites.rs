use ofx_sys::*;
use result::*;
use std::borrow::Borrow;
use std::rc::Rc;

#[derive(Clone)]
pub struct Suites {
	image_effect: Rc<OfxImageEffectSuiteV1>,
	property: Rc<OfxPropertySuiteV1>,
	parameter: Rc<OfxParameterSuiteV1>,
	memory: Rc<OfxMemorySuiteV1>,
	multi_thread: Rc<OfxMultiThreadSuiteV1>,
	message: Rc<OfxMessageSuiteV1>,
	message_v2: Option<Rc<OfxMessageSuiteV2>>,
	progress: Rc<OfxProgressSuiteV1>,
	progress_v2: Option<Rc<OfxProgressSuiteV2>>,
	time_line: Rc<OfxTimeLineSuiteV1>,
	parametric_parameter: Option<Rc<OfxParametricParameterSuiteV1>>,
	image_effect_opengl_render: Option<Rc<OfxImageEffectOpenGLRenderSuiteV1>>,
}

macro_rules! suite_call {
	($function:ident in $suite:expr, $($arg:expr),*) => {
		unsafe { ($suite).$function.ok_or(Error::SuiteNotInitialized)?($($arg),*) }
	};
}

impl Suites {
	pub fn new(
		image_effect: OfxImageEffectSuiteV1,
		property: OfxPropertySuiteV1,
		parameter: OfxParameterSuiteV1,
		memory: OfxMemorySuiteV1,
		multi_thread: OfxMultiThreadSuiteV1,
		message: OfxMessageSuiteV1,
		message_v2: Option<OfxMessageSuiteV2>,
		progress: OfxProgressSuiteV1,
		progress_v2: Option<OfxProgressSuiteV2>,
		time_line: OfxTimeLineSuiteV1,
		parametric_parameter: Option<OfxParametricParameterSuiteV1>,
		image_effect_opengl_render: Option<OfxImageEffectOpenGLRenderSuiteV1>,
	) -> Self {
		Suites {
			image_effect: Rc::new(image_effect),
			property: Rc::new(property),
			parameter: Rc::new(parameter),
			memory: Rc::new(memory),
			multi_thread: Rc::new(multi_thread),
			message: Rc::new(message),
			message_v2: message_v2.map(Rc::new),
			progress: Rc::new(progress),
			progress_v2: progress_v2.map(Rc::new),
			time_line: Rc::new(time_line),
			parametric_parameter: parametric_parameter.map(Rc::new),
			image_effect_opengl_render: image_effect_opengl_render.map(Rc::new),
		}
	}

	pub fn image_effect(&self) -> Rc<OfxImageEffectSuiteV1> {
		self.image_effect.clone()
	}

	pub fn property(&self) -> Rc<OfxPropertySuiteV1> {
		self.property.clone()
	}

	pub fn parameter(&self) -> Rc<OfxParameterSuiteV1> {
		self.parameter.clone()
	}
}
