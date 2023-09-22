use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
};

mod read;

const BPM_VERSION: &'static str = env!("CARGO_PKG_VERSION");
const BPM_DIR: &'static str = "bpm.d";

macro_rules! log {
    ($($arg:tt)*) => {
        {
            let now = chrono::Local::now().format("%+").to_string();
            println!("{}\t{}", now, format!($($arg)*));
        }
    };
}

fn supervise(exec_path: PathBuf) {
    let file_name = exec_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    loop {
        log!("{}\tstarting", file_name);
        let mut child = match Command::new(exec_path.as_path())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => {
                log!("{}\tfailed to start: {}", file_name, e);
                std::thread::sleep(std::time::Duration::from_secs(1));
                continue;
            }
        };

        // Spawn threads to read stdout and stderr
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let fname = file_name.to_string();
        std::thread::spawn(move || {
            if let Some(stdout) = stdout {
                let mut reader = std::io::BufReader::new(stdout);
                let reader = read::LineReader::from(&mut reader);
                for line in reader {
                    log!("{}/out\t{}", fname, line);
                }
            }
        });
        let fname = file_name.to_string();
        std::thread::spawn(move || {
            if let Some(stderr) = stderr {
                let mut reader = std::io::BufReader::new(stderr);
                let reader = read::LineReader::from(&mut reader);
                for line in reader {
                    log!("{}/err\t{}", fname, line);
                }
            }
        });

        // Wait for child to exit
        let exit_status = match child.wait() {
            Ok(exit_status) => exit_status,
            Err(e) => {
                log!("{}\tfailed to wait: {}", file_name, e);
                std::thread::sleep(std::time::Duration::from_secs(1));
                continue;
            }
        };
        log!("{}\texited with status {}", file_name, exit_status);

        if exit_status.success() {
            break;
        }

        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn main() {
    log!("\tbpm v{}", BPM_VERSION);

    // Read files in bpm.d folder
    let threads = fs::read_dir(BPM_DIR)
        .expect(format!("{} folder not found", BPM_DIR).as_str())
        .filter_map(|entry| entry.ok())
        .map(|entry| std::thread::spawn(move || supervise(entry.path())))
        .collect::<Vec<_>>();

    // Wait for all threads to finish
    log!("\tLaunched {} threads", threads.len());
    for thread in threads {
        thread.join().expect("\tThread panicked");
    }

    log!("\tAll threads finished")
}
