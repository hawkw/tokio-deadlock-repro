extern crate futures;
extern crate pretty_env_logger;
extern crate tokio;
#[macro_use]
extern crate log;

use futures::{stream, sync::mpsc, Future, Sink, Stream};
use std::sync::{Arc, Mutex};
use std::usize;

fn main() {
    let _ = pretty_env_logger::try_init();

    let mut i = 0;
    let mut runtime = tokio::runtime::Runtime::new().expect("rt");
    let (tx, rx) = mpsc::unbounded();
    let send_all = tx
        .send_all(stream::iter_ok(1..usize::MAX))
        .map(|_| debug!("sent..."))
        .map_err(|e| panic!("send_all failed: {:?}", e));
    runtime.spawn(Box::new(send_all));
    let rxf = Arc::new(Mutex::new(Some(rx.into_future())));
    loop {
        i += 1;
        let rxf2 = rxf.clone();
        let f = rxf
            .lock()
            .expect("lock rx future")
            .take()
            .expect("rx future already taken!")
            .map(move |(i, rest)| {
                debug!("{:?} received", i);
                *(rxf2.lock().unwrap()) = Some(rest.into_future());
            }).map_err(|_| ());
        runtime
            .block_on(Box::new(f))
            .map(move |_| info!("{}th block_on finished", i))
            .expect("block on failed");
    }
}
