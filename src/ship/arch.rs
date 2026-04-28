use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArchRegister {
    Gp0,
    Gp1,
    Gp2,
    Gp3,
    ShipPowerReserve,
    ShipAverageHeat,
    MissionThreat,
    ReactorHeat,
    ReactorInstability,
    StorageRawSalvage,
    StorageRepairCharge,
    ProcessorRawSalvage,
    ProcessorRepairCharge,
    TurretReady,
    TurretCooldown,
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
            Self::ReactorHeat => "RRH0",
            Self::ReactorInstability => "RRS0",
            Self::StorageRawSalvage => "LSR0",
            Self::StorageRepairCharge => "LSR1",
            Self::ProcessorRawSalvage => "LPR0",
            Self::ProcessorRepairCharge => "LPR1",
            Self::TurretReady => "WTR0",
            Self::TurretCooldown => "WTC0",
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
    pub instructions: Vec<ArchInstruction>,
}

impl ArchProgram {
    pub fn from_template(template: ArchProgramTemplate) -> Self {
        match template {
            ArchProgramTemplate::ReactorGuard => Self {
                template,
                name: template.as_str().to_string(),
                constants: [9, 7],
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
        }
    }
}
