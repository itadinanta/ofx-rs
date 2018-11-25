use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fmt;
use std::fmt::Display;
use std::marker::PhantomData;

use handle::*;
use enums::*;
use ofx_sys::*;
use result::*;
use types::*;
use plugin::PluginContext;

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
pub enum Action {
	Load,
	Unload,
	Describe(ImageEffectHandle),
	DescribeInContext(ImageEffectHandle, ImageEffectContext),
	GenericGlobal(GlobalAction, GenericPluginHandle),
	GenericImageEffect(ImageEffectAction, ImageEffectHandle),
}

pub trait Execute {
	fn execute(&mut self, context: &PluginContext, action: &mut Action) -> Result<Int> {
		Ok(eOfxStatus_OK)
	}
}

pub trait MapAction {
	fn map_action(
		&self,
		action: CharPtr,
		handle: VoidPtr,
		in_args: OfxPropertySetHandle,
		out_args: OfxPropertySetHandle,
	) -> Result<Action>;
}
