use std::ffi::c_void;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::os::unix::io::AsRawFd;
use std::os::unix::process::CommandExt;
use std::os::unix::raw::off_t;
use std::process::Command;

use libc::{lseek, pread64};

const PAGE_SIZE: usize = 4096;

fn read_proc_mem(pid: i32) -> io::Result<Vec<u8>> {
    let mut file = File::open(format!("/proc/{}/mem", pid))?;
    let mut buffer = Vec::new();

    // Get the start and end address of the process's memory mappings
    let mut maps_file = File::open(format!("/proc/{}/maps", pid))?;
    let mut maps_buffer = String::new();
    maps_file.read_to_string(&mut maps_buffer)?;
    let mut start_addr: Option<usize> = None;
    let mut end_addr: Option<usize> = None;

    for line in maps_buffer.lines() {
        let mut parts = line.split_ascii_whitespace();
        let addr_str = parts.next().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid maps file"))?;
        let perms = parts.next().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid maps file"))?;
        if !perms.contains("r") {
            println!("perms: {}", perms.clone());
            // Skip non-readable memory mappings
            continue;
        }
        let addr_parts: Vec<&str> = addr_str.split("-").collect();
        if addr_parts.len() != 2 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid maps file"));
        }
        let start = usize::from_str_radix(addr_parts[0], 16).unwrap();
        let end = usize::from_str_radix(addr_parts[1], 16).unwrap();
        if start_addr.is_none() || start < start_addr.unwrap() {
            start_addr = Some(start);
        }
        if end_addr.is_none() || end > end_addr.unwrap() {
            end_addr = Some(end);
        }
    }
    let start_addr = start_addr.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no readable memory mappings found (start)"))?;
    let end_addr = end_addr.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no readable memory mappings found (end)"))?;

    // Read the process memory one page at a time
    let mut offset: i64 = start_addr as i64;
    while offset < end_addr as i64 {
        let mut page_buffer = [0u8; PAGE_SIZE];
        let bytes_read: i64 = unsafe {
            pread64(file.as_raw_fd(), page_buffer.as_mut_ptr() as *mut c_void, PAGE_SIZE, offset as i64).try_into().unwrap()
        };
        if bytes_read <= 0 {
            // Error or end of file
            break;
        }
        buffer.extend_from_slice(&page_buffer[..bytes_read as usize]);
        offset += bytes_read;
    }

    Ok(buffer)
}

fn main() -> io::Result<()> {
    print!("Enter PID of a target process: ");
    io::stdout().flush().unwrap();

    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    let pid: i32 = buf.trim().parse().expect("You must provide an integer!");

    match read_proc_mem(pid) {
        Ok(mem) => {
            println!("Process {} used {} bytes of memory", pid, mem.len());
            let mut file = File::options()
                .read(true)
                .write(true)
                .create_new(true)
                .open(format!("/tmp/memdump_{}", pid))?;
            file.write(mem.as_slice()).unwrap();
        }
        Err(e) => eprintln!("Error reading process memory: {}", e),
    }


    Ok(())
}
