//! Ordinary host-native control, never a portable-artifact demonstration.
use std::io::Write;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("echo") => {
            println!("{}", args[2]);
            eprintln!("control stderr");
            ExitCode::from(7)
        }
        Some("bytes") => {
            std::io::stdout().write_all(&[0, 255, 10]).unwrap();
            ExitCode::SUCCESS
        }
        Some("sleep") => {
            std::thread::sleep(std::time::Duration::from_secs(10));
            ExitCode::SUCCESS
        }
        Some("flood") => {
            let _ = std::io::stdout().write_all(&vec![b'x'; 256 * 1024]);
            ExitCode::SUCCESS
        }
        Some("mark") => {
            let mut file = std::fs::OpenOptions::new().create(true).append(true).open(&args[2]).unwrap();
            writeln!(file, "executed").unwrap();
            ExitCode::SUCCESS
        }
        Some("environment") => {
            assert!(std::env::var_os("POLYSTRIDE_TEST_SECRET").is_none());
            assert_eq!(std::env::current_dir().unwrap().file_name().unwrap(), "work");
            assert!(std::path::Path::new(&std::env::var_os("HOME").unwrap()).is_dir());
            std::fs::write("control-file", "file probe").unwrap();
            assert_eq!(std::fs::read_to_string("control-file").unwrap(), "file probe");
            std::fs::remove_file("control-file").unwrap();
            println!("{}", std::env::var("LAB_VALUE").unwrap());
            ExitCode::SUCCESS
        }
        Some("mutate") => {
            let executable = std::env::current_exe().unwrap();
            std::fs::remove_file(&executable).unwrap();
            std::fs::write(executable, b"changed").unwrap();
            ExitCode::SUCCESS
        }
        _ => ExitCode::from(2),
    }
}
