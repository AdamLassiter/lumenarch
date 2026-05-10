use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchParseDiagnostic {
    pub line: usize,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArchRegister {
    Gp0,
    Gp1,
    Gp2,
    Gp3,
    ShipPowerReserve,
    ShipAverageHeat,
    MissionThreat,
    ReactorReactionRate,
    ReactorTurbineLoad,
    ReactorHeat,
    ReactorInstability,
    ReactorPowerOutput,
    StorageRawSalvage,
    StorageRepairCharge,
    ProcessorRawSalvage,
    ProcessorRepairCharge,
    TurretReady,
    TurretCooldown,
    DetectLifeFriendly,
    DetectLifeHostile,
    DetectLifeDirX,
    DetectLifeDirY,
    DetectLifeDistance,
    DetectShipNearby,
    DetectShipHostile,
    DetectShipDirX,
    DetectShipDirY,
    DetectShipDistance,
    DetectDamageIncoming,
    DetectDamageCritical,
    DetectDamageDirX,
    DetectDamageDirY,
    DetectDamageIntensity,
    DetectPowerDeficit,
    DetectPowerLowBattery,
    DetectHeatAlert,
    DetectHeatDirX,
    DetectHeatDirY,
    DetectHeatSeverity,
    DetectLogisticsDemand,
    DetectLogisticsDirX,
    DetectLogisticsDirY,
    DetectLogisticsSeverity,
    CmdReactorBias,
    CmdLogisticsEnable,
    CmdLogisticsPreference,
    CmdTurretAssist,
    CmdTurretAutoFire,
}

impl ArchRegister {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gp0 => "GP0",
            Self::Gp1 => "GP1",
            Self::Gp2 => "GP2",
            Self::Gp3 => "GP3",
            Self::ShipPowerReserve => "VPR",
            Self::ShipAverageHeat => "VHT",
            Self::MissionThreat => "VTHR",
            Self::ReactorReactionRate => "RRF0",
            Self::ReactorTurbineLoad => "RRT0",
            Self::ReactorHeat => "RRH0",
            Self::ReactorInstability => "RRS0",
            Self::ReactorPowerOutput => "RRP0",
            Self::StorageRawSalvage => "LSR0",
            Self::StorageRepairCharge => "LSR1",
            Self::ProcessorRawSalvage => "LPR0",
            Self::ProcessorRepairCharge => "LPR1",
            Self::TurretReady => "WTR0",
            Self::TurretCooldown => "WTC0",
            Self::DetectLifeFriendly => "DLF0",
            Self::DetectLifeHostile => "DLH0",
            Self::DetectLifeDirX => "DLX0",
            Self::DetectLifeDirY => "DLY0",
            Self::DetectLifeDistance => "DLD0",
            Self::DetectShipNearby => "DSN0",
            Self::DetectShipHostile => "DSH0",
            Self::DetectShipDirX => "DSX0",
            Self::DetectShipDirY => "DSY0",
            Self::DetectShipDistance => "DSD0",
            Self::DetectDamageIncoming => "DDM0",
            Self::DetectDamageCritical => "DDC0",
            Self::DetectDamageDirX => "DDX0",
            Self::DetectDamageDirY => "DDY0",
            Self::DetectDamageIntensity => "DDI0",
            Self::DetectPowerDeficit => "DPP0",
            Self::DetectPowerLowBattery => "DPB0",
            Self::DetectHeatAlert => "DHH0",
            Self::DetectHeatDirX => "DHX0",
            Self::DetectHeatDirY => "DHY0",
            Self::DetectHeatSeverity => "DHI0",
            Self::DetectLogisticsDemand => "DLG0",
            Self::DetectLogisticsDirX => "DLX1",
            Self::DetectLogisticsDirY => "DLY1",
            Self::DetectLogisticsSeverity => "DLI0",
            Self::CmdReactorBias => "RRC0",
            Self::CmdLogisticsEnable => "LMC0",
            Self::CmdLogisticsPreference => "LMP0",
            Self::CmdTurretAssist => "WTA0",
            Self::CmdTurretAutoFire => "WTF0",
        }
    }

    pub fn is_writable(self) -> bool {
        matches!(
            self,
            Self::Gp0
                | Self::Gp1
                | Self::Gp2
                | Self::Gp3
                | Self::CmdReactorBias
                | Self::CmdLogisticsEnable
                | Self::CmdLogisticsPreference
                | Self::CmdTurretAssist
                | Self::CmdTurretAutoFire
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchValueRef {
    Register(ArchRegister),
    Immediate(i32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchInstruction {
    Nop,
    Mov {
        dst: ArchRegister,
        src: ArchValueRef,
    },
    Clp {
        dst: ArchRegister,
        value: ArchValueRef,
        min: ArchValueRef,
        max: ArchValueRef,
    },
    Abs {
        dst: ArchRegister,
        src: ArchValueRef,
    },
    Neg {
        dst: ArchRegister,
        src: ArchValueRef,
    },
    Add {
        dst: ArchRegister,
        lhs: ArchValueRef,
        rhs: ArchValueRef,
    },
    Sub {
        dst: ArchRegister,
        lhs: ArchValueRef,
        rhs: ArchValueRef,
    },
    Mul {
        dst: ArchRegister,
        lhs: ArchValueRef,
        rhs: ArchValueRef,
    },
    Div {
        dst: ArchRegister,
        lhs: ArchValueRef,
        rhs: ArchValueRef,
    },
    Mod {
        dst: ArchRegister,
        lhs: ArchValueRef,
        rhs: ArchValueRef,
    },
    Pow {
        dst: ArchRegister,
        lhs: ArchValueRef,
        rhs: ArchValueRef,
    },
    Sqrt {
        dst: ArchRegister,
        src: ArchValueRef,
    },
    Log {
        dst: ArchRegister,
        src: ArchValueRef,
    },
    Sin {
        dst: ArchRegister,
        src: ArchValueRef,
    },
    Cos {
        dst: ArchRegister,
        src: ArchValueRef,
    },
    Tan {
        dst: ArchRegister,
        src: ArchValueRef,
    },
    Asin {
        dst: ArchRegister,
        src: ArchValueRef,
    },
    Acos {
        dst: ArchRegister,
        src: ArchValueRef,
    },
    Atan {
        dst: ArchRegister,
        src: ArchValueRef,
    },
    Atn2 {
        dst: ArchRegister,
        y: ArchValueRef,
        x: ArchValueRef,
    },
    Gt {
        dst: ArchRegister,
        lhs: ArchValueRef,
        rhs: ArchValueRef,
    },
    Gte {
        dst: ArchRegister,
        lhs: ArchValueRef,
        rhs: ArchValueRef,
    },
    Lt {
        dst: ArchRegister,
        lhs: ArchValueRef,
        rhs: ArchValueRef,
    },
    Lte {
        dst: ArchRegister,
        lhs: ArchValueRef,
        rhs: ArchValueRef,
    },
    Eq {
        dst: ArchRegister,
        lhs: ArchValueRef,
        rhs: ArchValueRef,
    },
    Neq {
        dst: ArchRegister,
        lhs: ArchValueRef,
        rhs: ArchValueRef,
    },
    And {
        dst: ArchRegister,
        lhs: ArchValueRef,
        rhs: ArchValueRef,
    },
    Or {
        dst: ArchRegister,
        lhs: ArchValueRef,
        rhs: ArchValueRef,
    },
    Xor {
        dst: ArchRegister,
        lhs: ArchValueRef,
        rhs: ArchValueRef,
    },
    Not {
        dst: ArchRegister,
        src: ArchValueRef,
    },
    Jmp {
        target: usize,
    },
    Jeq {
        lhs: ArchValueRef,
        rhs: ArchValueRef,
        target: usize,
    },
    Jne {
        lhs: ArchValueRef,
        rhs: ArchValueRef,
        target: usize,
    },
    Jgt {
        lhs: ArchValueRef,
        rhs: ArchValueRef,
        target: usize,
    },
    Jge {
        lhs: ArchValueRef,
        rhs: ArchValueRef,
        target: usize,
    },
    Jlt {
        lhs: ArchValueRef,
        rhs: ArchValueRef,
        target: usize,
    },
    Jle {
        lhs: ArchValueRef,
        rhs: ArchValueRef,
        target: usize,
    },
    Jnz {
        cond: ArchValueRef,
        target: usize,
    },
    Min {
        dst: ArchRegister,
        lhs: ArchValueRef,
        rhs: ArchValueRef,
    },
    Max {
        dst: ArchRegister,
        lhs: ArchValueRef,
        rhs: ArchValueRef,
    },
    Avg {
        dst: ArchRegister,
        lhs: ArchValueRef,
        rhs: ArchValueRef,
    },
    Sgn {
        dst: ArchRegister,
        src: ArchValueRef,
    },
    Lerp {
        dst: ArchRegister,
        start: ArchValueRef,
        end: ArchValueRef,
        factor: ArchValueRef,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchProgramTemplate {
    ReactorGuard,
    LogisticsFeed,
    TurretAssist,
    BalancedOps,
}

impl ArchProgramTemplate {
    pub const ALL: [Self; 4] = [
        Self::ReactorGuard,
        Self::LogisticsFeed,
        Self::TurretAssist,
        Self::BalancedOps,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReactorGuard => "ReactorGuard",
            Self::LogisticsFeed => "LogisticsFeed",
            Self::TurretAssist => "TurretAssist",
            Self::BalancedOps => "BalancedOps",
        }
    }

    pub fn next(self) -> Self {
        let index = Self::ALL
            .iter()
            .position(|template| *template == self)
            .unwrap_or(0);
        Self::ALL[(index + 1) % Self::ALL.len()]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchProgram {
    pub template: ArchProgramTemplate,
    pub name: String,
    pub constants: [i32; 2],
    #[serde(default)]
    pub source_text: String,
    pub instructions: Vec<ArchInstruction>,
}

impl ArchProgram {
    pub fn from_template(template: ArchProgramTemplate) -> Self {
        let mut program = match template {
            ArchProgramTemplate::ReactorGuard => Self {
                template,
                name: template.as_str().to_string(),
                constants: [9, 7],
                source_text: String::new(),
                instructions: vec![
                    ArchInstruction::Mov {
                        dst: ArchRegister::CmdReactorBias,
                        src: ArchValueRef::Immediate(0),
                    },
                    ArchInstruction::Gt {
                        dst: ArchRegister::Gp0,
                        lhs: ArchValueRef::Register(ArchRegister::ReactorHeat),
                        rhs: ArchValueRef::Immediate(9),
                    },
                    ArchInstruction::Jnz {
                        cond: ArchValueRef::Register(ArchRegister::Gp0),
                        target: 6,
                    },
                    ArchInstruction::Gt {
                        dst: ArchRegister::Gp1,
                        lhs: ArchValueRef::Register(ArchRegister::ReactorInstability),
                        rhs: ArchValueRef::Immediate(7),
                    },
                    ArchInstruction::Jnz {
                        cond: ArchValueRef::Register(ArchRegister::Gp1),
                        target: 6,
                    },
                    ArchInstruction::Jmp { target: 7 },
                    ArchInstruction::Mov {
                        dst: ArchRegister::CmdReactorBias,
                        src: ArchValueRef::Immediate(3),
                    },
                ],
            },
            ArchProgramTemplate::LogisticsFeed => Self {
                template,
                name: template.as_str().to_string(),
                constants: [2, 1],
                source_text: String::new(),
                instructions: vec![
                    ArchInstruction::Mov {
                        dst: ArchRegister::CmdLogisticsEnable,
                        src: ArchValueRef::Immediate(0),
                    },
                    ArchInstruction::Mov {
                        dst: ArchRegister::CmdLogisticsPreference,
                        src: ArchValueRef::Immediate(0),
                    },
                    ArchInstruction::Gt {
                        dst: ArchRegister::Gp0,
                        lhs: ArchValueRef::Register(ArchRegister::StorageRawSalvage),
                        rhs: ArchValueRef::Immediate(0),
                    },
                    ArchInstruction::Lt {
                        dst: ArchRegister::Gp1,
                        lhs: ArchValueRef::Register(ArchRegister::ProcessorRawSalvage),
                        rhs: ArchValueRef::Immediate(2),
                    },
                    ArchInstruction::Add {
                        dst: ArchRegister::Gp2,
                        lhs: ArchValueRef::Register(ArchRegister::Gp0),
                        rhs: ArchValueRef::Register(ArchRegister::Gp1),
                    },
                    ArchInstruction::Gt {
                        dst: ArchRegister::Gp3,
                        lhs: ArchValueRef::Register(ArchRegister::Gp2),
                        rhs: ArchValueRef::Immediate(1),
                    },
                    ArchInstruction::Jnz {
                        cond: ArchValueRef::Register(ArchRegister::Gp3),
                        target: 9,
                    },
                    ArchInstruction::Jmp { target: 11 },
                    ArchInstruction::Mov {
                        dst: ArchRegister::CmdLogisticsEnable,
                        src: ArchValueRef::Immediate(1),
                    },
                    ArchInstruction::Mov {
                        dst: ArchRegister::CmdLogisticsPreference,
                        src: ArchValueRef::Immediate(0),
                    },
                ],
            },
            ArchProgramTemplate::TurretAssist => Self {
                template,
                name: template.as_str().to_string(),
                constants: [1, 2],
                source_text: String::new(),
                instructions: vec![
                    ArchInstruction::Mov {
                        dst: ArchRegister::CmdTurretAssist,
                        src: ArchValueRef::Immediate(0),
                    },
                    ArchInstruction::Mov {
                        dst: ArchRegister::CmdTurretAutoFire,
                        src: ArchValueRef::Immediate(0),
                    },
                    ArchInstruction::Gt {
                        dst: ArchRegister::Gp0,
                        lhs: ArchValueRef::Register(ArchRegister::MissionThreat),
                        rhs: ArchValueRef::Immediate(0),
                    },
                    ArchInstruction::Gt {
                        dst: ArchRegister::Gp1,
                        lhs: ArchValueRef::Register(ArchRegister::TurretReady),
                        rhs: ArchValueRef::Immediate(0),
                    },
                    ArchInstruction::Gt {
                        dst: ArchRegister::Gp2,
                        lhs: ArchValueRef::Register(ArchRegister::ShipPowerReserve),
                        rhs: ArchValueRef::Immediate(2),
                    },
                    ArchInstruction::Add {
                        dst: ArchRegister::Gp3,
                        lhs: ArchValueRef::Register(ArchRegister::Gp0),
                        rhs: ArchValueRef::Register(ArchRegister::Gp1),
                    },
                    ArchInstruction::Add {
                        dst: ArchRegister::Gp3,
                        lhs: ArchValueRef::Register(ArchRegister::Gp3),
                        rhs: ArchValueRef::Register(ArchRegister::Gp2),
                    },
                    ArchInstruction::Gt {
                        dst: ArchRegister::Gp3,
                        lhs: ArchValueRef::Register(ArchRegister::Gp3),
                        rhs: ArchValueRef::Immediate(2),
                    },
                    ArchInstruction::Jnz {
                        cond: ArchValueRef::Register(ArchRegister::Gp3),
                        target: 11,
                    },
                    ArchInstruction::Jmp { target: 13 },
                    ArchInstruction::Mov {
                        dst: ArchRegister::CmdTurretAssist,
                        src: ArchValueRef::Immediate(1),
                    },
                    ArchInstruction::Mov {
                        dst: ArchRegister::CmdTurretAutoFire,
                        src: ArchValueRef::Immediate(1),
                    },
                ],
            },
            ArchProgramTemplate::BalancedOps => Self {
                template,
                name: template.as_str().to_string(),
                constants: [8, 2],
                source_text: String::new(),
                instructions: vec![
                    ArchInstruction::Mov {
                        dst: ArchRegister::CmdReactorBias,
                        src: ArchValueRef::Immediate(0),
                    },
                    ArchInstruction::Mov {
                        dst: ArchRegister::CmdLogisticsEnable,
                        src: ArchValueRef::Immediate(0),
                    },
                    ArchInstruction::Mov {
                        dst: ArchRegister::CmdLogisticsPreference,
                        src: ArchValueRef::Immediate(0),
                    },
                    ArchInstruction::Mov {
                        dst: ArchRegister::CmdTurretAssist,
                        src: ArchValueRef::Immediate(0),
                    },
                    ArchInstruction::Mov {
                        dst: ArchRegister::CmdTurretAutoFire,
                        src: ArchValueRef::Immediate(0),
                    },
                    ArchInstruction::Gt {
                        dst: ArchRegister::Gp0,
                        lhs: ArchValueRef::Register(ArchRegister::ReactorHeat),
                        rhs: ArchValueRef::Immediate(8),
                    },
                    ArchInstruction::Jnz {
                        cond: ArchValueRef::Register(ArchRegister::Gp0),
                        target: 17,
                    },
                    ArchInstruction::Gt {
                        dst: ArchRegister::Gp1,
                        lhs: ArchValueRef::Register(ArchRegister::StorageRawSalvage),
                        rhs: ArchValueRef::Immediate(0),
                    },
                    ArchInstruction::Jnz {
                        cond: ArchValueRef::Register(ArchRegister::Gp1),
                        target: 19,
                    },
                    ArchInstruction::Gt {
                        dst: ArchRegister::Gp2,
                        lhs: ArchValueRef::Register(ArchRegister::MissionThreat),
                        rhs: ArchValueRef::Immediate(0),
                    },
                    ArchInstruction::Jnz {
                        cond: ArchValueRef::Register(ArchRegister::Gp2),
                        target: 22,
                    },
                    ArchInstruction::Jmp { target: 24 },
                    ArchInstruction::Mov {
                        dst: ArchRegister::CmdReactorBias,
                        src: ArchValueRef::Immediate(2),
                    },
                    ArchInstruction::Jmp { target: 24 },
                    ArchInstruction::Mov {
                        dst: ArchRegister::CmdLogisticsEnable,
                        src: ArchValueRef::Immediate(1),
                    },
                    ArchInstruction::Mov {
                        dst: ArchRegister::CmdLogisticsPreference,
                        src: ArchValueRef::Immediate(0),
                    },
                    ArchInstruction::Jmp { target: 24 },
                    ArchInstruction::Mov {
                        dst: ArchRegister::CmdTurretAssist,
                        src: ArchValueRef::Immediate(1),
                    },
                    ArchInstruction::Mov {
                        dst: ArchRegister::CmdTurretAutoFire,
                        src: ArchValueRef::Immediate(1),
                    },
                ],
            },
        };
        program.refresh_source_text();
        program
    }

    pub fn refresh_source_text(&mut self) {
        self.source_text = arch_program_to_source(&self.instructions);
    }

    pub fn compile_source_text(
        &mut self,
        source_text: &str,
    ) -> Result<(), Vec<ArchParseDiagnostic>> {
        let instructions = parse_arch_program(source_text)?;
        self.instructions = instructions;
        self.source_text = source_text.to_string();
        Ok(())
    }

    pub fn validate_source_text(
        source_text: &str,
    ) -> Result<Vec<ArchInstruction>, Vec<ArchParseDiagnostic>> {
        parse_arch_program(source_text)
    }
}

pub fn arch_program_to_source(instructions: &[ArchInstruction]) -> String {
    instructions
        .iter()
        .map(instruction_to_source)
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn parse_arch_program(
    source_text: &str,
) -> Result<Vec<ArchInstruction>, Vec<ArchParseDiagnostic>> {
    let mut instructions = Vec::new();
    let mut diagnostics = Vec::new();

    for (line_index, raw_line) in source_text.lines().enumerate() {
        let line_number = line_index + 1;
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }

        let tokens: Vec<_> = line.split_whitespace().collect();
        let opcode = tokens[0].to_ascii_uppercase();
        let parsed = match opcode.as_str() {
            "NOP" => parse_nop(tokens.len(), line_number),
            "MOV" => parse_binary_dst(tokens.as_slice(), line_number, |dst, src| {
                ArchInstruction::Mov { dst, src }
            }),
            "ADD" => parse_ternary_dst(tokens.as_slice(), line_number, |dst, lhs, rhs| {
                ArchInstruction::Add { dst, lhs, rhs }
            }),
            "SUB" => parse_ternary_dst(tokens.as_slice(), line_number, |dst, lhs, rhs| {
                ArchInstruction::Sub { dst, lhs, rhs }
            }),
            "GT" => parse_ternary_dst(tokens.as_slice(), line_number, |dst, lhs, rhs| {
                ArchInstruction::Gt { dst, lhs, rhs }
            }),
            "LT" => parse_ternary_dst(tokens.as_slice(), line_number, |dst, lhs, rhs| {
                ArchInstruction::Lt { dst, lhs, rhs }
            }),
            "JNZ" => parse_jump_conditional(tokens.as_slice(), line_number, |cond, target| {
                ArchInstruction::Jnz { cond, target }
            }),
            "JMP" => parse_jump(tokens.as_slice(), line_number, |target| {
                ArchInstruction::Jmp { target }
            }),
            other => Err(ArchParseDiagnostic {
                line: line_number,
                message: format!("unknown ARCH opcode '{other}'"),
            }),
        };

        match parsed {
            Ok(instruction) => instructions.push(instruction),
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }

    for (index, instruction) in instructions.iter().enumerate() {
        if let Some(target) = jump_target(instruction)
            && target >= instructions.len()
        {
            diagnostics.push(ArchParseDiagnostic {
                line: index + 1,
                message: format!("jump target {target} is past end of program"),
            });
        }
        if let Some(dst) = instruction_destination(instruction)
            && !dst.is_writable()
        {
            diagnostics.push(ArchParseDiagnostic {
                line: index + 1,
                message: format!("destination register '{}' is read-only", dst.as_str()),
            });
        }
    }

    if diagnostics.is_empty() {
        Ok(instructions)
    } else {
        Err(diagnostics)
    }
}

fn parse_nop(token_count: usize, line: usize) -> Result<ArchInstruction, ArchParseDiagnostic> {
    if token_count == 1 {
        Ok(ArchInstruction::Nop)
    } else {
        Err(ArchParseDiagnostic {
            line,
            message: "NOP takes no arguments".to_string(),
        })
    }
}

fn parse_binary_dst(
    tokens: &[&str],
    line: usize,
    build: impl FnOnce(ArchRegister, ArchValueRef) -> ArchInstruction,
) -> Result<ArchInstruction, ArchParseDiagnostic> {
    if tokens.len() != 3 {
        return Err(ArchParseDiagnostic {
            line,
            message: format!("{} expects 2 arguments", tokens[0]),
        });
    }
    let dst = parse_arch_register(tokens[1], line)?;
    let src = parse_arch_value(tokens[2], line)?;
    Ok(build(dst, src))
}

fn parse_ternary_dst(
    tokens: &[&str],
    line: usize,
    build: impl FnOnce(ArchRegister, ArchValueRef, ArchValueRef) -> ArchInstruction,
) -> Result<ArchInstruction, ArchParseDiagnostic> {
    if tokens.len() != 4 {
        return Err(ArchParseDiagnostic {
            line,
            message: format!("{} expects 3 arguments", tokens[0]),
        });
    }
    let dst = parse_arch_register(tokens[1], line)?;
    let lhs = parse_arch_value(tokens[2], line)?;
    let rhs = parse_arch_value(tokens[3], line)?;
    Ok(build(dst, lhs, rhs))
}

fn parse_jump_conditional(
    tokens: &[&str],
    line: usize,
    build: impl FnOnce(ArchValueRef, usize) -> ArchInstruction,
) -> Result<ArchInstruction, ArchParseDiagnostic> {
    if tokens.len() != 3 {
        return Err(ArchParseDiagnostic {
            line,
            message: format!("{} expects 2 arguments", tokens[0]),
        });
    }
    let cond = parse_arch_value(tokens[1], line)?;
    let target = parse_jump_target(tokens[2], line)?;
    Ok(build(cond, target))
}

fn parse_jump(
    tokens: &[&str],
    line: usize,
    build: impl FnOnce(usize) -> ArchInstruction,
) -> Result<ArchInstruction, ArchParseDiagnostic> {
    if tokens.len() != 2 {
        return Err(ArchParseDiagnostic {
            line,
            message: format!("{} expects 1 argument", tokens[0]),
        });
    }
    let target = parse_jump_target(tokens[1], line)?;
    Ok(build(target))
}

fn parse_arch_register(token: &str, line: usize) -> Result<ArchRegister, ArchParseDiagnostic> {
    let token = token.to_ascii_uppercase();
    match token.as_str() {
        "GP0" | "GP00" => Ok(ArchRegister::Gp0),
        "GP1" | "GP01" => Ok(ArchRegister::Gp1),
        "GP2" | "GP02" => Ok(ArchRegister::Gp2),
        "GP3" | "GP03" => Ok(ArchRegister::Gp3),
        "VPR" => Ok(ArchRegister::ShipPowerReserve),
        "VHT" => Ok(ArchRegister::ShipAverageHeat),
        "VTHR" => Ok(ArchRegister::MissionThreat),
        "RRF0" | "RRF1" | "RRF2" | "RRF3" | "RRF4" | "RRF5" | "RRF6" | "RRF7" | "RRF8" | "RRF9" => {
            Ok(ArchRegister::ReactorReactionRate)
        }
        "RRT0" | "RRT1" | "RRT2" | "RRT3" | "RRT4" | "RRT5" | "RRT6" | "RRT7" | "RRT8" | "RRT9" => {
            Ok(ArchRegister::ReactorTurbineLoad)
        }
        "RRH0" | "RRH1" | "RRH2" | "RRH3" | "RRH4" | "RRH5" | "RRH6" | "RRH7" | "RRH8" | "RRH9" => {
            Ok(ArchRegister::ReactorHeat)
        }
        "RRS0" | "RRS1" | "RRS2" | "RRS3" | "RRS4" | "RRS5" | "RRS6" | "RRS7" | "RRS8" | "RRS9" => {
            Ok(ArchRegister::ReactorInstability)
        }
        "RRP0" | "RRP1" | "RRP2" | "RRP3" | "RRP4" | "RRP5" | "RRP6" | "RRP7" | "RRP8" | "RRP9" => {
            Ok(ArchRegister::ReactorPowerOutput)
        }
        "LSR0" => Ok(ArchRegister::StorageRawSalvage),
        "LSR1" => Ok(ArchRegister::StorageRepairCharge),
        "LPR0" => Ok(ArchRegister::ProcessorRawSalvage),
        "LPR1" => Ok(ArchRegister::ProcessorRepairCharge),
        "WTR0" | "WTR1" | "WTR2" | "WTR3" | "WTR4" | "WTR5" | "WTR6" | "WTR7" | "WTR8" | "WTR9" => {
            Ok(ArchRegister::TurretReady)
        }
        "WTC0" | "WTC1" | "WTC2" | "WTC3" | "WTC4" | "WTC5" | "WTC6" | "WTC7" | "WTC8" | "WTC9" => {
            Ok(ArchRegister::TurretCooldown)
        }
        "DLF0" => Ok(ArchRegister::DetectLifeFriendly),
        "DLH0" => Ok(ArchRegister::DetectLifeHostile),
        "DLX0" => Ok(ArchRegister::DetectLifeDirX),
        "DLY0" => Ok(ArchRegister::DetectLifeDirY),
        "DLD0" => Ok(ArchRegister::DetectLifeDistance),
        "DSN0" => Ok(ArchRegister::DetectShipNearby),
        "DSH0" => Ok(ArchRegister::DetectShipHostile),
        "DSX0" => Ok(ArchRegister::DetectShipDirX),
        "DSY0" => Ok(ArchRegister::DetectShipDirY),
        "DSD0" => Ok(ArchRegister::DetectShipDistance),
        "DDM0" => Ok(ArchRegister::DetectDamageIncoming),
        "DDC0" => Ok(ArchRegister::DetectDamageCritical),
        "DDX0" => Ok(ArchRegister::DetectDamageDirX),
        "DDY0" => Ok(ArchRegister::DetectDamageDirY),
        "DDI0" => Ok(ArchRegister::DetectDamageIntensity),
        "DPP0" => Ok(ArchRegister::DetectPowerDeficit),
        "DPB0" => Ok(ArchRegister::DetectPowerLowBattery),
        "DHH0" => Ok(ArchRegister::DetectHeatAlert),
        "DHX0" => Ok(ArchRegister::DetectHeatDirX),
        "DHY0" => Ok(ArchRegister::DetectHeatDirY),
        "DHI0" => Ok(ArchRegister::DetectHeatSeverity),
        "DLG0" => Ok(ArchRegister::DetectLogisticsDemand),
        "DLX1" => Ok(ArchRegister::DetectLogisticsDirX),
        "DLY1" => Ok(ArchRegister::DetectLogisticsDirY),
        "DLI0" => Ok(ArchRegister::DetectLogisticsSeverity),
        "RRC0" | "RRC1" | "RRC2" | "RRC3" | "RRC4" | "RRC5" | "RRC6" | "RRC7" | "RRC8" | "RRC9" => {
            Ok(ArchRegister::CmdReactorBias)
        }
        "LMC0" => Ok(ArchRegister::CmdLogisticsEnable),
        "LMP0" => Ok(ArchRegister::CmdLogisticsPreference),
        "WTA0" | "WTA1" | "WTA2" | "WTA3" | "WTA4" | "WTA5" | "WTA6" | "WTA7" | "WTA8" | "WTA9" => {
            Ok(ArchRegister::CmdTurretAssist)
        }
        "WTF0" | "WTF1" | "WTF2" | "WTF3" | "WTF4" | "WTF5" | "WTF6" | "WTF7" | "WTF8" | "WTF9" => {
            Ok(ArchRegister::CmdTurretAutoFire)
        }
        _ => Err(ArchParseDiagnostic {
            line,
            message: format!("unknown ARCH register '{token}'"),
        }),
    }
}

fn parse_arch_value(token: &str, line: usize) -> Result<ArchValueRef, ArchParseDiagnostic> {
    if let Ok(register) = parse_arch_register(token, line) {
        return Ok(ArchValueRef::Register(register));
    }
    let token = token.trim_start_matches('#');
    token
        .parse::<i32>()
        .map(ArchValueRef::Immediate)
        .map_err(|_| ArchParseDiagnostic {
            line,
            message: format!("invalid ARCH operand '{token}'"),
        })
}

fn parse_jump_target(token: &str, line: usize) -> Result<usize, ArchParseDiagnostic> {
    let token = token.trim_start_matches(':').trim_start_matches('L');
    token.parse::<usize>().map_err(|_| ArchParseDiagnostic {
        line,
        message: format!("invalid jump target '{token}'"),
    })
}

fn jump_target(instruction: &ArchInstruction) -> Option<usize> {
    match instruction {
        ArchInstruction::Jmp { target } | ArchInstruction::Jnz { target, .. } => Some(*target),
        _ => None,
    }
}

fn instruction_destination(instruction: &ArchInstruction) -> Option<ArchRegister> {
    match instruction {
        ArchInstruction::Mov { dst, .. }
        | ArchInstruction::Add { dst, .. }
        | ArchInstruction::Sub { dst, .. }
        | ArchInstruction::Gt { dst, .. }
        | ArchInstruction::Lt { dst, .. } => Some(*dst),
        _ => None,
    }
}

fn instruction_to_source(instruction: &ArchInstruction) -> String {
    match instruction {
        ArchInstruction::Nop => "NOP".to_string(),
        ArchInstruction::Mov { dst, src } => {
            format!("MOV {} {}", dst.as_str(), value_to_source(src))
        }
        ArchInstruction::Add { dst, lhs, rhs } => {
            format!(
                "ADD {} {} {}",
                dst.as_str(),
                value_to_source(lhs),
                value_to_source(rhs)
            )
        }
        ArchInstruction::Sub { dst, lhs, rhs } => {
            format!(
                "SUB {} {} {}",
                dst.as_str(),
                value_to_source(lhs),
                value_to_source(rhs)
            )
        }
        ArchInstruction::Gt { dst, lhs, rhs } => {
            format!(
                "GT {} {} {}",
                dst.as_str(),
                value_to_source(lhs),
                value_to_source(rhs)
            )
        }
        ArchInstruction::Lt { dst, lhs, rhs } => {
            format!(
                "LT {} {} {}",
                dst.as_str(),
                value_to_source(lhs),
                value_to_source(rhs)
            )
        }
        ArchInstruction::Jnz { cond, target } => {
            format!("JNZ {} {}", value_to_source(cond), target)
        }
        ArchInstruction::Jmp { target } => format!("JMP {}", target),
        other => format!("# Unsupported structured op: {other:?}"),
    }
}

fn value_to_source(value: &ArchValueRef) -> String {
    match value {
        ArchValueRef::Register(register) => register.as_str().to_string(),
        ArchValueRef::Immediate(value) => value.to_string(),
    }
}
