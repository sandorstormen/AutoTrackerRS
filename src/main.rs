
extern crate x11rb;
extern crate x11;
use x11::xrecord::*;
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
use std::sync::mpsc;
use std::thread::*;
use std::cell::RefCell;
use lazy_static::*;
use std::sync::Mutex;
use std::thread;
use std::time::*;

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}


lazy_static! {
    static ref CHANNEL: Mutex<(std::sync::mpsc::Sender<String>, std::sync::mpsc::Receiver<String>)> = Mutex::new(mpsc::channel());
}

fn main() {
    let channel_mutguard = CHANNEL.lock().unwrap();
    let (channel_send, _) = channel_mutguard.deref();
    let tx = channel_send.clone();
    drop(channel_mutguard);

    let (conn, screen_num) = x11rb::connect(None).unwrap();
    let screen = &conn.setup().roots[screen_num];
    let root_window = screen.root;

    let net_active_atom = conn.intern_atom(true, String::from("_NET_ACTIVE_WINDOW").as_bytes()).unwrap().reply().unwrap().atom;
    let net_wm_atom = conn.intern_atom(true, String::from("_NET_WM_NAME").as_bytes()).unwrap().reply().unwrap().atom;
    let wm_atom = conn.intern_atom(true, String::from("WM_NAME").as_bytes()).unwrap().reply().unwrap().atom;
    
    let rec_ver = record::ConnectionExt::record_query_version(&conn, 0, 0).unwrap().reply().unwrap();
    println!("{} {}", rec_ver.major_version, rec_ver.minor_version);
    
    // std::thread::sleep(Duration::from_secs(15));
    //let values = ChangeWindowAttributesAux::default().event_mask(xproto::EventMask::KeyPress | xproto::EventMask::ButtonPress | xproto::EventMask::PointerMotion | xproto::EventMask::StructureNotify | xproto::EventMask::SubstructureNotify | xproto::EventMask::FocusChange | xproto::EventMask::PropertyChange);
    match conn.change_window_attributes(root_window, &(ChangeWindowAttributesAux::default().event_mask(xproto::EventMask::PropertyChange | xproto::EventMask::KeyPress | xproto::EventMask::PointerMotion | xproto::EventMask::ButtonPress))) {
	Ok(res) => println!("attr changed"),
	Err(e) => println!("err {}", e)
    };
    match conn.flush() {
	Ok(res) => println!("No error"),
	Err(e) => println!("{}", e)
    };

    spawn(|| {
	read_input_thread();
    });

    let focus_window = xproto::get_input_focus(&conn).unwrap().reply().unwrap().focus;
    let res: &[u8] = &get_property(&conn, false, root_window, AtomEnum::WM_NAME, AtomEnum::STRING, 0, 64).unwrap().reply().unwrap().value;
    let name = String::from_utf8_lossy(res);
    tx.send(String::from(name));
    
    // present::select_input(&conn, ); //Titta nogrannre
    // device_mask.push(xproto::EventMask::KeyPress);
    // device_mask.push(xproto::EventMask::PointerMotion);
    // device_mask.push(xproto::EventMask::ButtonPress);
    // let devices = xinput::list_input_devices(&conn).unwrap().reply().unwrap();
    // let mut device_ids = Vec::new();
    // let mut device_names = std::collections::HashMap::new();
    // for id in devices.devices {
    // 	device_ids.push(id.device_id);
    // };
    // for i in 0..devices.names.len()-1 {
    // 	let convert: &[u8] = &devices.names[i].name;
    // 	device_names.insert(device_ids[i], String::from_utf8_lossy(convert));
    // }
    // for id in device_ids {
    // 	match xinput::open_device(&conn, id).unwrap().reply() {
    // 	    Ok(_res) => (),
    // 	    Err(_err) => match device_names.get(&id) {
    // 		Some(name) => println!("Error with device: {}", name),
    // 		None => println!("Error with device of unknown name")
    // 	    }
    // 	};
    // };
    // let mut prev_val = 0;
    // // 512 - 3839
    // for i in 0..5000 {
    // 	match xinput::select_extension_event(&conn, root_window, &[i]).unwrap().check() {
    // 	    Ok(res) => (),
    // 	    Err(e) => ()
    // 	}
    // }
    
    
    // let mut focus_mask = Vec::new();
    // focus_mask.push(XIEventMask::Hierarchy.into());
    // let focus_in_mask = x11rb::protocol::xinput::EventMask{
    //  	deviceid: 0,
    //  	mask: focus_mask
    // };
    // xinput::xi_select_events(&conn, root_window, &[focus_in_mask]);
    //x11rb::protocol::xproto::change_property(&conn, x11rb::protocol::xproto::PropMode::Replace, root_window, x11rb::protocol::present::EventMask::CompleteNotify, x11rb::protocol::xproto::Atom, 32, 1, x11rb::protocol::Event::PropertyNotify);
    // x11rb::protocol::present::select_input(&conn, x11rb::protocol::present::COMPLETE_NOTIFY_EVENT as u32, root_window, x11rb::protocol::Event::PropertyNotify);
    // let a: u32 = 0;
    // let b: u32 = 5;
    // let mot_ev = xproto::get_motion_events(&conn, root_window, a, b).unwrap().reply().unwrap();
    // for i in mot_ev.events {
    // 	println!("{}, {}, {}", i.time, i.x, i.y);
    // }
    loop {
	match conn.wait_for_event() {
	    Ok(event) => match event {
		Event::PropertyNotify(event) => {
		    if event.atom == net_active_atom {
			let focus_window = xproto::get_input_focus(&conn).unwrap().reply().unwrap().focus;
			let res: &[u8] = &get_property(&conn, false, focus_window, AtomEnum::WM_NAME, AtomEnum::STRING, 0, 64).unwrap().reply().unwrap().value;
			let name = String::from_utf8_lossy(res);
			if !name.is_empty() {
			    tx.send(String::from(name));
			}
		    }
		    else if event.atom == net_wm_atom || event.atom == wm_atom {
			println!("NET_WM_ACTIVE or WM_ACTIVE");
		    }
		},
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
		Event::ClientMessage(event) => println!("ClientMessage"),
		_ => println!("Other event")
	    },
	    Err(e) => panic!(e)
	}
    }
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

fn read_input_thread() {
    // let (input_conn, screen_num) = x11rb::connect(None).unwrap();
    // let screen = &input_conn.setup().roots[screen_num];
    // let root_window = screen.root;

    set_focused_window();
    
    let c_char: *const std::os::raw::c_char = &mut 0;
    let input_display: *mut x11::xlib::Display;
    unsafe {
	input_display = x11::xlib::XOpenDisplay(c_char) as *mut x11::xlib::Display;
    }
    
    let mut rr;
    let mut rcs: XRecordClientSpec;
    let rc: XRecordContext;
    unsafe {
	rr = XRecordAllocRange() as *mut XRecordRange;
	(*rr).device_events.first = 2;
	(*rr).device_events.last = 6;
    };
    rcs = x11::xrecord::XRecordAllClients;

    unsafe {
	rc = XRecordCreateContext(input_display, 0, &mut rcs, 1, &mut rr, 1);
    };
    unsafe {
	XRecordEnableContext( input_display, rc, Some(input_event_handler), &mut 0);
    };
}

fn set_focused_window() {
    match x11rb::connect(None) {
	Ok(res) => {
	    let (conn, _screen_num) = res;
	    match x11rb::protocol::xproto::get_input_focus(&conn) {
		Ok(res) => {
		    match res.reply() {
			Ok(res) => {
			    let res: &[u8] = &get_property(&conn, false, res.focus, AtomEnum::WM_NAME, AtomEnum::STRING, 0, 64).unwrap().reply().unwrap().value;
			    let name = String::from_utf8_lossy(res);
			    PREV_TITLE.with(|prev_title| {
				CURRENT_TITLE.with(|current_title|{
				    (*prev_title.borrow_mut()) = String::from(name.clone());
				    (*current_title.borrow_mut()) = String::from(name.clone());
				});
			    });
			},
			Err(e) => {
			    PREV_TITLE.with(|prev_title| {
				CURRENT_TITLE.with(|current_title|{
				    (*prev_title.borrow_mut()) = String::from("<uninitialized window>");
				    (*current_title.borrow_mut()) = String::from("<uninitialized window>");
				});
			    });
			}
		    }
		},
		Err(e) => {
		    PREV_TITLE.with(|prev_title| {
			CURRENT_TITLE.with(|current_title|{
			    (*prev_title.borrow_mut()) = String::from("<uninitialized window>");
			    (*current_title.borrow_mut()) = String::from("<uninitialized window>");
			});
		    });
		}
	    };
	},
	Err(e) => {
	    PREV_TITLE.with(|prev_title| {
		CURRENT_TITLE.with(|current_title|{
		    (*prev_title.borrow_mut()) = String::from("<uninitialized window>");
		    (*current_title.borrow_mut()) = String::from("<uninitialized window>");
		});
	    });
	}
    };
}

#[allow(non_upper_case_globals)]
unsafe extern "C" fn input_event_handler( _c_char: *mut std::os::raw::c_char, hook: *mut XRecordInterceptData) {
    match (*hook).category {
	XRecordFromServer => (),
	_ => return
    };
    let channel_mutguard = CHANNEL.lock().unwrap();
    let (_, rx) = channel_mutguard.deref();

    PREV_TITLE.with(|prev_title| {
	CURRENT_TITLE.with(|current_title| {
	    WINDOW_TIMES.with(|window_times| {
		START_DATE.with(|start_date| {
		    END_DATE.with(|end_date| {
			PREV_COUNTER.with(|prev_counter| {
			    match rx.try_recv() {
				Ok(result) => *(current_title.borrow_mut()) = result,
				Err(_error) => ()
			    }
			    if *prev_title.borrow() == *current_title.borrow() {
				// println!("{}", *prev_title.borrow());
				if (*prev_counter.borrow()).elapsed().as_secs() > 10 {
				    let window_title = current_title.borrow().clone();
				    let tuple = ( window_title, (*start_date.borrow()).to_rfc3339(), ((*start_date.borrow()) + chrono::Duration::seconds(10)).to_rfc3339() );
				    (*window_times.borrow_mut()).push(tuple);
				    (*prev_counter.borrow_mut()) = Instant::now();
				    (*start_date.borrow_mut()) = chrono::offset::Local::now();
				}
				else {
				    (*prev_counter.borrow_mut()) = Instant::now();
				}
			    }
			    else if *prev_title.borrow() != *current_title.borrow() {
				//println!("{}, {}", *prev_title.borrow(), *current_title.borrow());
				if (*prev_counter.borrow()).elapsed().as_secs() > 10 {
				    let window_title = prev_title.borrow().clone();
				    let tuple = ( window_title, (*start_date.borrow()).to_rfc3339(), ((*start_date.borrow()) + chrono::Duration::seconds(10)).to_rfc3339() );
				    (*window_times.borrow_mut()).push(tuple);
				    (*prev_counter.borrow_mut()) = Instant::now();
				    (*start_date.borrow_mut()) = chrono::offset::Local::now();
				}
				else {
				    let window_title = prev_title.borrow().clone();
				    let tuple = ( window_title, (*start_date.borrow()).to_rfc3339(), chrono::offset::Local::now().to_rfc3339() );
				    (*window_times.borrow_mut()).push(tuple);
				    (*prev_counter.borrow_mut()) = Instant::now();
				    (*start_date.borrow_mut()) = chrono::offset::Local::now();
				}
				*prev_title.borrow_mut() = (*current_title.borrow()).clone();
			    }
			   
			    
			    // println!("--------------------------------------------");
			    // println!("{:?}", (*window_times.borrow()));
			    // println!("--------------------------------------------");
			});
		    });
		});
	    });
	});
    });
    /*
    let dt = (*hook).data;
    match *dt {
	2 => println!("KeyPress"),
	4 => println!("ButtonPress"),
	6 => println!("MotionNotify"),
	_ => return
    };*/
}

thread_local! (
    static PREV_TITLE: RefCell<String> = RefCell::new(String::from("<uninitialized window>"));
    static CURRENT_TITLE: RefCell<String> = RefCell::new(String::from("<uninitialized window>"));
    static START_DATE: RefCell<chrono::DateTime<chrono::offset::Local>> = RefCell::new(chrono::offset::Local::now());
    static END_DATE: chrono::DateTime<chrono::offset::Local> = chrono::offset::Local::now();
    static PREV_COUNTER: RefCell<Instant> = RefCell::new(Instant::now());
    static WINDOW_TIMES: RefCell<Vec<(String, String, String)>> = RefCell::new(Vec::<(String, String, String)>::new());
);





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
