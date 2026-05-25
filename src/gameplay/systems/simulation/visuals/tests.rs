use super::service_link_should_draw;
use crate::gameplay::components::{InfrastructureRouteKind, InfrastructureServiceStatus};

fn service(
    network_id: Option<u32>,
    service_coord: Option<(i32, i32)>,
    blocked_reason: Option<&str>,
) -> InfrastructureServiceStatus {
    InfrastructureServiceStatus {
        route_kind: InfrastructureRouteKind::Power,
        network_id,
        service_coord,
        required: true,
        blocked_reason: blocked_reason.map(str::to_string),
    }
}

#[test]
fn service_links_draw_only_for_connected_unblocked_live_services() {
    assert!(service_link_should_draw(
        &service(Some(1), Some((0, 0)), None),
        false
    ));
    assert!(!service_link_should_draw(
        &service(None, Some((0, 0)), None),
        false
    ));
    assert!(!service_link_should_draw(
        &service(Some(1), None, None),
        false
    ));
    assert!(!service_link_should_draw(
        &service(Some(1), Some((0, 0)), Some("closed valve")),
        false
    ));
    assert!(!service_link_should_draw(
        &service(Some(1), Some((0, 0)), None),
        true
    ));
}
