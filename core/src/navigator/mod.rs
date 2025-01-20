use crate::navigator::mori::Rail;

// pub mod basic;
pub mod mori;
pub mod mori_cost;
pub mod path_executor;
pub mod path_grouper;
pub mod path_planner;
pub mod path_side;
mod rail_point_compare;
pub mod resource_cloud;
pub mod shinri;
mod threaded_search;

pub enum PathingResult {
    Route { path: Vec<Rail>, cost: u32 },
    FailingDebug(Vec<Rail>),
}

impl PathingResult {
    pub fn is_route(&self) -> bool {
        match &self {
            PathingResult::Route { .. } => true,
            PathingResult::FailingDebug(..) => false,
        }
    }
}
