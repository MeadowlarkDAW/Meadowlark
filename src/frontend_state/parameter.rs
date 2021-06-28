#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gradient {
    Linear,
    Power(f32),
    Exponential,
}

#[derive(Debug, Clone, Copy)]
pub enum Type {
    Numeric {
        min: f32,
        max: f32,

        gradient: Gradient,
    },
    // eventually will have an Enum/Discrete type here
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unit {
    Generic,
    Decibels,
}

pub struct Param {
    pub unit: Unit,
    pub param_type: Type,
}

impl Param {}
