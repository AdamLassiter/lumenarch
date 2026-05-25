use std::collections::{BTreeMap, BTreeSet, VecDeque};

use bevy::{ecs::relationship::Relationship, prelude::*};

use crate::{
    gameplay::{
        components::{
            DestroyedModule,
            InfrastructureNetworkSummary,
            InfrastructureRouteKind,
            InfrastructureServiceStatus,
            Integrity,
            JunctionCommandState,
            ModuleInfrastructureStatus,
            PowerConsumer,
            PowerProducer,
            ProcessorModule,
            ReactorCommandState,
            ResourceKind,
            RuntimeFoundationVisual,
            RuntimeShipModule,
            ShipInfrastructureState,
            ShipPowerModel,
            ShipPowerState,
            ShipRoot,
            StorageModule,
            ValveCommandState,
            WeaponModule,
        },
        helpers::{Fx, cardinal_neighbors, component_service_coords, fx_from_time_delta},
    },
    ship::{ModuleKind, ShipFoundationKind},
};

mod networks;
mod systems;
#[cfg(test)]
mod tests;
mod types;

use networks::*;
pub(crate) use systems::{rebuild_infrastructure_networks, update_routed_ship_power};
use types::*;
