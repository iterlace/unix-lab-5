extern crate mach;

use std::{io, process, thread, time};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

use mach::{
    kern_return,
    mach_types::*,
    message::mach_msg_type_number_t,
    traps::{mach_task_self, task_for_pid},
    vm_types::*
};
use mach::vm::mach_vm_read;

fn main() -> io::Result<()> {
    print!("Enter PID of a target process: ");
    io::stdout().flush().unwrap();

    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    let pid: i32 = buf.trim().parse().expect("You must provide an integer!");

    // Open a handle to the target process.
    let task = unsafe { mach_task_self() };
    let mut target_task: task_t = 0;
    let result = unsafe { task_for_pid(task, pid, &mut target_task) };
    if result != kern_return::KERN_SUCCESS {
        panic!("Failed to open handle to target process: {:?}", result);
    }

    // Read the first 100 bytes of the target process's memory.
    let mut buffer: [u8; 100] = [0; 100];
    let mut size: mach_msg_type_number_t = 0;
    let result = unsafe {
        mach_vm_read(
            target_task,
            0x1000 as mach_vm_address_t, // Replace with the target address to read.
            100 as mach_vm_size_t, // Replace with the number of bytes to read.
            buffer.as_mut_ptr() as *mut vm_offset_t,
            &mut size,
        )
    };
    if result != kern_return::KERN_SUCCESS {
        panic!("Failed to read memory from target process: {:?}", result);
    }

    // Do something with the data.
    println!("{:?}", buffer);

    Ok(())
}
