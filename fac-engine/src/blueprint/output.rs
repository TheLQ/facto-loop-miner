use enum_map::EnumMap;
use std::{cell::RefCell, rc::Rc, sync::Mutex};
use tracing::{debug, trace};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    admiral::{
        err::AdmiralResult,
        executor::{ExecuteResponse, LuaCompiler, client::AdmiralClient},
        lua_command::LuaCommand,
    },
    common::names::FacEntityName,
    util::ansi::{
        C_BLOCK_LINE, C_FULL_BLOCK, Color, ansi_color, ansi_erase_line, ansi_previous_line,
    },
};

use super::{
    bpfac::{entity::FacBpEntity, position::FacBpPosition, tile::FacBpTile},
    bpitem::BlueprintItem,
    contents::BlueprintContents,
};

const FLAG_ENABLE_LINE_REWRITE: bool = false;
const CACHE_SIZE: usize = 1;

/// Middleware between entity output and blueprint/lua output
/// Instead of generating everything "post", obscuring errors and logic
pub struct FacItemOutput {
    otype: FacItemOutputType,
}

impl FacItemOutput {
    pub fn into_rc(self) -> Rc<Self> {
        Rc::new(self)
    }

    pub fn consume_rc(self: Rc<Self>) -> Self {
        Rc::into_inner(self).unwrap()
    }

    pub fn new_admiral(client: AdmiralClient) -> Self {
        Self {
            otype: FacItemOutputType::AdmiralClient(RefCell::new(OutputData {
                inner: client,
                dedupe: Some(Vec::new()),
                cache: Vec::new(),
                total_write: 0,
            })),
        }
    }

    pub fn new_admiral_dedupe(client: AdmiralClient) -> Self {
        Self {
            otype: FacItemOutputType::AdmiralClient(RefCell::new(OutputData {
                inner: client,
                dedupe: Some(Vec::new()),
                cache: Vec::new(),
                total_write: 0,
            })),
        }
    }

    pub fn new_blueprint() -> Self {
        Self {
            otype: FacItemOutputType::Blueprint(RefCell::new(OutputData {
                inner: BlueprintContents::new(),
                dedupe: None,
                cache: Vec::new(),
                total_write: 0,
            })),
        }
    }

    pub fn write(&self, item: BlueprintItem) {
        let blueprint = item.to_blueprint();

        let item_debug = format!("{:?}", item.entity());
        let message_pos = format!(
            "blueprint pos {:6} facpos {:10}",
            item.position().display(),
            blueprint.position.display(),
        );
        Self::log_write(item_debug, message_pos);

        self.otype
            .write(FacItemOutputWrite::Entity { item, blueprint })
    }

    pub fn write_tile(&self, blueprint: FacBpTile) {
        let item_debug = format!("{:?}", blueprint);
        let message_pos = format!("blueprint facpos {:10}", blueprint.position.display());
        Self::log_write(item_debug, message_pos);

        self.otype.write(FacItemOutputWrite::Tile { blueprint })
    }

    /// Status logs, then do actual write
    pub fn log_write(mut item_debug: String, message_pos: String) {
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

        let (contexts, subcontexts, total_with_context) = get_global_context_map(|log_info| {
            let contexts = ansi_color(
                log_info.context_map[ContextLevel::Block].join(&format!(" {C_BLOCK_LINE} ")),
                Color::Green,
            );
            let subcontexts = ansi_color(
                log_info.context_map[ContextLevel::Micro].join(" ! "),
                Color::Purple,
            );
            let key = [item_debug.as_str(), contexts.as_str(), subcontexts.as_str()].concat();
            if log_info.last_context != key {
                log_info.last_context = key;
                log_info.total_with_context = 1;
                // print!("\n");
            } else {
                log_info.total_with_context += 1;
            };
            (contexts, subcontexts, log_info.total_with_context)
        });
        let subcontexts = pad_grapheme(&subcontexts, 40);

        const EXTRA: [char; 8] = ['↑', '↗', '→', '↘', '↓', '↙', '←', '↖'];
        const PROGRESS_TRUNCATE: usize = 10;
        const UNICODE_TRUNCATE: usize = PROGRESS_TRUNCATE * 3;
        let mut total_progress = C_FULL_BLOCK.repeat(total_with_context.min(PROGRESS_TRUNCATE));
        let progress_len = total_progress.len();
        if true || progress_len == UNICODE_TRUNCATE {
            // let remain = total_with_context - PROGRESS_TRUNCATE;
            let remain = total_with_context;
            total_progress.push(EXTRA[remain % EXTRA.len()]);
        } else {
            // total_progress.push_str(&format!("m{progress_len} = {UNICODE_TRUNCATE}"));
        }

        let message_context = format!("{contexts:42} {total_with_context:2} {subcontexts}");
        let message_entity = format!("{item_debug:70}");
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
            debug!("{message_pos} {message_entity} {message_context} total {total_progress}",);
            println!("{message_pos}");
            println!("{message_context}");
            println!("{message_entity}");
        } else {
            debug!("{message_pos} {message_entity} {message_context} total {total_progress}",);
        }
    }

    pub fn flush(&self) {
        self.otype.flush_cache();
    }

    pub fn context_handle(
        &self,
        context_level: ContextLevel,
        new_context: String,
    ) -> OutputContextHandle {
        let res = OutputContextHandle {
            context_level: context_level.clone(),
        };
        get_global_context_map(move |log_info| {
            log_info.context_map[context_level.clone()].push(new_context.clone())
        });
        res
    }

    pub fn admiral_execute_command(
        &self,
        lua: Box<dyn LuaCommand>,
    ) -> AdmiralResult<ExecuteResponse> {
        self.otype.admiral_execute_command(lua)
    }

    #[cfg(test)]
    pub fn last_blueprint_write_last(&self) -> test::LastWrite {
        use test::LastWrite;

        let blueprint = &self.unwrap_blueprint().inner;
        let last_item = blueprint.items().last().unwrap();
        LastWrite {
            blueprint: blueprint.fac_entities().last().unwrap().clone(),
            size: last_item.entity().rectangle_size(),
        }
    }

    pub fn into_blueprint_contents(self) -> BlueprintContents {
        match self.otype {
            FacItemOutputType::Blueprint(inner) => {
                let output_data = inner.into_inner();
                output_data.inner
            }
            FacItemOutputType::AdmiralClient(_) => panic!("not a blueprint"),
        }
    }

    #[cfg(test)]
    fn unwrap_blueprint(&self) -> std::cell::Ref<'_, OutputData<BlueprintContents>> {
        match &self.otype {
            FacItemOutputType::Blueprint(inner) => inner.borrow(),
            FacItemOutputType::AdmiralClient(_) => panic!("not a blueprint"),
        }
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

/// TODO: lifetime soup with multiple muts is hard. New plan: Global!
fn get_global_context_map<A, R>(mut action: A) -> R
where
    A: FnMut(&mut FacItemOutputLogInfo) -> R,
{
    // thread_local! {
    static CUR: Mutex<Option<FacItemOutputLogInfo>> = Mutex::new(None);
    // }

    let mut lock = CUR.lock().unwrap();
    let map = lock.get_or_insert_default();
    action(map)
}

enum FacItemOutputType {
    AdmiralClient(RefCell<OutputData<AdmiralClient>>),
    Blueprint(RefCell<OutputData<BlueprintContents>>),
}

enum FacItemOutputWrite {
    Entity {
        item: BlueprintItem,
        blueprint: FacBpEntity,
    },
    Tile {
        blueprint: FacBpTile,
    },
}

impl FacItemOutputType {
    fn write(&self, write: FacItemOutputWrite) {
        self.push_cache(write);
        self.flush_cache_maybe();
    }

    fn push_cache(&self, write: FacItemOutputWrite) {
        match self {
            Self::AdmiralClient(cell) => {
                let OutputData { cache, .. } = &mut *cell.borrow_mut();
                cache.push(write);
            }
            Self::Blueprint(cell) => {
                let OutputData { cache, .. } = &mut *cell.borrow_mut();
                cache.push(write);
            }
        }
    }

    fn flush_cache_maybe(&self) {
        let size = match self {
            Self::AdmiralClient(cell) => {
                let OutputData { cache, .. } = &*cell.borrow();
                cache.len()
            }
            Self::Blueprint(cell) => {
                let OutputData { cache, .. } = &*cell.borrow();
                cache.len()
            }
        };
        if size < CACHE_SIZE {
            return;
        }
        self.flush_cache();
    }

    fn flush_cache(&self) {
        match self {
            Self::AdmiralClient(cell) => {
                let OutputData {
                    inner,
                    dedupe,
                    cache,
                    total_write,
                } = &mut *cell.borrow_mut();

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
                    }
                }
                let flush_count = lua_commands.len();
                if flush_count > 5 {
                    // don't spam the console on micro writes
                    trace!("Flush Cache {} total {}", flush_count, total_write)
                }
                let res = inner.execute_checked_commands_in_wrapper_function(lua_commands);

                // Vec::push() does not normally fail
                // For API sanity, do not make every FacBlk need to pass up the error
                if let Err(e) = res {
                    panic!("⛔⛔⛔ Write failed {}", e);
                }
            }
            Self::Blueprint(cell) => {
                let OutputData {
                    inner,
                    dedupe,
                    cache,
                    total_write,
                } = &mut *cell.borrow_mut();
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
                    }
                }
                *total_write += flush_count;

                if flush_count > 2 {
                    // don't spam the console on micro writes
                    trace!("Flush Cache {} total {}", flush_count, total_write)
                }
            }
        }
    }

    // fn push_context(&self, context_level: ContextLevel, new_context: String) {
    //     match self {
    //         Self::AdmiralClient(cell) => {
    //             cell.borrow_mut().log_info.context_map[context_level].push(new_context)
    //         }
    //         Self::Blueprint(cell) => {
    //             cell.borrow_mut().log_info.context_map[context_level].push(new_context)
    //         }
    //     }
    // }

    // fn pop_context(&self, context_level: ContextLevel) {
    //     match self {
    //         Self::AdmiralClient(cell) => {
    //             cell.borrow_mut().log_info.context_map[context_level].pop()
    //         }
    //         Self::Blueprint(cell) => cell.borrow_mut().log_info.context_map[context_level].pop(),
    //     }
    //     .unwrap();
    // }

    // fn get_context(&self) -> FacItemOutputLogInfo {
    //     match self {
    //         Self::AdmiralClient(cell) => cell.borrow().log_info.questitionable_clone(),
    //         Self::Blueprint(cell) => cell.borrow().log_info.questitionable_clone(),
    //     }
    // }

    fn admiral_execute_command(&self, lua: Box<dyn LuaCommand>) -> AdmiralResult<ExecuteResponse> {
        match self {
            Self::AdmiralClient(cell) => cell.borrow_mut().inner.execute_checked_command(lua),
            Self::Blueprint(_) => panic!("not a admiral"),
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
                panic!("dupe {:?}", blueprint);
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

struct OutputData<T> {
    inner: T,
    dedupe: Option<Vec<FacBpPosition>>,
    cache: Vec<FacItemOutputWrite>,
    total_write: usize,
}

// Keeps the context alive for access during logging
pub struct OutputContextHandle {
    context_level: ContextLevel,
}

impl Drop for OutputContextHandle {
    fn drop(&mut self) {
        get_global_context_map(|log_info| log_info.context_map[self.context_level.clone()].pop());
    }
}

// impl Deref for OutputContextHandle {
//     type Target = FacItemOutput;

//     fn deref(&self) -> &Self::Target {
//         self.output
//     }
// }
