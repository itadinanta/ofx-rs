use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fmt;
use std::fmt::Display;
use std::marker::PhantomData;

use enums::*;
use handle::*;
use ofx_sys::*;
use plugin::PluginContext;
use result::*;
use types::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum GlobalAction {
	Load,
	Describe,
	Unload,
	PurgeCaches,
	SyncPrivateData,
	CreateInstance,
	DestroyInstance,
	BeginInstanceChanged,
	InstanceChanged,
	EndInstanceChanged,
	BeginInstanceEdit,
	InstanceEdit,
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
	DescribeInContext(ImageEffectHandle, DescribeInContextInArgs),

	CreateInstance(ImageEffectHandle),
	DestroyInstance(ImageEffectHandle),

	GetRegionOfDefinition(
		ImageEffectHandle,
		GetRegionOfDefinitionInArgs,
		GetRegionOfDefinitionOutArgs,
	),
	GetRegionsOfInterest(
		ImageEffectHandle,
		GetRegionsOfInterestInArgs,
		GetRegionsOfInterestOutArgs,
	),

	BeginInstanceChanged(ImageEffectHandle, BeginInstanceChangedInArgs),
	InstanceChanged(ImageEffectHandle, InstanceChangedInArgs),
	EndInstanceChanged(ImageEffectHandle, EndInstanceChangedInArgs),

	BeginSequenceRender(ImageEffectHandle, BeginSequenceRenderInArgs),
	Render(ImageEffectHandle, RenderInArgs),
	EndSequenceRender(ImageEffectHandle, EndSequenceRenderInArgs),

	GetClipPreferences(ImageEffectHandle, GetClipPreferencesOutArgs),
	GetTimeDomain(ImageEffectHandle, GetTimeDomainOutArgs),
	IsIdentity(ImageEffectHandle, IsIdentityInArgs, IsIdentityOutArgs),
	GenericGlobal(GlobalAction, GenericPluginHandle),
	GenericImageEffect(ImageEffectAction, ImageEffectHandle),
}

pub trait Execute {
	fn execute(&mut self, context: &PluginContext, action: &mut Action) -> Result<Int> {
		Ok(eOfxStatus_OK)
	}
}

pub trait Filter {
	fn before_execute(&mut self, action: &Action) -> Result<Int>;
	fn after_execute(&mut self, context: &PluginContext, action: &mut Action) -> Result<Int>;
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
