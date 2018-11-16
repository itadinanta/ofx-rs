enum eOfxStatus {
	Unused = -1,
	OK = kOfxStatOK,
	Failed = kOfxStatFailed,
	ErrFatal = kOfxStatErrFatal,
	ErrBadHandle = kOfxStatErrBadHandle,
	ErrBadIndex = kOfxStatErrBadIndex,
	ErrValue = kOfxStatErrValue,
	ErrUnknown = kOfxStatErrUnknown,
	ErrMemory = kOfxStatErrMemory,
	ErrUnsupported = kOfxStatErrUnsupported,
	ErrMissingHostFeature = kOfxStatErrMissingHostFeature,
};

#define kOfxImageEffectOpenGLRenderSuite "OfxImageEffectOpenGLRenderSuite"
