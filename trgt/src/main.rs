use std::{io, process, thread, time};
use std::io::Write;


fn main() -> io::Result<()> {
    println!("PID: {}", process::id());
    let mut buf = String::new();
    print!("Enter any line of text: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut buf)?;

    println!("Saved. Falling asleep...");
    thread::sleep(time::Duration::from_secs(60*60));
    Ok(())
}
