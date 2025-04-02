use std::io::{BufRead, BufReader, Write};
use std::sync::mpsc::Sender;
use std::{fs, io, thread};

use interprocess::local_socket::prelude::{LocalSocketListener, LocalSocketStream};
use interprocess::local_socket::traits::{ListenerExt, Stream};
use interprocess::local_socket::{
    GenericFilePath, GenericNamespaced, ListenerOptions, Name, NameType, ToFsName, ToNsName,
};
use serde::{Deserialize, Serialize};
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

#[derive(Serialize, Deserialize, Debug)]
struct SocketMessage {
    opener: String,
    url: String,
}

// returns SingleInstance, so that lock is held until end of program lifetime
pub fn check_single_instance(
    url: &str,
    main_sender: Sender<MessageToMain>,
) -> (bool, SingleInstance) {
    let lock_name = lock_name_or_path();

    let runtime_dir = paths::get_runtime_dir();
    fs::create_dir_all(runtime_dir.as_path()).unwrap();
    let single_instance = SingleInstance::new(lock_name.as_str()).unwrap();

    let pipe_name_result = if GenericNamespaced::is_supported() {
        // interprocess calls namespaced pipe:
        // Windows named pipes
        // Unix domain sockets
        "software.Browsers.socket".to_ns_name::<GenericNamespaced>()
    } else {
        warn!("GenericNamespaced type is not supported or support could not be queried");
        return (true, single_instance);
    };

    if pipe_name_result.is_err() {
        warn!("Could not parse pipe name as valid name for GenericNamespaced");
        warn!("{}", pipe_name_result.unwrap_err());
        return (true, single_instance);
    }
    let pipe_name: Name = pipe_name_result.unwrap();

    return if single_instance.is_single() {
        info!("No other instance of Browsers was running");
        // another process is not running, so this is first

        let listener_result = ListenerOptions::new()
            .name(pipe_name)
            .create_sync_as::<LocalSocketListener>();

        if listener_result.is_err() {
            warn!("Could not run single instance socket server");
            warn!("{}", listener_result.unwrap_err());
            return (true, single_instance);
        }
        let listener = listener_result.unwrap();
        info!("Started socket listener for new instances of Browser");

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

                info!("Another Browsers instance sent arguments: {}", buffer);

                let message: SocketMessage = serde_json::from_str(buffer.as_str())
                    .expect("socket message is not in valid json format");

                let url_open_request =
                    MessageToMain::UrlOpenRequest(message.opener.to_string(), message.url.clone());
                main_sender.send(url_open_request).unwrap();

                // Clear the buffer so that the next iteration will display new data instead of messages
                // stacking on top of one another.
                buffer.clear();
            }
        });

        (true, single_instance)
    } else {
        info!("Other Browsers instance is already running");
        // another process is running, so this is not first
        let result = LocalSocketStream::connect(pipe_name);
        if result.is_err() {
            warn!("Could not connect to single instance socket server");
            return (true, single_instance);
        }
        let mut local_socket_stream = result.unwrap();

        let message = SocketMessage {
            opener: "".to_string(),
            url: url.to_string(),
        };

        let message_json =
            serde_json::to_string(&message).expect("Could not create socket message");

        let message = format!("{message_json}\n");
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
