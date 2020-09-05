
extern crate x11rb;
extern crate x11;
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

#[allow(dead_code)]
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

    let net_active_window_atom = conn.intern_atom(true, String::from("_NET_ACTIVE_WINDOW").as_bytes()).unwrap().reply().unwrap().atom;
    let net_wm_name_atom = conn.intern_atom(true, String::from("_NET_WM_NAME").as_bytes()).unwrap().reply().unwrap().atom;
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
	read_input_thread();
    });

    let focus_window = xproto::get_input_focus(&conn).unwrap().reply().unwrap().focus;
    let res: &[u8] = &get_property(&conn, false, root_window, AtomEnum::WM_NAME, AtomEnum::STRING, 0, 64).unwrap().reply().unwrap().value;
    let name = String::from_utf8_lossy(res);
    match tx.send(String::from(name)) {
	Ok(_) => (),
	Err(error) => eprintln!("Can't get initial window: {}", error)
    };
    
    loop {
	match conn.wait_for_event() {
	    Ok(event) => {
		if let Event::PropertyNotify(event) = event {
		    if event.atom == net_active_window_atom {
			let focus_window = xproto::get_input_focus(&conn).unwrap().reply().unwrap().focus;
			let res: &[u8] = &get_property(&conn, false, focus_window, AtomEnum::WM_NAME, AtomEnum::STRING, 0, 64).unwrap().reply().unwrap().value;
			let name = String::from_utf8_lossy(res);
			if !name.is_empty() {
			    match tx.send(String::from(name)) {
				Ok(_) => (),
				Err(error) => eprintln!("Can't send window: {}", error)
			    };
			}
			let res_utf8: &[u8] = &get_property(&conn, false, focus_window, net_wm_name_atom, AtomEnum::STRING, 0, 64).unwrap().reply().unwrap().value;
			let name_utf8 = String::from_utf8_lossy(res);
			for
			println!("{}", name_utf8);
			// 1 - 744
			// 746 - 799
			// println!("{}", String::from_utf8_lossy(&get_property(&conn, false, focus_window, 331 as u32, AtomEnum::STRING, 0, 64).unwrap().reply().unwrap().value));
			// for i in 1..744 {
			//     let mut atom = &get_property(&conn, false, focus_window, i as u32, AtomEnum::STRING, 0, 64).unwrap().reply().unwrap().value;
			//     let mut atom_str = String::from_utf8_lossy(atom);
			//     if !atom_str.is_empty() {
			// 	println!("{}: {}", i, atom_str);
			//     }
			// }
			// for i in 746..799 {
			//     let mut atom = &get_property(&conn, false, focus_window, i as u32, AtomEnum::STRING, 0, 64).unwrap().reply().unwrap().value;
			//     let mut atom_str = String::from_utf8_lossy(atom);
			//     if !atom_str.is_empty() {
			// 	println!("{}: {}", i, atom_str);
			//     }
			// }
			for i in 0..1000 {
			    let mut atom = match get_property(&conn, false, focus_window, i as u32, AtomEnum::STRING, 0, 64).unwrap().reply() {
				Ok(res) => res.value,
				Err(_) => std::vec::Vec::<u8>::new()
			    };
			    let mut atom_str = String::from_utf8_lossy(&atom);
			    let mut extended_str = String::new();
			    for c in atom.iter() {
				//println!("{}", *c as char);
			    }
			    if !atom_str.is_empty() {
				println!("{}: {}", i, atom_str);
			    }
			    if i == 999 {
				println!("i = {}", i)
			    }
			}
			// for i in 746..10000 {
			//      println!("{}", i);
			//      println!("{}", String::from_utf8_lossy(&get_property(&conn, false, focus_window, i as u32, AtomEnum::STRING, 0, 64).unwrap().reply().unwrap().value));
			// }
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
			    if (*window_times.borrow()).len() >= 10 {
				let window_times_save = (*window_times.borrow_mut()).clone();
				spawn(move || {
				    save_to_disk(window_times_save);
				});
				(*window_times.borrow_mut()).clear();
			    };
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

fn save_to_disk(window_times: Vec<(String, String, String)>) {
    let mut writer;
    
    if Path::new("window_times.csv").exists() {
	writer = csv::WriterBuilder::new().has_headers(false).from_writer(std::fs::OpenOptions::new().append(true).open("window_times.csv").unwrap());
    }
    else {
	writer = csv::WriterBuilder::new().has_headers(false).from_writer(std::fs::OpenOptions::new().append(true).create(true).open("window_times.csv").unwrap());
    }
    for (window_name, start_time, end_time) in window_times {
	match writer.write_byte_record(&csv::ByteRecord::from(vec![window_name.as_str(), start_time.as_str(), end_time.as_str()])) {
	    Ok(_) => (),
	    Err(e) => eprintln!("Writing csv row error: {}", e)
	};
    }
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
