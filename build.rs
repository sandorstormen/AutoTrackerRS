extern crate pkg_config;
use std::process::Command;
use std::env;
use std::path::Path;

fn main() {
    println!("cargo:rustc-link-lib=X11");
    println!("cargo:rustc-link-lib=Xtst");

    // let deps = [
    // 	("gl", "1", "glx"),
    // 	("x11", "1.4.99.1", "xlib"),
    // 	("x11-xcb", "1.6", "xlib_xcb"),
    // 	("xcursor", "1.1", "xcursor"),
    // 	("xext", "1.3", "dpms"),
    // 	("xft", "2.1", "xft"),
    // 	("xi", "1.7", "xinput"),
    // 	("xinerama", "1.1", "xinerama"),
    // 	("xmu", "1.1", "xmu"),
    // 	("xrandr", "1.5", "xrandr"),
    // 	("xrender", "0.9.6", "xrender"),
    // 	("xscrnsaver", "1.2", "xss"),
    // 	("xt", "1.1", "xt"),
    // 	("xtst", "1.2", "xtst"),
    // 	("xxf86vm", "1.1", "xf86vmode"),
    // ];

    // for &(dep, version, feature) in deps.iter() {
    // 	let var = format!(
    // 	    "CARGO_FEATURE_{}",
    // 	    feature.to_uppercase().replace('-', "_")
    // 	);
    // 	if env::var_os(var).is_none() { continue; }
    // 	pkg_config::Config::new().atleast_version(version).probe(dep).unwrap();
    // }
    
}
