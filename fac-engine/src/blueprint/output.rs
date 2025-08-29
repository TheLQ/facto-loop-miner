use super::{
    bpfac::{entity::FacBpEntity, position::FacBpPosition, tile::FacBpTile},
    bpitem::BlueprintItem,
    contents::BlueprintContents,
};
use crate::admiral::err::pretty_panic_admiral;
use crate::admiral::lua_command::fac_render_text::FacRenderText;
use crate::blueprint::converter::{ConvertResult, encode_blueprint_to_string_auto_index};
use crate::{
    admiral::{
        err::AdmiralResult,
        executor::{ExecuteResponse, LuaCompiler, client::AdmiralClient},
        lua_command::LuaCommand,
    },
    common::{entity::FacEntity, names::FacEntityName, vpoint::VPoint},
    util::ansi::{
        C_BLOCK_LINE, C_FULL_BLOCK, Color, ansi_color, ansi_erase_line, ansi_previous_line,
    },
};
use enum_map::EnumMap;
use itertools::Itertools;
use std::{cell::RefCell, rc::Rc};
use tracing::{debug, trace};
use unicode_segmentation::UnicodeSegmentation;

const FLAG_ENABLE_RENDER_TEXT: bool = true;
const FLAG_ENABLE_LINE_REWRITE: bool = false;
const CACHE_SIZE: usize = 9898989;

/// Middleware between entity output and blueprint/lua output
/// Instead of generating everything "post", obscuring errors and logic
pub struct FacItemOutput {
    odata: RefCell<FacItemOutputData>,
}

impl FacItemOutput {
    pub fn into_rc(self) -> Rc<Self> {
        Rc::new(self)
    }

    pub fn consume_rc(self: Rc<Self>) -> Self {
        Rc::into_inner(self).expect("Output Rc somewhere. Need to Drop?")
    }

    pub fn new_admiral(client: AdmiralClient) -> Self {
        Self {
            odata: RefCell::new(FacItemOutputData {
                otype: FacItemOutputType::AdmiralClient(client),
                dedupe: None,
                cache: Vec::new(),
                total_write: 0,
                contexts: Default::default(),
            }),
        }
    }

    pub fn new_admiral_dedupe(client: AdmiralClient) -> Self {
        Self {
            odata: RefCell::new(FacItemOutputData {
                otype: FacItemOutputType::AdmiralClient(client),
                dedupe: Some(Vec::new()),
                cache: Vec::new(),
                total_write: 0,
                contexts: Default::default(),
            }),
        }
    }

    pub fn new_blueprint() -> Self {
        Self {
            odata: RefCell::new(FacItemOutputData {
                otype: FacItemOutputType::Blueprint(BlueprintContents::new()),
                dedupe: None,
                cache: Vec::new(),
                total_write: 0,
                contexts: Default::default(),
            }),
        }
    }

    pub fn new_null() -> Self {
        Self {
            odata: RefCell::new(FacItemOutputData {
                otype: FacItemOutputType::Null,
                dedupe: None,
                cache: Vec::new(),
                total_write: 0,
                contexts: Default::default(),
            }),
        }
    }

    pub fn writei(&self, entity: impl FacEntity + 'static, position: VPoint) {
        self.write(BlueprintItem::newb(entity, position))
    }

    pub fn write(&self, item: BlueprintItem) {
        if let FacItemOutputType::Null = self.odata.borrow().otype {
            return;
        }
        let mut odata = self.odata.borrow_mut();

        let blueprint = item.to_blueprint();

        let item_debug = format!("{:?}", item.entity());
        let message_pos = format!(
            "blueprint pos {} facpos {}",
            item.position(),
            blueprint.position,
        );
        if FLAG_ENABLE_RENDER_TEXT {
            Self::render_context(&mut odata, &item);
        }

        Self::log_write(&mut odata.contexts, item_debug, message_pos);
        odata.write(FacItemOutputWrite::Entity { item, blueprint })
    }

    pub fn write_tile(&self, blueprint: FacBpTile) {
        if let FacItemOutputType::Null = self.odata.borrow().otype {
            return;
        }
        let item_debug = format!("{blueprint:?}");
        let message_pos = format!("blueprint facpos {}", blueprint.position);

        let mut odata = self.odata.borrow_mut();
        Self::log_write(&mut odata.contexts, item_debug, message_pos);
        odata.write(FacItemOutputWrite::Tile { blueprint })
    }

    /// Status logs, then do actual write
    pub fn log_write(
        log_info: &mut FacItemOutputLogInfo,
        mut item_debug: String,
        message_pos: String,
    ) {
        // Shorten by removing keys, their obvious
        let item_debug_str = item_debug.as_bytes().to_vec();
        for end in (0..item_debug.len()).rev() {
            if item_debug_str[end] == b':' {
                // find all chars
                let mut start = 0;
                for cur_start in (0..end).rev() {
                    let cur_char: char = item_debug_str[cur_start].into();
                    if cur_char.is_alphabetic() {
                        start = cur_start;
                    } else {
                        break;
                    }
                }
                // space
                start -= 1;
                // include the colon
                let end = end + 1;

                item_debug.replace_range(start..end, "");
            }
        }
        const ITEM_DEBUG_MAX: usize = 40;
        const ITEM_DEBUG_MAX_PRE_ELIPSIS: usize = ITEM_DEBUG_MAX - 3;
        if item_debug.len() > ITEM_DEBUG_MAX_PRE_ELIPSIS {
            item_debug.replace_range(ITEM_DEBUG_MAX_PRE_ELIPSIS.., "");
            item_debug.push_str("...");
        }

        let (contexts, subcontexts, total_with_context) = {
            let contexts = ansi_color(
                log_info.context_map[ContextLevel::Block].join(&format!(" {C_BLOCK_LINE} ")),
                Color::Green,
            );
            let subcontexts = ansi_color(
                log_info.context_map[ContextLevel::Micro].join(" ! "),
                Color::Purple,
            );
            let key = [item_debug.as_str(), contexts.as_str(), subcontexts.as_str()].concat();
            if log_info.last_context == key {
                log_info.total_with_context += 1;
            } else {
                log_info.last_context = key;
                log_info.total_with_context = 1;
                // print!("\n");
            };
            (contexts, subcontexts, log_info.total_with_context)
        };
        let subcontexts = pad_grapheme(&subcontexts, 40);

        const EXTRA: [char; 8] = ['↑', '↗', '→', '↘', '↓', '↙', '←', '↖'];
        const PROGRESS_TRUNCATE: usize = 10;
        const UNICODE_TRUNCATE: usize = PROGRESS_TRUNCATE * 3;
        let mut total_progress = C_FULL_BLOCK.repeat(total_with_context.min(PROGRESS_TRUNCATE));
        let progress_len = total_progress.len();
        if progress_len == UNICODE_TRUNCATE {
            // let remain = total_with_context - PROGRESS_TRUNCATE;
            let remain = total_with_context;
            total_progress.insert(0, EXTRA[remain % EXTRA.len()]);
        } else {
            // total_progress.push_str(&format!("m{progress_len} = {UNICODE_TRUNCATE}"));
        }

        let message_context = format!("{contexts:42} {total_with_context:2} {subcontexts}");
        let message_entity = format!("{item_debug:ITEM_DEBUG_MAX$}");
        // if message_entity.len() > 70 {
        //     message_entity = message_entity[..70].to_string();
        // }

        if FLAG_ENABLE_LINE_REWRITE {
            // push terminal down
            println!();

            // go up 3 lines (debug area) + log line
            print!(
                "{prev}{reset}{prev}{reset}{prev}{reset}{prev}{reset}",
                prev = ansi_previous_line(),
                reset = ansi_erase_line()
            );
            debug!("{message_pos} {message_entity} {message_context} {total_progress}",);
            println!("{message_pos}");
            println!("{message_context}");
            println!("{message_entity}");
        } else {
            debug!("{message_pos} {message_entity} {message_context} {total_progress}",);
        }
    }

    fn render_context(odata: &mut FacItemOutputData, item: &BlueprintItem) {
        const MAX_LINE_LEN: usize = 15;

        let mut line_num = 0;
        let local_context_map = odata.contexts.context_map.clone();
        for (level, blocks) in local_context_map {
            let color = match level {
                ContextLevel::Block => [19, 161, 14],
                ContextLevel::Micro => [19, 171, 146],
            };
            for block in blocks {
                // let mut text = block.clone();
                // text.truncate(4);

                let mut lines = block
                    .split("-")
                    .filter(|v| !v.is_empty())
                    .map(str::to_string)
                    .collect_vec();
                for i in (0..lines.len()).rev() {
                    if let Some(next) = lines.get(i + 1)
                        && next.parse::<u32>().is_ok()
                    {
                        lines[i] = [&lines[i], "-", next].concat();
                        lines.remove(i + 1);
                    }
                }
                for i in (0..lines.len()).rev() {
                    let cur = &lines[i];
                    if let Some(next) = lines.get(i + 1)
                        && next.len() + cur.len() < MAX_LINE_LEN
                    {
                        lines[i] = [cur, "-", next].concat();
                        lines.remove(i + 1);
                    }
                }

                for line in lines {
                    odata.write(FacItemOutputWrite::RenderText {
                        render: FacRenderText::text(
                            line,
                            item.position()
                                .to_fac_exact()
                                .move_y(0.25 * (line_num as f32)),
                        )
                        .with_scale(0.5)
                        .with_color(color),
                    });
                    line_num += 1;
                }
            }
        }
    }

    pub fn flush(&self) {
        let mut odata = self.odata.borrow_mut();
        odata.flush_cache();
    }

    pub fn context_handle(
        self: &Rc<Self>,
        context_level: ContextLevel,
        new_context: String,
    ) -> OutputContextHandle {
        let handle = OutputContextHandle {
            context_level: context_level.clone(),
            output: self.clone(),
        };
        let mut odata = self.odata.borrow_mut();
        odata.contexts.context_map[context_level].push(new_context);
        handle
    }

    pub fn admiral_execute_command(
        &self,
        lua: Box<dyn LuaCommand>,
    ) -> AdmiralResult<ExecuteResponse> {
        let mut odata = self.odata.borrow_mut();
        odata.admiral_execute_command(lua)
    }

    #[cfg(test)]
    pub fn last_blueprint_write(&self) -> test::LastWrite {
        use test::LastWrite;

        let odata = self.odata.borrow();
        if let FacItemOutputType::Blueprint(blueprint) = &odata.otype {
            let last_item = blueprint.items().last().unwrap();
            LastWrite {
                blueprint: blueprint.fac_entities().last().unwrap().clone(),
                size: last_item.entity().rectangle_size(),
            }
        } else {
            panic!("Not a blueprint")
        }
    }

    pub fn into_blueprint_contents(self) -> BlueprintContents {
        let odata = self.odata.into_inner();
        match odata.otype {
            FacItemOutputType::Blueprint(inner) => inner,
            FacItemOutputType::AdmiralClient(_) | FacItemOutputType::Null => {
                panic!("not a blueprint")
            }
        }
    }

    pub fn into_blueprint_string(self) -> ConvertResult<String> {
        let odata = self.odata.into_inner();
        let bp = match odata.otype {
            FacItemOutputType::Blueprint(inner) => inner,
            FacItemOutputType::AdmiralClient(_) | FacItemOutputType::Null => {
                panic!("not a blueprint")
            }
        };
        encode_blueprint_to_string_auto_index(bp.into_bp())
    }
}

#[cfg(test)]
mod test {
    use crate::{blueprint::bpfac::entity::FacBpEntity, common::entity::Size};

    pub struct LastWrite {
        pub blueprint: FacBpEntity,
        pub size: Size,
    }
}

fn pad_grapheme(input: &str, up_to: usize) -> String {
    let actual_len: usize = input
        .graphemes(false)
        .map(|v| if v.len() > 1 { 2 } else { 1 })
        .sum();
    let diff = up_to.saturating_sub(actual_len);
    if diff > 0 {
        format!("{}{:diff$}", input, "")
    } else {
        input.to_string()
    }
}

#[derive(Clone, enum_map::Enum)]
pub enum ContextLevel {
    Block,
    Micro,
}

struct FacItemOutputData {
    otype: FacItemOutputType,
    dedupe: Option<Vec<FacBpPosition>>,
    cache: Vec<FacItemOutputWrite>,
    total_write: usize,
    contexts: FacItemOutputLogInfo,
}

enum FacItemOutputType {
    AdmiralClient(AdmiralClient),
    Blueprint(BlueprintContents),
    Null,
}

enum FacItemOutputWrite {
    Entity {
        item: BlueprintItem,
        blueprint: FacBpEntity,
    },
    Tile {
        blueprint: FacBpTile,
    },
    RenderText {
        render: FacRenderText,
    },
}

impl FacItemOutputData {
    fn write(&mut self, write: FacItemOutputWrite) {
        self.push_cache(write);
        self.flush_cache_maybe();
    }

    fn push_cache(&mut self, write: FacItemOutputWrite) {
        self.cache.push(write);
    }

    fn flush_cache_maybe(&mut self) {
        let size = self.cache.len();
        if size < CACHE_SIZE {
            return;
        }
        self.flush_cache();
    }

    fn flush_cache(&mut self) {
        let Self {
            otype,
            dedupe,
            cache,
            total_write,
            contexts: _,
        } = self;
        match otype {
            FacItemOutputType::AdmiralClient(inner) => {
                let mut lua_commands = Vec::new();
                for write in cache.drain(0..) {
                    *total_write += 1;

                    match write {
                        FacItemOutputWrite::Entity { item, blueprint } => {
                            dedupe_position(dedupe, &item, &blueprint);
                            lua_commands.push(blueprint.to_lua().into_boxed());
                        }
                        FacItemOutputWrite::Tile { blueprint } => {
                            lua_commands.push(blueprint.to_lua().into_boxed());
                        }
                        FacItemOutputWrite::RenderText { render } => {
                            lua_commands.push(render.into_boxed());
                        }
                    }
                }
                let flush_count = lua_commands.len();
                if flush_count > 5 {
                    // don't spam the console on micro writes
                    trace!("Flush Cache {} total {}", flush_count, total_write)
                }
                let res = if flush_count == 1 {
                    let lua_command = lua_commands.remove(0);
                    inner.execute_checked_command(lua_command).map(|_| ())
                } else {
                    inner.execute_checked_commands_in_wrapper_function(lua_commands)
                };

                // Vec::push() does not normally fail
                // For API sanity, do not make every FacBlk need to pass up the error
                if let Err(e) = res {
                    pretty_panic_admiral(e);
                }
            }
            FacItemOutputType::Blueprint(inner) => {
                let mut flush_count = 0;
                for write in cache.drain(0..) {
                    flush_count += 1;

                    match write {
                        FacItemOutputWrite::Entity { item, blueprint } => {
                            dedupe_position(dedupe, &item, &blueprint);
                            inner.add(item, blueprint);
                        }
                        FacItemOutputWrite::Tile { blueprint } => {
                            inner.add_tile(blueprint);
                        }
                        FacItemOutputWrite::RenderText { render: _ } => {
                            // todo: can't do this in a blueprint
                        }
                    }
                }
                *total_write += flush_count;

                if flush_count > 2 {
                    // don't spam the console on micro writes
                    trace!("Flush Cache {} total {}", flush_count, total_write)
                }
            }
            FacItemOutputType::Null => {
                panic!("should not be here")
            }
        }
    }

    fn admiral_execute_command(
        &mut self,
        lua: Box<dyn LuaCommand>,
    ) -> AdmiralResult<ExecuteResponse> {
        match &mut self.otype {
            FacItemOutputType::AdmiralClient(inner) => inner.execute_checked_command(lua),
            FacItemOutputType::Blueprint(_) | FacItemOutputType::Null => panic!("not a admiral"),
        }
    }
}

fn dedupe_position(
    dedupe: &mut Option<Vec<FacBpPosition>>,
    item: &BlueprintItem,
    blueprint: &FacBpEntity,
) {
    if let Some(dedupe) = dedupe {
        let bppos = &blueprint.position;
        if dedupe.contains(bppos) {
            // hmm...

            if matches!(
                item.entity().name(),
                FacEntityName::Locomotive | FacEntityName::CargoWagon
            ) {
                // initially dedupe
            } else {
                tracing::warn!("dupe {:?}", blueprint);
            }
        } else {
            dedupe.push(bppos.clone());
        }
    }
}

#[derive(Default)]
pub struct FacItemOutputLogInfo {
    pub context_map: EnumMap<ContextLevel, Vec<String>>,
    pub last_context: String,
    pub total_with_context: usize,
}

// Keeps the context alive for access during logging
pub struct OutputContextHandle {
    context_level: ContextLevel,
    output: Rc<FacItemOutput>,
}

impl Drop for OutputContextHandle {
    fn drop(&mut self) {
        let mut data = self.output.odata.borrow_mut();
        data.contexts.context_map[self.context_level.clone()].pop();
    }
}

// impl Deref for OutputContextHandle {
//     type Target = FacItemOutput;

//     fn deref(&self) -> &Self::Target {
//         self.output
//     }
// }
