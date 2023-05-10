use std::ffi::c_void;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::os::unix::io::AsRawFd;
use std::os::unix::process::CommandExt;
use std::os::unix::raw::off_t;
use std::process::Command;
use std::str::FromStr;

use libc::{lseek, pread64};

const PAGE_SIZE: usize = 4096;


struct Mapping {
    addr_start: usize,
    addr_end: usize,
    file_path: String,
    perms: String,
    inode_id: u64
}


fn parse_mapping(line: &str) -> Result<Mapping, io::Error> {
    let mut parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() != 6 {
        println!("line: {:?}", parts);
        return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid line length"));
    };
    let addr_parts: Vec<&str> = parts[0].split("-").collect();
    if addr_parts.len() != 2 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid addr"));
    }
    let addr_start = usize::from_str_radix(addr_parts[0], 16)
        .map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData, "invalid addr_start")
        })?;

    let addr_end = usize::from_str_radix(addr_parts[1], 16)
        .map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData, "invalid addr_end")
        })?;

    Ok(Mapping {
        addr_start,
        addr_end,
        file_path: String::from(parts[5]),
        perms: String::from(parts[1]),
        inode_id: u64::from_str(parts[4]).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData, "invalid inode_id")
        })?,
    })
}


fn read_proc_mem(pid: i32) -> io::Result<Vec<u8>> {
    let mut file = File::open(format!("/proc/{}/mem", pid))?;

    let mut maps_file = File::open(format!("/proc/{}/maps", pid))?;
    let mut maps_buffer = String::new();
    maps_file.read_to_string(&mut maps_buffer)?;

    let mut memory_buffer = Vec::new();

    for line in maps_buffer.lines() {
        let mapping = match parse_mapping(line) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Error processing line: {}", e.to_string());
                continue;
            }
        };
        if !mapping.perms.contains("r") {
            // ignore non-readable memory mappings
            continue
        }
        if mapping.inode_id != 0 {
            // ignore all files
            continue
        }

        memory_buffer.extend_from_slice(&*format!("\0New segment: {}\0", mapping.file_path).into_bytes());

        // Read the process memory one page at a time
        let mut offset: i64 = mapping.addr_start as i64;
        while offset < mapping.addr_end as i64 {
            let mut page_buffer = [0u8; PAGE_SIZE];
            let bytes_read: i64 = unsafe {
                pread64(
                    file.as_raw_fd(),
                    page_buffer.as_mut_ptr() as *mut c_void,
                    PAGE_SIZE,
                    offset as i64
                ).try_into().unwrap()
            };
            if bytes_read <= 0 {
                // Error or end of file
                break;
            }
            memory_buffer.extend_from_slice(&page_buffer[..bytes_read as usize]);
            offset += bytes_read;
        }
    }

    Ok(memory_buffer)
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
                .create(true)
                .open(format!("/tmp/memdump_{}", pid))?;
            file.write(mem.as_slice()).unwrap();
        }
        Err(e) => eprintln!("Error reading process memory: {}", e),
    }


    Ok(())
}
