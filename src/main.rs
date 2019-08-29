extern crate hyper_sse;
extern crate inotify;

#[macro_use]
extern crate lazy_static;

use hyper_sse::Server;
use std::io::{self, BufRead};
use std::path::Path;
use futures::Stream;
use inotify::{Inotify, WatchMask};

lazy_static! {
    static ref PUSH_SERVER: Server<u64> = Server::new();
}

fn main() {
    let addr = ("[::1]:3000").parse().unwrap();
    PUSH_SERVER.spawn(addr);

    let auth_token = PUSH_SERVER.generate_auth_token(Some(0)).unwrap();

    println!("curl 'http://[::1]:3000/push/0?{}'", auth_token);


    let mut inotify = Inotify::init().expect("Failed to initialize inotify");

    inotify.add_watch(Path::new("/tmp"), WatchMask::CREATE | WatchMask::MODIFY);

    let mut buffer = [0; 32];
    let stream = inotify.event_stream(&mut buffer);
    for event in stream.wait() {
        PUSH_SERVER.push(0, "update", &format!("{:?}", event)).ok();
    }
}
