extern crate hyper_sse;
extern crate inotify;

#[macro_use]
extern crate lazy_static;

use hyper_sse::Server;
use std::env;
use futures::Stream;
use inotify::{Inotify, WatchMask};
use walkdir::WalkDir;

lazy_static! {
    static ref PUSH_SERVER: Server<u64> = Server::new();
}

fn main() {
    let addr = env::var("ADDR").unwrap_or("[::1]:3000".to_string()).parse().unwrap();
    PUSH_SERVER.spawn(addr);

    let auth_token = PUSH_SERVER.generate_auth_token(Some(0)).unwrap();

    println!("curl -isSL 'http://{}/push/0?{}'", addr, auth_token);

    let mut inotify = Inotify::init().expect("Failed to initialize inotify");

    let watch_dir = env::var("WATCH_DIR").unwrap_or(".".to_string());

    for entry in WalkDir::new(watch_dir).follow_links(true).into_iter().filter_map(filter_dir) {
        eprintln!("{:?}", entry);
        let _ = inotify.add_watch(entry.path(), WatchMask::MODIFY);
    }

    let mut buffer = [0; 1024];
    let stream = inotify.event_stream(&mut buffer[..]);
    for event in stream.wait() {
        //eprintln!("{:?}", event.unwrap().name.unwrap());
        PUSH_SERVER.push(0, "update", &format!("{}", event.unwrap().name.unwrap().to_string_lossy())).ok();
    }
}
fn filter_dir(e: walkdir::Result<walkdir::DirEntry>) -> Option<walkdir::DirEntry> {
    if let Ok(e) = e {
        if let Ok(metadata) = e.metadata() {
            if metadata.is_dir() {
                return Some(e);
            }
        }
    }
    None
}
