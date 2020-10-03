
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

thread_local! (
    static PREV_TITLE: RefCell<String> = RefCell::new(String::from("<uninitialized window>"));
    static CURRENT_TITLE: RefCell<String> = RefCell::new(String::from("<uninitialized window>"));
    static START_DATE: RefCell<chrono::DateTime<chrono::offset::Local>> = RefCell::new(chrono::offset::Local::now());
    static END_DATE: chrono::DateTime<chrono::offset::Local> = chrono::offset::Local::now();
    static PREV_COUNTER: RefCell<Instant> = RefCell::new(Instant::now());
    static WINDOW_TIMES: RefCell<Vec<(String, String, String)>> = RefCell::new(Vec::<(String, String, String)>::new());
);

lazy_static! {
    pub static ref CHANNEL: Mutex<(std::sync::mpsc::Sender<String>, std::sync::mpsc::Receiver<String>)> = Mutex::new(mpsc::channel());
}

pub fn read_input_thread() {
    let (ctrl_conn, screen_num) = x11rb::connect(None).unwrap();
    let (data_conn, screen_num) = x11rb::connect(None).unwrap();

    set_focused_window();
    
    let c_char: *const std::os::raw::c_char = &mut 0;
    let input_display: *mut x11::xlib::Display;

    let rc = ctrl_conn.generate_id().unwrap();
    let empty = record::Range8 {
        first: 0,
        last: 0,
    };
    let empty_ext = record::ExtRange {
        major: empty,
        minor: record::Range16 {
            first: 0,
            last: 0,
        },
    };
    let range = record::Range {
        core_requests: empty,
        core_replies: empty,
        ext_requests: empty_ext,
        ext_replies: empty_ext,
        delivered_events: empty,
        device_events: record::Range8 {
            // We want notification of core X11 events between key press and motion notify
            first: xproto::KEY_PRESS_EVENT,
            last: xproto::MOTION_NOTIFY_EVENT,
        },
        errors: empty,
        client_started: false,
        client_died: false,
    };
   
    record::create_context(&ctrl_conn, rc, 0, &[record::CS::AllClients.into()], &[range]).unwrap().check().unwrap();
    const START_OF_DATA: u8 = 4;
    const RECORD_FROM_SERVER: u8 = 0;
    loop {
	let data = record::enable_context(&data_conn, rc).unwrap().reply().unwrap();
	if data.client_swapped {
	    eprintln!("Byte swapped")
	}
	else if data.category == RECORD_FROM_SERVER {
	    println!("RECORD FROM SERver")
	}
	else if data.category == START_OF_DATA {
	    println!("Start of data")
	}
	else {
	    println!("Other")
	}
    }
    

    /*
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
    };*/
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
