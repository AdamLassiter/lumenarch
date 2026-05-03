use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LumenParseDiagnostic {
    pub line: usize,
    pub message: String,
}

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
    #[serde(default)]
    pub source_text: String,
    pub instructions: Vec<LumenInstruction>,
}

impl LumenProgram {
    pub fn from_template(template: LumenProgramTemplate) -> Self {
        let mut program = match template {
            LumenProgramTemplate::ThermalGuard => Self {
                template,
                name: template.as_str().to_string(),
                enabled: true,
                source_text: String::new(),
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
                source_text: String::new(),
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
                source_text: String::new(),
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
                source_text: String::new(),
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
        };
        program.refresh_source_text();
        program
    }

    pub fn refresh_source_text(&mut self) {
        self.source_text = lumen_program_to_source(&self.instructions);
    }

    pub fn compile_source_text(
        &mut self,
        source_text: &str,
    ) -> Result<(), Vec<LumenParseDiagnostic>> {
        let instructions = parse_lumen_program(source_text)?;
        self.instructions = instructions;
        self.source_text = source_text.to_string();
        Ok(())
    }

    pub fn validate_source_text(
        source_text: &str,
    ) -> Result<Vec<LumenInstruction>, Vec<LumenParseDiagnostic>> {
        parse_lumen_program(source_text)
    }
}

pub fn lumen_program_to_source(instructions: &[LumenInstruction]) -> String {
    instructions
        .iter()
        .map(LumenInstruction::syntax)
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn parse_lumen_program(
    source_text: &str,
) -> Result<Vec<LumenInstruction>, Vec<LumenParseDiagnostic>> {
    let mut instructions = Vec::new();
    let mut diagnostics = Vec::new();

    for (line_index, raw_line) in source_text.lines().enumerate() {
        let line_number = line_index + 1;
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }

        let tokens: Vec<_> = line.split_whitespace().collect();
        if tokens.len() != 4 {
            diagnostics.push(LumenParseDiagnostic {
                line: line_number,
                message: "LUMEN directives must be 'BUFF|NERF target aspect weight'".to_string(),
            });
            continue;
        }

        let op = match tokens[0].to_ascii_uppercase().as_str() {
            "BUFF" => LumenOp::Buff,
            "NERF" => LumenOp::Nerf,
            other => {
                diagnostics.push(LumenParseDiagnostic {
                    line: line_number,
                    message: format!("unknown LUMEN opcode '{other}'"),
                });
                continue;
            }
        };
        let target = match parse_lumen_target(tokens[1]) {
            Some(target) => target,
            None => {
                diagnostics.push(LumenParseDiagnostic {
                    line: line_number,
                    message: format!("unknown LUMEN target '{}'", tokens[1]),
                });
                continue;
            }
        };
        let aspect = match parse_lumen_aspect(tokens[2]) {
            Some(aspect) => aspect,
            None => {
                diagnostics.push(LumenParseDiagnostic {
                    line: line_number,
                    message: format!("unknown LUMEN aspect '{}'", tokens[2]),
                });
                continue;
            }
        };
        let weight_token = tokens[3].trim_start_matches('w').trim_start_matches('W');
        let Ok(weight) = weight_token.parse::<u8>() else {
            diagnostics.push(LumenParseDiagnostic {
                line: line_number,
                message: format!("invalid LUMEN weight '{}'", tokens[3]),
            });
            continue;
        };
        instructions.push(LumenInstruction {
            op,
            target,
            aspect,
            weight: weight.min(9),
        });
    }

    if diagnostics.is_empty() {
        Ok(instructions)
    } else {
        Err(diagnostics)
    }
}

fn parse_lumen_target(token: &str) -> Option<LumenTarget> {
    match token.to_ascii_lowercase().as_str() {
        "reactors" => Some(LumenTarget::Reactors),
        "turrets" => Some(LumenTarget::Turrets),
        "cargo" => Some(LumenTarget::Cargo),
        "processors" => Some(LumenTarget::Processors),
        "computers" => Some(LumenTarget::Computers),
        "hot_modules" => Some(LumenTarget::HotModules),
        _ => None,
    }
}

fn parse_lumen_aspect(token: &str) -> Option<LumenAspect> {
    match token.to_ascii_lowercase().as_str() {
        "heat_cooling" => Some(LumenAspect::HeatCooling),
        "instability" => Some(LumenAspect::Instability),
        "throughput" => Some(LumenAspect::Throughput),
        "fire_control" => Some(LumenAspect::FireControl),
        "power_draw" => Some(LumenAspect::PowerDraw),
        _ => None,
    }
}
