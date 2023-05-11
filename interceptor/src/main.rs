use std::ffi::CStr;
use std::fs::File;
use std::io::{self, Read};
use std::io::{BufRead, BufReader};
use std::os::raw::c_char;
use std::os::unix::io::{AsRawFd, RawFd};
use std::ptr::null_mut;

use libc;
use libc::c_uint;

fn main() -> io::Result<()> {
    let kb = detect_keyboard()?;
    let fd = open_keyboard_device(kb.as_str())?;
    loop {
        let mut event = read_keyboard_event(fd)?;
        println!("got event: value={:?}; type={:?}, code={:?}", event.value, event.type_, event.code);
    }
}

fn open_keyboard_device(path: &str) -> io::Result<RawFd> {
    let fd = unsafe { libc::open(path.as_ptr() as *const c_char, libc::O_RDONLY) };
    if fd < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(fd)
    }
}

fn read_keyboard_event(fd: RawFd) -> io::Result<libc::input_event> {
    let mut event: libc::input_event = unsafe { std::mem::zeroed() };
    let result = unsafe {
        libc::read(
            fd,
            &mut event as *mut libc::input_event as *mut libc::c_void,
            std::mem::size_of::<libc::input_event>(),
        )
    };
    if result < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(event)
    }
}

fn detect_keyboard() -> Result<String, io::Error> {
    println!("Provide a path for your keyboard device: ");
    return Ok("/dev/input/by-id/usb-SteelSeries_SteelSeries_Apex_100_Gaming_Keyboard-event-kbd".to_string());
}