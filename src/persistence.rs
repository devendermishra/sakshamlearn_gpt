//! One ordered disk writer for all progress files. Pending snapshots for the
//! same path are replaced together, including their completion notifications.
use dioxus::prelude::*;
use serde::Serialize;
use std::{
    collections::BTreeMap,
    io,
    path::PathBuf,
    sync::{mpsc, LazyLock},
};

type Completion = Box<dyn FnOnce(Result<(), String>) + Send>;
struct Save {
    path: PathBuf,
    write: Box<dyn FnOnce(&std::path::Path) -> io::Result<()> + Send>,
    completions: Vec<Completion>,
}
enum Command {
    Save(Save),
    Flush(mpsc::Sender<()>),
}

fn write_pending(pending: &mut BTreeMap<PathBuf, Save>) {
    for (_, save) in std::mem::take(pending) {
        let result = (save.write)(&save.path).map_err(|error| error.to_string());
        if let Err(error) = &result {
            eprintln!("Could not save {}: {error}", save.path.display());
        }
        for completion in save.completions {
            completion(result.clone());
        }
    }
}

static WRITER: LazyLock<mpsc::Sender<Command>> = LazyLock::new(|| {
    let (send, receive) = mpsc::channel::<Command>();
    std::thread::Builder::new()
        .name("learner-storage".into())
        .spawn(move || {
            while let Ok(first) = receive.recv() {
                let mut pending = BTreeMap::new();
                // Bound each batch so continuous input cannot starve disk writes.
                for command in std::iter::once(first).chain(receive.try_iter().take(63)) {
                    match command {
                        Command::Save(next) => {
                            if let Some(mut previous) = pending.insert(next.path.clone(), next) {
                                pending
                                    .get_mut(&previous.path)
                                    .unwrap()
                                    .completions
                                    .append(&mut previous.completions);
                            }
                        }
                        Command::Flush(done) => {
                            write_pending(&mut pending);
                            let _ = done.send(());
                        }
                    }
                }
                write_pending(&mut pending);
            }
        })
        .expect("start learner storage worker");
    send
});

fn enqueue<T: Serialize + Send + 'static>(path: PathBuf, value: T, completion: Completion) {
    let save = Save {
        path,
        write: Box::new(move |path| crate::state::write_local(path, &value)),
        completions: vec![completion],
    };
    if let Err(mpsc::SendError(Command::Save(save))) = WRITER.send(Command::Save(save)) {
        for completion in save.completions {
            completion(Err("Storage worker stopped".into()));
        }
    }
}

pub fn flush() -> io::Result<()> {
    let (send, receive) = mpsc::channel();
    WRITER
        .send(Command::Flush(send))
        .map_err(|error| io::Error::other(error.to_string()))?;
    receive.recv().map_err(io::Error::other)
}

pub fn save<T: Serialize + Send + 'static>(path: PathBuf, value: T) -> io::Result<()> {
    let (send, receive) = mpsc::channel();
    enqueue(
        path,
        value,
        Box::new(move |result| {
            let _ = send.send(result);
        }),
    );
    receive
        .recv()
        .map_err(io::Error::other)?
        .map_err(io::Error::other)
}

/// Move loaded data into its signal once. Keeping a cloned load result in a
/// hook would retain a second complete copy of all saved handwriting.
pub fn use_loaded<T: Default + 'static>(
    load: impl FnOnce() -> Result<T, String>,
) -> (Signal<T>, Result<(), String>) {
    let mut initial = None;
    let status = use_hook(|| {
        load().map(|value| {
            initial = Some(value);
        })
    });
    let value = use_signal(|| initial.take().unwrap_or_default());
    (value, status)
}

/// Enqueue before spawning: navigation may cancel the UI future, but never the
/// save itself. Ignore acknowledgements for superseded edits or disposed scopes.
pub fn use_saver<T: Serialize + Send + 'static>(
    path: PathBuf,
    mut failed: Signal<bool>,
) -> Callback<T> {
    let mut revision = use_signal(|| 0u64);
    use_callback(move |value: T| {
        let current = *revision.peek() + 1;
        revision.set(current);
        let (send, receive) = tokio::sync::oneshot::channel();
        enqueue(
            path.clone(),
            value,
            Box::new(move |result| {
                let _ = send.send(result);
            }),
        );
        spawn(async move {
            let result = receive.await;
            if revision.try_read().is_ok_and(|latest| *latest == current) {
                if let Ok(mut flag) = failed.try_write() {
                    *flag = !matches!(result, Ok(Ok(())));
                }
            }
        });
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synchronous_save_waits_for_pending_snapshots() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("progress.json");
        for value in 0..100 {
            enqueue(
                path.clone(),
                vec![value],
                Box::new(|result| assert!(result.is_ok())),
            );
        }
        save(path.clone(), vec![100]).unwrap();
        assert_eq!(
            crate::state::load_local::<Vec<i32>>(&path).unwrap(),
            vec![100]
        );
    }

    #[test]
    fn reopening_waits_for_queued_progress() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("new-progress.json");
        enqueue(
            path.clone(),
            vec![17],
            Box::new(|result| assert!(result.is_ok())),
        );
        assert_eq!(
            crate::state::load_local::<Vec<i32>>(&path).unwrap(),
            vec![17]
        );
    }

    #[test]
    fn failed_save_reports_error_and_preserves_previous_file() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("progress.json");
        save(path.clone(), vec![42]).unwrap();
        assert!(save(path.join("child.json"), vec![7]).is_err());
        assert_eq!(
            crate::state::load_local::<Vec<i32>>(&path).unwrap(),
            vec![42]
        );
    }
}
