use itertools::Itertools;

use crate::common::names::FacEntityName;

/// Basic Input of package type from mine, train, or factory
pub struct BrainProvide {
    package: FacEntityName,
    count: usize,
}

pub struct BrainWant {
    package: FacEntityName,
    count: usize,
    delivery: DeliverySide,
    priority: usize,
}

#[derive(Debug, PartialEq)]
pub enum DeliverySide {
    Both,
    First,
    Last,
}

/// Describe a base in code
///
/// NOT: Calculate resource tree from "1 belt of red-science"
///
/// Input
/// - Vec:
/// - Vec: Fixed Input/Output belts
///
/// Output
/// -
pub struct BrainSolve {
    combiner_outputs: Vec<Vec<usize>>,
}

impl BrainSolve {
    pub fn new(provides: Vec<BrainProvide>, wants: Vec<BrainWant>) -> Self {
        Self::validate_wants_sides(&wants);

        let mut combiner_outputs = Vec::new();

        let mut want_solved = vec![false; wants.len()];
        for provide in &provides {
            let mut my_wants = wants
                .iter()
                .enumerate()
                .filter(|(_, want)| provide.package == want.package)
                .filter(|(index, _)| want_solved[*index])
                .collect_vec();
            my_wants.sort_by_key(|(_, want)| want.priority);

            for provide_index in 0..provide.count {
                for (want_index, want) in &my_wants {
                    if want.delivery == DeliverySide::Both {
                    } else {
                    }
                }
            }
        }

        Self { combiner_outputs }
    }

    fn validate_wants_sides(wants: &[BrainWant]) {
        let mut want_itr = wants.iter();
        while let Some(want) = want_itr.next() {
            match want.delivery {
                DeliverySide::Both => {}
                DeliverySide::First => {
                    let next = want_itr.next().unwrap();
                    assert_eq!(next.delivery, DeliverySide::Last, "not chained")
                }
                DeliverySide::Last => {
                    let next = want_itr.next().unwrap();
                    assert_eq!(next.delivery, DeliverySide::First, "not chained")
                }
            }
        }
    }
}
