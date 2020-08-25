extern crate x11rb;
use std::error::Error;
use x11rb::connection::{Connection, SequenceNumber};
use x11rb::errors::{ConnectionError, ReplyError, ReplyOrIdError};
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use x11rb::wrapper::ConnectionExt as _;
use x11rb::COPY_DEPTH_FROM_PARENT;
use x11rb::protocol::*;
use x11rb::protocol::xinput::*;
use x11rb::x11_utils::TryParse;
use std::process::exit;
use std::time::*;
use x11rb::protocol::record::*;
use x11rb::protocol::Error::*;
use x11rb::protocol::Event::*;
use device_query::{DeviceQuery, DeviceState, MouseState, Keycode};
use std::sync::mpsc;
use std::thread::*;

fn main() {
    let (input_conn, screen_num) = x11rb::connect(None).unwrap();
    let screen = &input_conn.setup().roots[screen_num];
    let root_window = screen.root;
    let cntx = input_conn.generate_id().unwrap();
    let range_input = x11rb::protocol::record::Range {
	core_requests: Range8 {
	    first: 0,
	    last: 0
	},
	core_replies: Range8 {
	    first: 0,
	    last: 0
	},
	ext_requests: ExtRange {
	    major: Range8 {
		first: 0,
		last: 0
	    },
	    minor: Range16 {
		first: 0,
		last: 0
	    }
	},
	ext_replies: ExtRange {
	    major: Range8 {
		first: 0,
		last: 0
	    },
	    minor: Range16 {
		first: 0,
		last: 0
	    }
	},
	delivered_events: Range8 {
	    first: 0,
	    last: 0
	},
	device_events: Range8 {
	    first: 2,
	    last: 36
	},
	errors: Range8 {
	    first: 0,
	    last: 0
	},
	client_started: false,
	client_died: false
    };
    let void_cookie = record::create_context(&input_conn, cntx, record::HType::FromServerTime.into(), &[x11rb::protocol::record::CS::AllClients.into()], &[range_input]);
    // let void_cookie = record::create_context(&input_conn, cntx, 7, &[3], &[range_input]);
    match void_cookie.unwrap().check() {
	Ok(_res) => (),
	Err(error) => println!("Error when making context\n{}", error)
    };
    match record::enable_context(&input_conn, cntx).unwrap().reply() {
	Ok(res) => println!("Worked! {}, {}, {}", res.xid_base, res.server_time, res.element_header),
	Err(e) => panic!("Could not create context\n{}", e)
    };
    // let get_cntx_reply = record::get_context(&input_conn, cntx).unwrap();
    // let tid = Instant::now();
    // match get_cntx_reply.reply() {
    // 	Ok(res) => {
    // 	    println!("Händer något? {}", tid.elapsed().as_secs());
    // 	    println!("Enabled?: {}, Intercepted clients: {}", res.enabled, res.num_intercepted_clients());
    // 	    let intercepted_clients = res.intercepted_clients;
    // 	    for client in intercepted_clients {
    // 		println!("num_ranges: {}, client_resource {}", client.num_ranges(), client.client_resource);
    // 	    }	    
    // 	},
    // 	Err(e) => println!("{}, {}", e, tid.elapsed().as_secs())
    // };
    //record::ConnectionExt::record_free_context(input_conn, cntx).unwrap();
    record::register_clients(&input_conn, cntx, record::HType::FromServerTime.into(), &[x11rb::protocol::record::CS::AllClients.into()], &[range_input]).unwrap().check().unwrap();
    loop {
	println!("Looping");
	let event = input_conn.wait_for_event().unwrap();
	match event {
	    Event::ButtonPress(event) => println!("Input event"),
	    Event::KeyPress(event) => println!("Input event"),
	    Event::MotionNotify(event) => println!("Input event"),
	    Event::XinputButtonPress(event) => println!("Input event"),
	    Event::XinputDeviceButtonPress(event) => println!("Input event"),
	    Event::XinputDeviceKeyPress(event) => println!("Input event"),
	    Event::XinputKeyPress(event) => println!("Input event"),
	    Event::XinputMotion(event) => println!("Input event"),
	    Event::XinputDeviceMotionNotify(event) => println!("Input event"),
	    Event::Error(error) => match_protocol_error(error),
	    _ => match_event(event)
	}
    }
}

fn match_event(event: Event) {
    match event {
	Event::Unknown(res) => println!("Unknown event"),
	Error(_event) => println!("Error"),
	ButtonPress(_event) => println!("Button press event"),
	ButtonRelease(_event) => println!("ButtonRelease event"),
	CirculateNotify(_event) => println!("CirculateNotify event"),
	CirculateRequest(_event) => println!("CirculateRequest event"),
	ClientMessage(_event) => println!("ClientMessage event"),
	ColormapNotify(_event) => println!("ColormapNotify event"),
	ConfigureNotify(_event) => println!("ConfigureNotify event"),
	ConfigureRequest(_event) => println!("ConfigureRequest event"),
	CreateNotify(_event) => println!("CreateNotify event"),
	DestroyNotify(_event) => println!("DestroyNotify event"),
	EnterNotify(_event) => println!("EnterNotify event"),
	Expose(_event) => println!("Expose event"),
	FocusIn(_event) => println!("FocusIn event"),
	FocusOut(_event) => println!("FocusOut event"),
	GeGeneric(_event) => println!("GeGeneric event"),
	GraphicsExposure(_event) => println!("GraphicsExposure event"),
	GravityNotify(_event) => println!("GravityNotify event"),
	KeyPress(_event) => println!("KeyPress event"),
	KeyRelease(_event) => println!("KeyRelease event"),
	KeymapNotify(_event) => println!("KeymapNotify event"),
	LeaveNotify(_event) => println!("LeaveNotify event"),
	MapNotify(_event) => println!("MapNotify event"),
	MapRequest(_event) => println!("MapRequest event"),
	MappingNotify(_event) => println!("MappingNotify event"),
	MotionNotify(_event) => println!("MotionNotify event"),
	NoExposure(_event) => println!("NoExposure event"),
	PropertyNotify(_event) => println!("PropertyNotify event"),
	ReparentNotify(_event) => println!("ReparentNotify event"),
	ResizeRequest(_event) => println!("ResizeRequest event"),
	SelectionClear(_event) => println!("SelectionClear event"),
	SelectionNotify(_event) => println!("SelectionNotify event"),
	SelectionRequest(_event) => println!("SelectionRequest event"),
	UnmapNotify(_event) => println!("UnmapNotify event"),
	VisibilityNotify(_event) => println!("VisibilityNotify event"),
	DamageNotify(_event) => println!("DamageNotify event"),
	Dri2BufferSwapComplete(_event) => println!("Dri2BufferSwapComplete event"),
	Dri2InvalidateBuffers(_event) => println!("Dri2InvalidateBuffers event"),
	GlxBufferSwapComplete(_event) => println!("GlxBufferSwapComplete event"),
	GlxPbufferClobber(_event) => println!("GlxPbufferClobber event"),
	PresentCompleteNotify(_event) => println!("PresentCompleteNotify event"),
	PresentConfigureNotify(_event) => println!("PresentCompleteNotify event"),
	PresentGeneric(_event) => println!("PresentGeneric event"),
	PresentIdleNotify(_event) => println!("PresentIdleNotify event"),
	PresentRedirectNotify(_event) => println!("PresentRedirectNotify event"),
	RandrNotify(_event) => println!("RandrNotify event"),
	RandrScreenChangeNotify(_event) => println!("RandrScreenChangeNotify event"),
	ScreensaverNotify(_event) => println!("ScreensaverNotify event"),
	ShapeNotify(_event) => println!("ShapeNotify event"),
	ShmCompletion(_event) => println!("ShmCompletion event"),
	SyncAlarmNotify(_event) => println!("SyncAlarmNotify event"),
	_ => println!("Some other event")
    }
}
fn match_protocol_error(error: x11rb::protocol::Error) {
    match error {
	Access(AccessError) => println!("AccessError"),
	Alloc(AllocError) => println!("AllocError"),
	Atom(AtomError) => println!("AtomError"),
	Colormap(ColormapError) => println!("ColormapError"),
	Cursor(CursorError) => println!("Cursorerror"),
	Drawable(DrawableError)  => println!("Drawableerror"),
	Font(FontError)  => println!("Fonterror"),
	GContext(GContextError) => println!("GContexterror"),
	IDChoice(IDChoiceError) => println!("IDchoiceerror"),
	Implementation(ImplementationError) => println!("Implementationerror"),
	Length(LengthError) => println!("Lengtherror"),
	Match(MatchError) => println!("Matcherror"),
	Name(NameError) => println!("Nameerror"),
	Pixmap(PixmapError) => println!("Pixmaperror"),
	Request(RequestError) => println!("Requesterror"),
	Value(ValueError) => println!("Valueerror"),
	Window(WindowError) => println!("Windowerror"),
	DamageBadDamage(BadDamageError) => println!("BadDamageerror"),
	GlxBadContext(BadContextError) => println!("BadContexterror"),
	GlxBadContextState(BadContextStateError) => println!("Badcontextstateerror"),
	GlxBadContextTag(BadContextTagError) => println!("BadContexttagerror"),
	GlxBadCurrentDrawable(BadCurrentDrawableError) => println!("BadCurrentdrawableerror"),
	GlxBadCurrentWindow(BadCurrentWindowError) => println!("BadWindowerror"),
	GlxBadDrawable(BadDrawableError) => println!("BadDamageerror"),
	GlxBadFBConfig(BadFBConfigError) => println!("BadFBconfigerror"),
	GlxBadLargeRequest(BadLargeRequestError) => println!("BadLargerequesterror"),
	GlxBadPbuffer(BadPbufferError) => println!("BadPbuffererror"),
	GlxBadPixmap(BadPixmapError) => println!("BadPixmaperror"),
	_ => println!("umatched error")
	/*
	GlxBadRenderRequest(BadRenderRequestError) => println!(""),
	GlxBadWindow(BadWindowError) => println!(""),
	GlxGLXBadProfileARB(GLXBadProfileARBError) => println!(""),
	GlxUnsupportedPrivateRequest(UnsupportedPrivateRequestError) => println!(""),
	RandrBadCrtc(BadCrtcError) => println!(""),
	RandrBadMode(BadModeError) => println!(""),
	RandrBadOutput(BadOutputError) => println!(""),
	RandrBadProvider(BadProviderError) => println!(""),
	RecordBadContext(BadContextError) => println!(""),
	RenderGlyph(GlyphError) => println!(""),
	RenderGlyphSet(GlyphSetError) => println!(""),
	RenderPictFormat(PictFormatError) => println!(""),
	RenderPictOp(PictOpError) => println!(""),
	RenderPicture(PictureError) => println!(""),
	ShmBadSeg(BadSegError) => println!(""),
	SyncAlarm(AlarmError) => println!(""),
	SyncCounter(CounterError) => println!(""),
	Xf86vidmodeBadClock(BadClockError) => println!(""),
	Xf86vidmodeBadHTimings(BadHTimingsError) => println!(""),
	Xf86vidmodeBadVTimings(BadVTimingsError) => println!(""),
	Xf86vidmodeClientNotLocal(ClientNotLocalError) => println!(""),
	Xf86vidmodeExtensionDisabled(ExtensionDisabledError) => println!(""),
	Xf86vidmodeModeUnsuitable(ModeUnsuitableError) => println!(""),
	Xf86vidmodeZoomLocked(ZoomLockedError) => println!(""),
	XfixesBadRegion(BadRegionError) => println!(""),
	XinputClass(ClassError) => println!(""),
	XinputDevice(DeviceError) => println!(""),
	XinputDeviceBusy(DeviceBusyError) => println!(""),
	XinputEvent(EventError) => println!(""),
	XinputMode(ModeError) => println!(""),
	XkbKeyboard(KeyboardError) => println!(""),
	XprintBadContext(BadContextError) => println!(""),
	XprintBadSequence(BadSequenceError) => println!(""),
	XvBadControl(BadControlError) => println!(""),
	XvBadEncoding(BadEncodingError) => println!(""),
	XvBadPort(BadPortError) => println!("")*/
    }
}
