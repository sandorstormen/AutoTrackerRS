
extern crate x11rb;
extern crate x11;
extern crate encoding;
use x11::xrecord::*;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use x11rb::protocol::*;
use std::time::*;
use x11rb::protocol::record::*;
use x11rb::protocol::Error::*;
use x11rb::protocol::Event::*;
use std::sync::mpsc;
use std::thread::*;
use std::cell::RefCell;
use lazy_static::*;
use std::sync::Mutex;
use std::path::Path;
use encoding::{Encoding, DecoderTrap};
use encoding::all::ISO_8859_1;
mod input;

#[allow(dead_code)]
fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}


fn main() {
    let channel_mutguard = input::CHANNEL.lock().unwrap();
    let (channel_send, _) = channel_mutguard.deref();
    let tx = channel_send.clone();
    drop(channel_mutguard);

    let (conn, screen_num) = x11rb::connect(None).unwrap();
    let screen = &conn.setup().roots[screen_num];
    let root_window = screen.root;

    let net_active_window_atom = conn.intern_atom(true, String::from("_NET_ACTIVE_WINDOW").as_bytes()).unwrap().reply().unwrap().atom;
    let net_wm_name_atom = conn.intern_atom(true, String::from("_NET_WM_NAME").as_bytes()).unwrap().reply().unwrap().atom;
    let utf8_atom = conn.intern_atom(true, String::from("UTF8_STRING").as_bytes()).unwrap().reply().unwrap().atom;
    let atom_reply: &[u8] = &conn.get_atom_name(utf8_atom).unwrap().reply().unwrap().name;
    println!("{}", String::from_utf8_lossy(atom_reply));
    let wm_name_atom = conn.intern_atom(true, String::from("WM_NAME").as_bytes()).unwrap().reply().unwrap().atom;
    
    let rec_ver = record::ConnectionExt::record_query_version(&conn, 0, 0).unwrap().reply().unwrap();
    eprintln!("X11 Xtst-package, record extension version: {}.{}", rec_ver.major_version, rec_ver.minor_version);

    match conn.change_window_attributes(root_window, &(ChangeWindowAttributesAux::default().event_mask(xproto::EventMask::PropertyChange | xproto::EventMask::KeyPress | xproto::EventMask::PointerMotion | xproto::EventMask::ButtonPress))) {
	Ok(res) => println!("attr changed"),
	Err(e) => println!("err {}", e)
    };
    match conn.flush() {
	Ok(res) => println!("No error"),
	Err(e) => println!("{}", e)
    };

    spawn(|| {
	input::read_input_thread();
    });

    let focus_window = xproto::get_input_focus(&conn).unwrap().reply().unwrap().focus;
    let res: &[u8] = &get_property(&conn, false, root_window, AtomEnum::WM_NAME, AtomEnum::STRING, 0, 64).unwrap().reply().unwrap().value;
    let name = String::from_utf8_lossy(res);
    match tx.send(String::from(name)) {
	Ok(_) => (),
	Err(error) => eprintln!("Can't get initial window: {}", error)
    };
    let mut legacy_name_print_out: bool = false;
    loop {
	match conn.wait_for_event() {
	    Ok(event) => {
		if let Event::PropertyNotify(event) = event {
		    if event.atom == net_active_window_atom {
			let focus_window = xproto::get_input_focus(&conn).unwrap().reply().unwrap().focus;
			// let res: &[u8] = &get_property(&conn, false, focus_window, wm_name_atom, AtomEnum::STRING, 0, 64).unwrap().reply().unwrap().value;
			// let name = String::from_utf8_lossy(res);
			// if !name.is_empty() {
			//     match tx.send(String::from(name)) {
			// 	Ok(_) => (),
			// 	Err(error) => eprintln!("Can't send window: {}", error)
			//     };
			// }
			let res_utf8: &[u8] = &get_property(&conn, false, focus_window, net_wm_name_atom, utf8_atom, 0, 64).unwrap().reply().unwrap().value;
			let name = match String::from_utf8(res_utf8.to_vec()) {
			    Ok(string) => string,
			    Err(_error) => {
				if !legacy_name_print_out {
				    eprintln!("No _NET_WM_NAME found for windows, defaulting to VM_NAME");
				    legacy_name_print_out = true;
				};
				let legacy_res: &[u8] = &get_property(&conn, false, focus_window, wm_name_atom, AtomEnum::STRING, 0, 64).unwrap().reply().unwrap().value;
				let legacy_name = match String::from_utf8(legacy_res.to_vec()) {
				    Ok(res) => res,
				    Err(_error) => {
					eprintln!("WM_NAME error for window, trying to decode in ISO-8859-1 ");
					String::from_utf8_lossy(legacy_res)
				    }.to_string()
				};
				legacy_name
			    }
			};
			if !name.is_empty() {
			    match tx.send(String::from(name)) {
				Ok(_) => (),
				Err(error) => eprintln!("Can't send window: {}", error)
			    };
			}
			
		    }
		    // NET_WM_ACTIVE or WM_ACTIVE
		}
	    },
	    Err(error) => eprintln!("Could not get X11 event: {}", error)
	}
    }
}

fn test_property(conn: &impl x11rb::connection::RequestConnection, focus_window: x11rb::protocol::xproto::Window, property: u32) {
    // println!("{}", String::from_utf8_lossy(&get_property(&conn, false, focus_window, property, AtomEnum::STRING, 0, 64).unwrap().reply().unwrap().value));
}

fn make_cntx(conn: &impl x11rb::connection::Connection, cntx: u32) {
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
    // let void_cookie = record::create_context(conn, cntx, record::HType::FromServerTime.into(), &[x11rb::protocol::record::CS::AllClients.into()], &[range_input]);
    let void_cookie = record::create_context(conn, cntx, 7, &[3], &[range_input]);
    match void_cookie.unwrap().check() {
	Ok(_res) => (),
	Err(error) => println!("Error when making context\n{}", error)
    };
    match record::enable_context(conn, cntx).unwrap().reply() {
	Ok(res) => println!("Worked! {}, {}, {}", res.xid_base, res.server_time, res.element_header),
	Err(e) => panic!("Could not create context\n{}", e)
    };
    let get_cntx_reply = record::get_context(conn, cntx).unwrap();
    let tid = Instant::now();
    match get_cntx_reply.reply() {
	Ok(res) => {
	    println!("Händer något? {}", tid.elapsed().as_secs());
	    println!("Enabled?: {}, Intercepted clients: {}", res.enabled, res.num_intercepted_clients());
	    let intercepted_clients = res.intercepted_clients;
	    for client in intercepted_clients {
		println!("num_ranges: {}, client_resource {}", client.num_ranges(), client.client_resource);
	    }
	    
	},
	Err(e) => println!("{}, {}", e, tid.elapsed().as_secs())
    };
    //record::ConnectionExt::record_free_context(conn, cntx).unwrap();
}

fn match_protocol_error(error: x11rb::protocol::Error) {
    match error {
	Access(_) => println!("AccessError"),
	Alloc(_) => println!("AllocError"),
	Atom(_) => println!("AtomError"),
	Colormap(_) => println!("ColormapError"),
	Cursor(_) => println!("Cursorerror"),
	Drawable(_)  => println!("Drawableerror"),
	Font(_)  => println!("Fonterror"),
	GContext(_) => println!("GContexterror"),
	IDChoice(_) => println!("IDchoiceerror"),
	Implementation(_) => println!("Implementationerror"),
	Length(_) => println!("Lengtherror"),
	Match(_) => println!("Matcherror"),
	Name(_) => println!("Nameerror"),
	Pixmap(_) => println!("Pixmaperror"),
	Request(_) => println!("Requesterror"),
	Value(_) => println!("Valueerror"),
	Window(_) => println!("Windowerror"),
	DamageBadDamage(_) => println!("BadDamageerror"),
	GlxBadContext(_) => println!("BadContexterror"),
	GlxBadContextState(_) => println!("Badcontextstateerror"),
	GlxBadContextTag(_) => println!("BadContexttagerror"),
	GlxBadCurrentDrawable(_) => println!("BadCurrentdrawableerror"),
	GlxBadCurrentWindow(_) => println!("BadWindowerror"),
	GlxBadDrawable(_) => println!("BadDamageerror"),
	GlxBadFBConfig(_) => println!("BadFBconfigerror"),
	GlxBadLargeRequest(_) => println!("BadLargerequesterror"),
	GlxBadPbuffer(_) => println!("BadPbuffererror"),
	GlxBadPixmap(_) => println!("BadPixmaperror"),
	GlxBadRenderRequest(_) => println!("BadRenderRequestError"),
	GlxBadWindow(_) => println!("RawWindowError"),
	GlxGLXBadProfileARB(_) => println!("GLXBadProfileARBError"),
	GlxUnsupportedPrivateRequest(_) => println!("UnsupportedPrivateRequestError"),
	RandrBadCrtc(_) => println!("BadCrtcError"),
	RandrBadMode(_) => println!("BadModeError"),
	RandrBadOutput(_) => println!("BadOutputError"),
	RandrBadProvider(_) => println!("BadProviderError"),
	RecordBadContext(_) => println!("BadContextError"),
	RenderGlyph(_) => println!("BadContextError"),
	RenderGlyphSet(_) => println!("GlyphSetError"),
	RenderPictFormat(_) => println!("PictFormatError"),
	RenderPictOp(_) => println!("PictOpError"),
	RenderPicture(_) => println!("PictureError"),
	ShmBadSeg(_) => println!("BadSegError"),
	SyncAlarm(_) => println!("AlarmError"),
	SyncCounter(_) => println!("CounterError"),
	Xf86vidmodeBadClock(_) => println!("BadClockError"),
	Xf86vidmodeBadHTimings(_) => println!("BadHTimingsError"),
	Xf86vidmodeBadVTimings(_) => println!("BadVTimingsError"),
	Xf86vidmodeClientNotLocal(_) => println!("ClientNotLocalError"),
	Xf86vidmodeExtensionDisabled(_) => println!("ExtensionDisabledError"),
	Xf86vidmodeModeUnsuitable(_) => println!("ModeUnsuitableError"),
	Xf86vidmodeZoomLocked(_) => println!("ZoomLockedError"),
	XfixesBadRegion(_) => println!("BadRegionError"),
	XinputClass(_) => println!("ClassError"),
	XinputDevice(_) => println!("DeviceError"),
	XinputDeviceBusy(_) => println!("DeviceBusyError"),
	XinputEvent(_) => println!("EventError"),
	XinputMode(_) => println!("ModeError"),
	XkbKeyboard(_) => println!("KeyboardError"),
	XprintBadContext(_) => println!("BadContextError"),
	XprintBadSequence(_) => println!("BadSequenceError"),
	XvBadControl(_) => println!("BadControlError"),
	XvBadEncoding(_) => println!("BadEncodingError"),
	XvBadPort(_) => println!("BadPortError"),
	_ => println!("umatched error")
    }
}

fn match_event(event: Event) {
    match event {
	Event::Unknown(_event) => println!("Unknown event"),
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
