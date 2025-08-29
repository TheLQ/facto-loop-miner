use crate::admiral::lua_command::LuaCommand;
use crate::admiral::trimmer::string_space_shrinker;
use crate::common::varea::{VArea, VAreaSugar};

#[derive(Debug)]
pub struct FacRenderDestroy {
    area: Option<VArea>,
}

impl FacRenderDestroy {
    pub fn destroy_area(area: VArea) -> Self {
        Self { area: Some(area) }
    }
}

impl LuaCommand for FacRenderDestroy {
    fn make_lua(&self) -> String {
        let loop_pre;
        let loop_post;

        if let Some(area) = &self.area {
            let VAreaSugar {
                start_x,
                start_y,
                end_x,
                end_y,
            } = area.sugar();
            loop_pre = format!(
                r"
local target  = rendering.get_left_top(id)
if target == nil then
    target = rendering.get_target(id)
end

if target.position.x >= {start_x}
    and target.position.x <= {end_x}
    and target.position.y >= {start_y}
    and target.position.y <= {end_y}
     then
"
            );
            loop_post = "end";
        } else {
            loop_pre = "".into();
            loop_post = "";
        }

        let res = format!(
            r"
local texts = rendering.get_all_ids()
for _, id in ipairs(texts) do
    {loop_pre}
    rendering.destroy(id)
    {loop_post}
end
        "
        );
        string_space_shrinker(res)
    }
}
