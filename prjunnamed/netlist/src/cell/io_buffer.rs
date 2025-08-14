use crate::{ControlNet, IoValue, Net, Value};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IoBuffer {
    pub io: IoValue,
    pub output: Value,
    pub enable: ControlNet,
}

impl IoBuffer {
    pub fn output_len(&self) -> usize {
        self.io.len()
    }

    pub fn slice(&self, range: impl std::ops::RangeBounds<usize> + Clone) -> IoBuffer {
        IoBuffer { io: self.io.slice(range.clone()), output: self.output.slice(range.clone()), enable: self.enable }
    }

    pub fn visit(&self, mut f: impl FnMut(Net)) {
        self.output.visit(&mut f);
        self.enable.visit(&mut f);
    }

    pub fn visit_mut(&mut self, mut f: impl FnMut(&mut Net)) {
        self.output.visit_mut(&mut f);
        self.enable.visit_mut(&mut f);
    }
}
