use std::io::{BufRead, BufReader, Write};
use std::sync::mpsc::Sender;
use std::{fs, io, thread};

use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use single_instance::SingleInstance;
use tracing::{info, warn};

use crate::{paths, MessageToMain};

fn lock_name_or_path() -> String {
    // macos uses file and flock
    if cfg!(target_os = "macos") {
        let runtime_dir = paths::get_runtime_dir();
        fs::create_dir_all(runtime_dir.as_path()).unwrap();

        let lock_file_path = runtime_dir.join("software.Browsers.lock");
        let lock_file_path = lock_file_path.to_str().unwrap();
        return lock_file_path.to_string();
    }

    // Windows uses mutex
    // Linux uses abstract unix domain socket
    return "software.Browsers.lock".to_string();
}

// returns SingleInstance, so that lock is held until end of program lifetime
pub fn check_single_instance(
    url: &str,
    main_sender: Sender<MessageToMain>,
) -> (bool, SingleInstance) {
    let runtime_dir = paths::get_runtime_dir();
    fs::create_dir_all(runtime_dir.as_path()).unwrap();
    let local_socket_path = runtime_dir.join("software.Browsers.socket");

    let lock_name = lock_name_or_path();

    let single_instance = SingleInstance::new(lock_name.as_str()).unwrap();
    return if single_instance.is_single() {
        info!("Other instance is not running");
        // another process is not running, so this is first

        if local_socket_path.exists() {
            let result1 = fs::remove_file(local_socket_path.as_path());
            if result1.is_err() {
                warn!("Could not remove local socket file")
            }
        }
        let listener_result = LocalSocketListener::bind(local_socket_path);
        if listener_result.is_err() {
            warn!("Could not run single instance socket server");
            warn!("{}", listener_result.unwrap_err());
            return (true, single_instance);
        }
        let listener = listener_result.unwrap();
        info!("Started socket listener");

        // Preemptively allocate a sizeable buffer for reading at a later moment. This size should be
        // enough and should be easy to find for the allocator. Since we only have one concurrent
        // client, there's no need to reallocate the buffer repeatedly.
        let mut buffer = String::with_capacity(128);

        thread::spawn(move || {
            for conn in listener.incoming().filter_map(handle_error) {
                // Wrap the connection into a buffered reader right away
                // so that we could read a single line out of it.
                let mut conn = BufReader::new(conn);
                info!("Incoming connection");

                // Since our client example writes first, the server should read a line and only then send a
                // response. Otherwise, because reading and writing on a connection cannot be simultaneous
                // without threads or async, we can deadlock the two processes by having both sides wait for
                // the write buffer to be emptied by the other.
                conn.read_line(&mut buffer).unwrap();

                info!("Another instance sent arguments: {}", buffer);
                let url = buffer.clone();
                let url_open_request =
                    MessageToMain::UrlOpenRequest("".to_string(), url.to_string());
                main_sender.send(url_open_request).unwrap();

                // Clear the buffer so that the next iteration will display new data instead of messages
                // stacking on top of one another.
                buffer.clear();
            }
        });

        (true, single_instance)
    } else {
        info!("Other instance is already running");
        // another process is running, so this is not first
        let result = LocalSocketStream::connect(local_socket_path);
        if result.is_err() {
            warn!("Could not connect to single instance socket server");
            return (true, single_instance);
        }
        let mut local_socket_stream = result.unwrap();

        let message = format!("{url}\n");
        let message_bytes = message.as_bytes();
        let write_result = local_socket_stream.write_all(message_bytes);
        if write_result.is_err() {
            warn!("Could not write message to single instance socket server")
        }
        let flush_result = local_socket_stream.flush();
        if flush_result.is_err() {
            warn!("Could not flush message to single instance socket server")
        }
        info!("Instance was already running");
        (false, single_instance)
    };
}

// Define a function that checks for errors in incoming connections. We'll use this to filter
// through connections that fail on initialization for one reason or another.
fn handle_error(conn: io::Result<LocalSocketStream>) -> Option<LocalSocketStream> {
    match conn {
        Ok(c) => Some(c),
        Err(e) => {
            eprintln!("Incoming connection failed: {}", e);
            None
        }
    }
}
