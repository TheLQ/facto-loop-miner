use crate::navigator::mori::Rail;

pub mod basic;
pub mod mori;
pub mod mori_cost;
pub mod path_grouper;
pub mod path_planner;
mod rail_point_compare;
pub mod resource_cloud;
pub mod shinri;
mod threaded_search;

pub enum PathingResult {
    Route(Vec<Rail>),
    FailingDebug(Vec<Rail>),
}
