use net_ninja::models::types::ProcessId;
use uuid::Uuid;

#[test]
fn test_process_id_new() {
    let id1 = ProcessId::new();
    let id2 = ProcessId::new();

    assert_ne!(id1.inner(), id2.inner(), "UUIDs should be unique");
}

#[test]
fn test_process_id_from_uuid() {
    let uuid = Uuid::new_v4();
    let process_id = ProcessId::from(uuid);

    assert_eq!(process_id.inner(), uuid);
}

#[test]
fn test_process_id_equality() {
    let uuid = Uuid::new_v4();
    let id1 = ProcessId::from(uuid);
    let id2 = ProcessId::from(uuid);

    assert_eq!(id1, id2);
}
