use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;

pub fn get_socket_path() -> PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(runtime_dir).join("powertoys.sock")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".config").join("powertoys").join("powertoys.sock")
    } else {
        PathBuf::from("/tmp/powertoys.sock")
    }
}

pub fn send_ipc_message(message: &str) -> Result<(), String> {
    let sock_path = get_socket_path();
    let mut stream = UnixStream::connect(&sock_path).map_err(|e| e.to_string())?;
    writeln!(stream, "{}", message).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn start_ipc_listener<F>(callback: F)
where
    F: Fn(String) + Send + Sync + 'static,
{
    let sock_path = get_socket_path();
    let _ = std::fs::remove_file(&sock_path);

    std::thread::spawn(move || {
        let listener = match UnixListener::bind(&sock_path) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("[IPC] Failed to bind socket {}: {}", sock_path.display(), e);
                return;
            }
        };

        for stream in listener.incoming() {
            if let Ok(stream) = stream {
                let reader = BufReader::new(stream);
                for line in reader.lines().flatten() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        callback(trimmed.to_string());
                    }
                }
            }
        }
    });
}
