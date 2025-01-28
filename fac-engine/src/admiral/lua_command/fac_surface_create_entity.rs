use crate::admiral::lua_command::{DEFAULT_FORCE_VAR, LuaCommand};
use crate::admiral::trimmer::string_space_shrinker;
use crate::blueprint::bpfac::infinity::{FacBpFilter, FacBpInfinitySettings};
use crate::blueprint::bpfac::position::FacBpPosition;
use crate::blueprint::bpfac::schedule::FacBpSchedule;
use crate::game_entities::belt_split::FacEntBeltSplitPriority;
use crate::game_entities::direction::FacDirectionEighth;
use crate::game_entities::module::FacModule;
use crate::util::ansi::C_BLOCK_LINE;
use itertools::Itertools;
use std::convert::AsRef;

pub const DEBUG_PRE_COLLISION: bool = true;
pub const DEBUG_POSITION_EXPECTED: bool = true;

#[derive(Debug)]
pub struct FacSurfaceCreateEntity {
    pub name: String,
    pub position: FacBpPosition,
    pub params: Vec<CreateParam>,
    pub commands: Vec<String>,
}

impl LuaCommand for FacSurfaceCreateEntity {
    fn make_lua(&self) -> String {
        let params_str = self
            .params
            .iter()
            .map(|v| {
                let (key, value) = v.to_param();
                format!("{}={}", key, value)
            })
            .join(",");

        let mut lua: Vec<String> = Vec::new();

        let name = &self.name;
        let x = self.position.x;
        let y = self.position.y;
        let nice_pos = self.position.display();

        if DEBUG_PRE_COLLISION {
            let direction = self.params.iter().find_map(|v| match v {
                CreateParam::DirectionFacto(direction) => Some(direction.as_ref()),
                _ => None,
            });

            let direction_param = if let Some(direction) = direction {
                format!("defines.direction.{}", direction.to_lowercase())
            } else {
                "".to_string()
            };

            lua.push(
                format!(
                    r#"
                    if game.surfaces[1].entity_prototype_collides("{name}", {{ {x}, {y} }}, false, {direction_param}) then
                        rcon.print("[Admiral] Collision {name} {nice_pos}")           
                    end 
                    "#
                )
                .trim()
                .replace('\n', " ")
                .replace("    ", ""),
            )
        }

        if !self.commands.is_empty() || DEBUG_POSITION_EXPECTED {
            lua.push("local admiral_create =".to_string());
        }

        lua.push(
            format!(
                r#"game.surfaces[1].create_entity{{ 
                    name="{name}", 
                    position={{ {x}, {y} }}, 
                    force={DEFAULT_FORCE_VAR},
                    {params_str}
                }}"#,
            )
            .trim()
            .replace('\n', "")
            .replace("    ", ""),
        );

        lua.extend_from_slice(&self.commands);

        if DEBUG_POSITION_EXPECTED {
            lua.push(format!(
                r#"if admiral_create == nil then
                    rcon.print("[Admiral] Inserted {name} at {nice_pos} but was nil")
                elseif admiral_create.position.x ~= {x} or admiral_create.position.y ~= {y} then
                    rcon.print("[Admiral] Inserted {name} at {nice_pos} but was placed at " .. admiral_create.position.x .. "{C_BLOCK_LINE}" .. admiral_create.position.y)
                end"#
            ).trim().replace('\n', ""));
        }

        lua.join(" ")
    }
}

impl FacSurfaceCreateEntity {
    pub fn new(name: &str, position: FacBpPosition) -> Self {
        FacSurfaceCreateEntity {
            name: name.to_string(),
            position,
            params: Vec::new(),
            commands: Vec::new(),
        }
    }

    pub fn with_param(&mut self, param: CreateParam) {
        self.params.push(param);
    }

    fn with_command(&mut self, command: String) {
        self.commands.push(command);
    }

    pub fn with_command_module(&mut self, module: &FacModule) {
        self.with_command(format!(
            "admiral_create.get_module_inventory().insert(\"{}\")",
            module.to_fac_name()
        ));
    }

    pub fn with_command_infinity_settings(
        &mut self,
        FacBpInfinitySettings {
            remove_unfiltered_items,
            filters,
        }: &FacBpInfinitySettings,
    ) {
        self.with_command(format!(
            "admiral_create.remove_unfiltered_items = {remove_unfiltered_items}"
        ));
        self.with_command(format!(
            "admiral_create.infinity_container_filters  = {{ }}"
        ));
        for (i, FacBpFilter { name, count, mode }) in filters.iter().enumerate() {
            let lua_index = i + 1;
            let text = format!(
                "admiral_create.set_infinity_container_filter({lua_index}, {{
                    name = \"{name}\",
                    count = {count},
                    mode = \"{mode}\",
                }} )"
            )
            .replace("\n", " ");
            let text = string_space_shrinker(text);
            self.with_command(text)
        }
    }

    pub fn with_command_schedule(&mut self, schedule: &FacBpSchedule) {
        let lua_sched = serde_lua_table::to_string(&schedule.schdata).unwrap();
        // self.with_command(format!("admiral_create.train.schedule  = {{ }}"));
        self.with_command(format!(
            "admiral_create.train.schedule  = {{ current = 1, records = {lua_sched} }}"
        ));
        // TODO: Doesn't work, must be seperate command
        self.with_command(format!("admiral_create.train.manual_mode = false"));
    }

    pub fn with_command_splitter(&mut self, pri: FacEntBeltSplitPriority) {
        self.with_command(format!(
            "admiral_create.splitter_input_priority  = {}",
            serde_json::to_string(&pri.input).unwrap()
        ));
        self.with_command(format!(
            "admiral_create.splitter_output_priority  = {}",
            serde_json::to_string(&pri.output).unwrap()
        ));
    }
}

#[derive(Debug)]
pub enum CreateParam {
    DirectionFacto(FacDirectionEighth),
    Lua { name: &'static str, lua: String },
}

impl CreateParam {
    pub fn to_param(&self) -> (&str, String) {
        match self {
            CreateParam::DirectionFacto(direction) => {
                let direction: &str = direction.as_ref();
                (
                    "direction",
                    format!("defines.direction.{}", direction.to_lowercase()),
                )
            }
            CreateParam::Lua { name, lua } => (name, wrap_quotes(lua)),
        }
    }
}

fn wrap_quotes(input: impl AsRef<str>) -> String {
    format!(r#""{}""#, input.as_ref())
}
