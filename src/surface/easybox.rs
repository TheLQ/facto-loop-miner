use crate::gamedata::lua::LuaEntity;
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};

#[derive(Serialize, Deserialize, Debug)]
pub struct EasyBox {
    pub max_x: i32,
    pub max_y: i32,
    pub min_x: i32,
    pub min_y: i32,
    pub width: u32,
    pub height: u32,
}

impl Default for EasyBox {
    fn default() -> Self {
        EasyBox {
            max_x: 0,
            max_y: 0,
            min_x: 0,
            min_y: 0,
            width: 0,
            height: 0,
        }
    }
}

impl EasyBox {
    pub fn absolute_x_f32(&self, game_center_x: f32) -> u32 {
        (game_center_x.floor() as i32 - self.min_x) as u32
    }

    pub fn absolute_y_f32(&self, game_center_y: f32) -> u32 {
        (game_center_y.floor() as i32 - self.min_y) as u32
    }

    pub fn absolute_x_u32(&self, game_center_x: i32) -> u32 {
        (game_center_x - self.min_x) as u32
    }

    pub fn absolute_y_u32(&self, game_center_y: i32) -> u32 {
        (game_center_y - self.min_y) as u32
    }

    pub fn expand_to<E>(&mut self, entities: &[E])
    where
        E: LuaEntity,
    {
        for entity in entities {
            self.max_x = max(self.max_x, entity.position().x.floor() as i32);
            self.max_y = max(self.max_y, entity.position().y.floor() as i32);
            self.min_x = min(self.min_x, entity.position().x.floor() as i32);
            self.min_y = min(self.min_y, entity.position().y.floor() as i32);
        }
        self.width = (self.max_x - self.min_x).try_into().unwrap();
        self.height = (self.max_y - self.min_y).try_into().unwrap();
    }
}
