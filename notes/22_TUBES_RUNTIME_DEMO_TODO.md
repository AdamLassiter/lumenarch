# 22_TUBES_RUNTIME_DEMO_TODO

## Implemented

- [x] Rebuild deterministic runtime infrastructure networks from underlay route tiles.
- [x] Support power, oxygen duct, raw salvage, repair charge, fuel, ammunition, and oxygen resource route kinds.
- [x] Attach modules to compatible routes under or adjacent to their cell.
- [x] Split networks with closed junction boxes and valves.
- [x] Add ship-level infrastructure summaries and per-module blocked reasons.
- [x] Aggregate routed power upward for existing HUD compatibility.
- [x] Remove cockpit special interaction path; cockpit now follows nearby station interaction.
- [x] Require routed power for powered module runtime effects.
- [x] Route reactor fuel consumption through connected fuel pipes.
- [x] Route turret ammunition consumption through connected ammunition pipes.
- [x] Route processor raw input and typed output through connected pipes/storage.
- [x] Let oxygen ducts replenish tile atmosphere from connected oxygen supply.
- [x] Add station readouts for infrastructure status and blocker controls.
- [x] Add compact ARCH registers for junction/valve readback and open/close command.
- [x] Keep manual, manipulator, and drone logistics operating alongside routed pipe behavior.

## Verification

- [x] `cargo fmt`
- [x] `cargo test -q`

## Follow-Up Tests

- [ ] Graph ids and summaries are deterministic across repeated rebuilds.
- [ ] Same-type cardinal route tiles connect.
- [ ] Different route types do not connect.
- [ ] Closed valves and junction boxes split networks.
- [ ] Destroyed or missing route tiles break networks.
- [ ] Modules attach only to compatible under/adjacent routes.
- [ ] Wired reactor powers connected consumers.
- [ ] Disconnected consumers report `no wired power`.
- [ ] Closing a junction changes which consumers receive power.
- [ ] O2 generator/canister feeds ducts.
- [ ] Closed valves stop oxygen replenishment.
- [ ] Player breathing continues to read tile oxygen.
- [ ] Reactors consume fuel only from connected fuel storage.
- [ ] Ammunition turrets consume only connected ammunition.
- [ ] Processors consume/output through compatible connected routes.
- [ ] ARCH/manual valve and junction control changes runtime behavior.

## Follow-Up Implementation

- [ ] Add dedicated editor/runtime debug overlays for network ids, disconnected consumers, and closed blockers.
- [ ] Make battery/capacitor reserve behavior fully per-network and spendable.
- [ ] Route repair workflows through connected repair-charge storage.
- [ ] Extend ARCH infrastructure access beyond compact first-module/global command registers.
- [ ] Add scenario coverage for core-only enemies and routed combat-capable player ships.
