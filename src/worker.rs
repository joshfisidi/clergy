use crossbeam_channel::{unbounded, Receiver};
use std::thread;

use crate::model::PurgeData;

pub fn spawn_purge_worker() -> Receiver<Result<PurgeData, String>> {
    let (tx, rx) = unbounded();

    thread::spawn(move || {
        // Authentication belongs to the main thread with the TUI suspended.
        // The backend uses sudo -n exclusively, so this worker cannot read keys.
        match crate::actions::run_purge() {
            Ok(data) => {
                let _ = tx.send(Ok(data));
            }
            Err(e) => {
                let _ = tx.send(Err(e.to_string()));
            }
        }
    });

    rx
}
