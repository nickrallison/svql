use crate::{IoValue, Net, TargetPrototype, Value};

use crate::ParamValue;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TargetCell {
    pub kind: String,
    pub params: Vec<ParamValue>,
    pub inputs: Value,
    pub output_len: usize,
    pub ios: IoValue,
}

impl TargetCell {
    pub fn new(kind: impl Into<String>, prototype: &TargetPrototype) -> Self {
        let mut result = Self {
            kind: kind.into(),
            params: vec![],
            inputs: Value::new(),
            output_len: prototype.output_len,
            ios: IoValue::floating(prototype.io_len),
        };
        for param in &prototype.params {
            result.params.push(param.default.clone());
        }
        for input in &prototype.inputs {
            result.inputs.extend(Value::from(&input.default));
        }
        result
    }

    pub fn visit(&self, f: impl FnMut(Net)) {
        self.inputs.visit(f);
    }

    pub fn visit_mut(&mut self, f: impl FnMut(&mut Net)) {
        self.inputs.visit_mut(f);
    }
}
