
// use x11rb::connection::Connection;
// use x11rb::errors::ReplyOrIdError;
// use x11rb::protocol::xproto::*;
// use x11rb::COPY_DEPTH_FROM_PARENT;
// use x11rb::protocol::*;
// use x11rb::protocol::xinput::*;
// use std::error::Error;
// // use x11rb::xcb_ffi::XCBConnection;
// // use x11rb::generated::xproto::{ConnectionExt, Atom, GetPropertyReply, ATOM, WINDOW};
// // use x11rb::wrapper::LazyAtom;
// use x11rb::x11_utils::TryParse;
// use std::process::exit;

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


fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

fn main() {
    let (conn, screen_num) = x11rb::connect(None).unwrap();
    let screen = &conn.setup().roots[screen_num];
    let root_window = screen.root;
    println!("{}", root_window);

    let net_active_atom = conn.intern_atom(true, String::from("_NET_ACTIVE_WINDOW").as_bytes()).unwrap().reply().unwrap().atom;
    let net_wm_atom = conn.intern_atom(true, String::from("_NET_WM_NAME").as_bytes()).unwrap().reply().unwrap().atom;
    let wm_atom = conn.intern_atom(true, String::from("WM_NAME").as_bytes()).unwrap().reply().unwrap().atom;
    // let mut net_active_window = LazyAtom::new(&conn, false, b"_NET_ACTIVE_WINDOW");
    // let mut net_wm_name = LazyAtom::new(&conn, false, b"_NET_WM_NAME");
    // let mut utf8_string = LazyAtom::new(&conn, false, b"UTF8_STRING");
    
    //let values = ChangeWindowAttributesAux::default().event_mask(xproto::EventMask::KeyPress | xproto::EventMask::ButtonPress | xproto::EventMask::PointerMotion | xproto::EventMask::StructureNotify | xproto::EventMask::SubstructureNotify | xproto::EventMask::FocusChange | xproto::EventMask::PropertyChange);

    match conn.change_window_attributes(root_window, &(ChangeWindowAttributesAux::default().event_mask(xproto::EventMask::PropertyChange))) {
	Ok(res) => println!("attr changed"),
	Err(e) => println!("err {}", e)
    };
    match conn.flush(){
	Ok(res) => println!("No error"),
	Err(e) => println!("{}", e)
    };
    //present::select_input(&conn, ); //Titta nogrannre

    
    
    //print_type_of(&conn);
    //x11rb::protocol::xproto::GetInputFocusRequest();
    //xproto::select_input(&conn, event_id, win_id, present::EventMask::ConfigureNotify);
    let mut focus_mask = Vec::new();
    //focus_mask.push(XIEventMask::FocusIn.into());
    let focus_in_mask = x11rb::protocol::xinput::EventMask{
	deviceid: 1,
	mask: focus_mask
    };
    //xi_select_events(&conn, root, &[focus_in_mask]);
    //x11rb::protocol::xproto::change_property(&conn, x11rb::protocol::xproto::PropMode::Replace, root_window, x11rb::protocol::present::EventMask::CompleteNotify, x11rb::protocol::xproto::Atom, 32, 1, x11rb::protocol::Event::PropertyNotify);
    // x11rb::protocol::present::select_input(&conn, x11rb::protocol::present::COMPLETE_NOTIFY_EVENT as u32, root_window, x11rb::protocol::Event::PropertyNotify);
    loop {
	/*
	match conn.get_input_focus() {
	    Ok(focus) => {
		//print_type_of(&focus);
		handle_window_reply(focus, &conn)
	    }
	    Err(_e) => panic!()
	};
	 */
	match conn.wait_for_event() {
	    Ok(event) => {
		if let Event::PropertyNotify(event) = event {
		    if event.atom == net_active_atom {
			let focus_window = xproto::get_input_focus(&conn).unwrap().reply().unwrap().focus;
			let res: &[u8] = &get_property(&conn, false, focus_window, AtomEnum::WM_NAME, AtomEnum::STRING, 0, 64).unwrap().reply().unwrap().value;
			let name = String::from_utf8_lossy(res);
			println!("{}", name);
		    }
		    else if event.atom == net_wm_atom || event.atom == wm_atom {
			println!("NET_WM_ACTIVE or WM_ACTIVE");
			
		    }
		}
	    },
	    Err(e) => panic!(e)
	}
    };
    
}

fn handle_window_reply(cookie: x11rb::cookie::Cookie<'_, impl x11rb::connection::Connection+std::marker::Send+std::marker::Sync, x11rb::protocol::xproto::GetInputFocusReply>, conn: &(impl x11rb::connection::Connection+std::marker::Send+std::marker::Sync)) -> Result<x11rb::protocol::xproto::GetInputFocusReply, x11rb::errors::ReplyError> {
    match cookie.reply() {
	Ok(reply) => {
	    //print_type_of(&reply);
	    //println!("{}", reply.focus);
	    match conn.get_window_attributes(reply.focus) {
		Ok(wnd_attr) => print_type_of(&wnd_attr),
		Err(_e) => panic!()
	    };
	    Ok(reply)
	},
	Err(e) =>  Err(e)
    }
}
