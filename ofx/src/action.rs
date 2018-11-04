use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fmt;
use std::fmt::Display;
use std::marker::PhantomData;

use ofx_sys::*;
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
