use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LumenProgramTemplate {
    ThermalGuard,
    SalvageFlow,
    ThreatResponse,
    BalancedSupervision,
}

impl LumenProgramTemplate {
    pub const ALL: [Self; 4] = [
        Self::ThermalGuard,
        Self::SalvageFlow,
        Self::ThreatResponse,
        Self::BalancedSupervision,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ThermalGuard => "ThermalGuard",
            Self::SalvageFlow => "SalvageFlow",
            Self::ThreatResponse => "ThreatResponse",
            Self::BalancedSupervision => "BalancedSupervision",
        }
    }

    pub fn next(self) -> Self {
        let index = Self::ALL.iter().position(|item| *item == self).unwrap_or(0);
        Self::ALL[(index + 1) % Self::ALL.len()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LumenOp {
    Buff,
    Nerf,
}

impl LumenOp {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Buff => "BUFF",
            Self::Nerf => "NERF",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Buff => Self::Nerf,
            Self::Nerf => Self::Buff,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LumenTarget {
    Reactors,
    Turrets,
    Cargo,
    Processors,
    Computers,
    HotModules,
}

impl LumenTarget {
    pub const ALL: [Self; 6] = [
        Self::Reactors,
        Self::Turrets,
        Self::Cargo,
        Self::Processors,
        Self::Computers,
        Self::HotModules,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reactors => "reactors",
            Self::Turrets => "turrets",
            Self::Cargo => "cargo",
            Self::Processors => "processors",
            Self::Computers => "computers",
            Self::HotModules => "hot_modules",
        }
    }

    pub fn next(self) -> Self {
        let index = Self::ALL.iter().position(|item| *item == self).unwrap_or(0);
        Self::ALL[(index + 1) % Self::ALL.len()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LumenAspect {
    HeatCooling,
    Instability,
    Throughput,
    FireControl,
    PowerDraw,
}

impl LumenAspect {
    pub const ALL: [Self; 5] = [
        Self::HeatCooling,
        Self::Instability,
        Self::Throughput,
        Self::FireControl,
        Self::PowerDraw,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::HeatCooling => "heat_cooling",
            Self::Instability => "instability",
            Self::Throughput => "throughput",
            Self::FireControl => "fire_control",
            Self::PowerDraw => "power_draw",
        }
    }

    pub fn next(self) -> Self {
        let index = Self::ALL.iter().position(|item| *item == self).unwrap_or(0);
        Self::ALL[(index + 1) % Self::ALL.len()]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LumenInstruction {
    pub op: LumenOp,
    pub target: LumenTarget,
    pub aspect: LumenAspect,
    pub weight: u8,
}

impl LumenInstruction {
    pub fn syntax(&self) -> String {
        format!(
            "{} {} {} w{}",
            self.op.as_str(),
            self.target.as_str(),
            self.aspect.as_str(),
            self.weight
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LumenProgram {
    pub template: LumenProgramTemplate,
    pub name: String,
    pub enabled: bool,
    pub instructions: Vec<LumenInstruction>,
}

impl LumenProgram {
    pub fn from_template(template: LumenProgramTemplate) -> Self {
        match template {
            LumenProgramTemplate::ThermalGuard => Self {
                template,
                name: template.as_str().to_string(),
                enabled: true,
                instructions: vec![
                    LumenInstruction {
                        op: LumenOp::Buff,
                        target: LumenTarget::Reactors,
                        aspect: LumenAspect::HeatCooling,
                        weight: 2,
                    },
                    LumenInstruction {
                        op: LumenOp::Nerf,
                        target: LumenTarget::HotModules,
                        aspect: LumenAspect::Instability,
                        weight: 2,
                    },
                ],
            },
            LumenProgramTemplate::SalvageFlow => Self {
                template,
                name: template.as_str().to_string(),
                enabled: true,
                instructions: vec![
                    LumenInstruction {
                        op: LumenOp::Buff,
                        target: LumenTarget::Cargo,
                        aspect: LumenAspect::Throughput,
                        weight: 2,
                    },
                    LumenInstruction {
                        op: LumenOp::Buff,
                        target: LumenTarget::Processors,
                        aspect: LumenAspect::Throughput,
                        weight: 3,
                    },
                ],
            },
            LumenProgramTemplate::ThreatResponse => Self {
                template,
                name: template.as_str().to_string(),
                enabled: true,
                instructions: vec![
                    LumenInstruction {
                        op: LumenOp::Buff,
                        target: LumenTarget::Turrets,
                        aspect: LumenAspect::FireControl,
                        weight: 3,
                    },
                    LumenInstruction {
                        op: LumenOp::Nerf,
                        target: LumenTarget::Turrets,
                        aspect: LumenAspect::PowerDraw,
                        weight: 1,
                    },
                ],
            },
            LumenProgramTemplate::BalancedSupervision => Self {
                template,
                name: template.as_str().to_string(),
                enabled: true,
                instructions: vec![
                    LumenInstruction {
                        op: LumenOp::Buff,
                        target: LumenTarget::Reactors,
                        aspect: LumenAspect::HeatCooling,
                        weight: 1,
                    },
                    LumenInstruction {
                        op: LumenOp::Buff,
                        target: LumenTarget::Processors,
                        aspect: LumenAspect::Throughput,
                        weight: 2,
                    },
                    LumenInstruction {
                        op: LumenOp::Buff,
                        target: LumenTarget::Turrets,
                        aspect: LumenAspect::FireControl,
                        weight: 1,
                    },
                ],
            },
        }
    }
}
