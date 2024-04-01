use crate::navigator::mori::Rail;
use std::collections::VecDeque;

pub fn threaded_searcher<E, FN, FS>(start: E, successors: FN, success: FS) -> Option<(Vec<E>, u32)>
where
    E: Clone,
    FN: Fn(&[E]) -> Vec<(E, u32)>,
    FS: Fn(E) -> bool,
{
    let mut best_cost = u32::MAX;
    let mut best_path: Vec<E> = Vec::new();

    let mut queue: VecDeque<Vec<E>> = VecDeque::new();
    queue.push_back(vec![start]);

    while let Some(search_path) = queue.pop_front() {
        for (next_entry, cost) in successors(&search_path) {
            let mut queued_path = Vec::new();
            queued_path.reserve(search_path.len() + 1);
            queued_path.clone_from_slice(&search_path);

            if success(next_entry) && cost < best_cost {
                best_path = queued_path;
                best_cost = cost;
            } else {
                queue.push_back(queued_path);
            }
        }
    }

    if best_path.is_empty() {
        None
    } else {
        Some((best_path, best_cost))
    }
}
