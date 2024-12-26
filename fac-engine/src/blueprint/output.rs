use std::{cell::RefCell, ops::Deref, rc::Rc};
use tracing::error;

use crate::admiral::{
    err::AdmiralResult,
    executor::{ExecuteResponse, LuaCompiler, client::AdmiralClient},
    lua_command::LuaCommand,
};

use super::{
    bpfac::{entity::FacBpEntity, position::FacBpPosition},
    bpitem::BlueprintItem,
    contents::BlueprintContents,
};

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
                dedupe: None,
                log_info: FacItemOutputLogInfo::new(),
            })),
        }
    }

    pub fn new_admiral_dedupe(client: AdmiralClient) -> Self {
        Self {
            otype: FacItemOutputType::AdmiralClient(RefCell::new(OutputData {
                inner: client,
                dedupe: Some(Vec::new()),
                log_info: FacItemOutputLogInfo::new(),
            })),
        }
    }

    pub fn new_blueprint() -> Self {
        Self {
            otype: FacItemOutputType::Blueprint(RefCell::new(OutputData {
                inner: BlueprintContents::new(),
                dedupe: None,
                log_info: FacItemOutputLogInfo::new(),
            })),
        }
    }

    pub fn write(&self, item: BlueprintItem) {
        let blueprint = item.to_blueprint(&self);

        self.otype.write(item, blueprint)
    }

    pub fn any_handle<'o>(&'o mut self) -> OutputContextHandle<'o> {
        OutputContextHandle {
            output: self,
            htype: OutputContextHandleType::Empty,
        }
    }

    pub fn context_handle<'o>(&'o self, new_context: String) -> OutputContextHandle<'o> {
        self.otype.push_context(new_context);
        OutputContextHandle {
            output: self,
            htype: OutputContextHandleType::Context,
        }
    }

    pub fn subcontext_handle<'o>(&'o self, new_context: String) -> OutputContextHandle<'o> {
        self.otype.push_subcontext(new_context);
        OutputContextHandle {
            output: self,
            htype: OutputContextHandleType::Subcontext,
        }
    }

    pub fn log_info(&self) -> FacItemOutputLogInfo {
        self.otype.get_context()
    }

    pub fn admiral_execute_command(
        &self,
        lua: Box<dyn LuaCommand>,
    ) -> AdmiralResult<ExecuteResponse> {
        self.otype.admiral_execute_command(lua)
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
}

enum FacItemOutputType {
    AdmiralClient(RefCell<OutputData<AdmiralClient>>),
    Blueprint(RefCell<OutputData<BlueprintContents>>),
}

impl FacItemOutputType {
    fn write(&self, item: BlueprintItem, blueprint: FacBpEntity) {
        match self {
            Self::AdmiralClient(cell) => {
                let OutputData { inner, dedupe, .. } = &mut *cell.borrow_mut();
                dedupe_position(dedupe, &blueprint);
                let res = inner.execute_checked_command(blueprint.to_lua().into_boxed());
                // all_items.push(item);
                // Vec::push() does not normally fail
                // For API sanity, do not make every FacBlk need to pass up the error
                if let Err(e) = res {
                    error!("⛔⛔⛔ Write failed {}", e);
                }
            }
            Self::Blueprint(cell) => {
                let OutputData { inner, dedupe, .. } = &mut *cell.borrow_mut();
                dedupe_position(dedupe, &blueprint);
                inner.add(item, blueprint);
            }
        }
    }

    fn push_context(&self, new_context: String) {
        match self {
            Self::AdmiralClient(cell) => cell.borrow_mut().log_info.contexts.push(new_context),
            Self::Blueprint(cell) => cell.borrow_mut().log_info.contexts.push(new_context),
        }
    }

    fn push_subcontext(&self, new_context: String) {
        match self {
            Self::AdmiralClient(cell) => cell.borrow_mut().log_info.subcontexts.push(new_context),
            Self::Blueprint(cell) => cell.borrow_mut().log_info.subcontexts.push(new_context),
        }
    }

    fn pop_context(&self) {
        match self {
            Self::AdmiralClient(cell) => cell.borrow_mut().log_info.contexts.pop(),
            Self::Blueprint(cell) => cell.borrow_mut().log_info.contexts.pop(),
        }
        .unwrap();
    }

    fn pop_subcontext(&self) {
        match self {
            Self::AdmiralClient(cell) => cell.borrow_mut().log_info.subcontexts.pop(),
            Self::Blueprint(cell) => cell.borrow_mut().log_info.subcontexts.pop(),
        }
        .unwrap();
    }

    fn get_context(&self) -> FacItemOutputLogInfo {
        match self {
            Self::AdmiralClient(cell) => cell.borrow().log_info.questitionable_clone(),
            Self::Blueprint(cell) => cell.borrow().log_info.questitionable_clone(),
        }
    }

    fn admiral_execute_command(&self, lua: Box<dyn LuaCommand>) -> AdmiralResult<ExecuteResponse> {
        match self {
            Self::AdmiralClient(cell) => cell.borrow_mut().inner.execute_checked_command(lua),
            Self::Blueprint(_) => panic!("not a admiral"),
        }
    }
}

fn dedupe_position(dedupe: &mut Option<Vec<FacBpPosition>>, blueprint: &FacBpEntity) {
    if let Some(dedupe) = dedupe {
        let bppos = &blueprint.position;
        if dedupe.contains(bppos) {
            return;
        } else {
            dedupe.push(bppos.clone());
        }
    }
}

pub struct FacItemOutputLogInfo {
    pub contexts: Vec<String>,
    pub subcontexts: Vec<String>,
}

impl FacItemOutputLogInfo {
    const fn new() -> Self {
        Self {
            contexts: Vec::new(),
            subcontexts: Vec::new(),
        }
    }

    fn questitionable_clone(&self) -> Self {
        Self {
            contexts: self.contexts.clone(),
            subcontexts: self.contexts.clone(),
        }
    }
}

struct OutputData<T> {
    inner: T,
    dedupe: Option<Vec<FacBpPosition>>,
    log_info: FacItemOutputLogInfo,
}

// Keeps the context alive for access during logging
pub struct OutputContextHandle<'o> {
    output: &'o FacItemOutput,
    htype: OutputContextHandleType,
}
impl<'o> OutputContextHandle<'o> {
    fn new_context(&'o mut self, htype: OutputContextHandleType) -> Self {
        Self {
            output: self.output,
            htype,
        }
    }
}

impl<'o> Drop for OutputContextHandle<'o> {
    fn drop(&mut self) {
        match self.htype {
            OutputContextHandleType::Context => self.output.otype.pop_context(),
            OutputContextHandleType::Subcontext => self.output.otype.pop_subcontext(),
            OutputContextHandleType::Empty => {
                // nothing
            }
        }
    }
}

impl<'o> Deref for OutputContextHandle<'o> {
    type Target = FacItemOutput;

    fn deref(&self) -> &Self::Target {
        self.output
    }
}

enum OutputContextHandleType {
    Context,
    Subcontext,
    Empty,
}
