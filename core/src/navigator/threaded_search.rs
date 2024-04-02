use crossbeam::channel::unbounded;
use rayon::Scope;
use std::thread;
use tracing::{info, trace, warn};

pub fn threaded_searcher<E, FN, FS>(start: E, successors: FN, success: FS) -> Option<(Vec<E>, u32)>
where
    E: Clone + Send,
    FN: Fn(&[E]) -> Vec<(E, u32)> + Send + Sync,
    FS: Fn(&E) -> bool + Send + Sync,
{
    thread::scope(|s| {
        let (success_send, success_recv) = unbounded::<Option<(Vec<E>, u32)>>();

        let success_thread = s.spawn(move || {
            info!("spawing success");
            let mut best_cost = u32::MAX;
            let mut best_path = Vec::new();

            while let Some((path, cost)) = success_recv.recv().unwrap() {
                if cost < best_cost {
                    best_cost = cost;
                    best_path = path;
                    trace!("new lowest {} {}", best_path.len(), best_cost);
                }
            }

            (best_path, best_cost)
        });

        rayon::scope(|s| {
            rayon_recursive(
                s,
                vec![start],
                &successors,
                &success,
                success_send.clone(),
                0,
            )
        });

        success_send.send(None).unwrap();

        let (best_path, best_cost) = success_thread.join().unwrap();
        if best_cost == u32::MAX {
            warn!("empty pathfinding!");
            None
        } else {
            Some((best_path, best_cost))
        }
    })
}

pub fn rayon_recursive<'a, 'b, 's, E, FN, FS>(
    scope: &Scope<'a>,
    search_path: Vec<E>,
    successors: &'b FN,
    success: &'b FS,
    done_channel: crossbeam::channel::Sender<Option<(Vec<E>, u32)>>,
    total_cost: u32,
) where
    'b: 'a,
    E: Clone + Send + 'a,
    FN: Fn(&[E]) -> Vec<(E, u32)> + Send + Sync,
    FS: Fn(&E) -> bool + Send + Sync,
{
    for (next_entry, cost) in successors(&search_path) {
        let mut queued_path = Vec::with_capacity(search_path.len() + 1);
        queued_path.extend_from_slice(&search_path);

        let is_success = success(&next_entry);
        queued_path.push(next_entry);

        let total_cost = total_cost.checked_add(cost).unwrap();

        if is_success {
            done_channel.send(Some((queued_path, total_cost))).unwrap();
        } else {
            let done_channel = done_channel.clone();
            scope.spawn(move |scope| {
                rayon_recursive(
                    scope,
                    queued_path,
                    successors,
                    success,
                    done_channel,
                    total_cost,
                )
            });
        }
    }
}
