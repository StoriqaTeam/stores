include!("tests_setup.rs");

use tokio_core::reactor::Core;

#[test]
fn test_get_store() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_store_service(Some(MOCK_USER_ID), handle);
    let work = service.get(1);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, 1);
}

#[test]
#[ignore]
fn test_create_allready_existed() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_store_service(Some(MOCK_USER_ID), handle);
    let new_store = create_new_store(serde_json::from_str(MOCK_STORE_NAME).unwrap());
    let work = service.create(new_store);
    let result = core.run(work);
    assert_eq!(result.is_err(), true);
}

#[test]
#[ignore]
fn test_list() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_store_service(Some(MOCK_USER_ID), handle);
    let work = service.list(1, 5);
    let result = core.run(work).unwrap();
    assert_eq!(result.len(), 5);
}

#[test]
#[ignore]
fn test_create_store() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_store_service(Some(MOCK_USER_ID), handle);
    let new_store = create_new_store(serde_json::from_str(MOCK_STORE_NAME).unwrap());
    let work = service.create(new_store);
    let result = core.run(work).unwrap();
    assert_eq!(result.name.to_string(), MOCK_STORE_NAME.to_string());
}

#[test]
#[ignore]
fn test_update() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_store_service(Some(MOCK_USER_ID), handle);
    let new_store = create_update_store(serde_json::from_str(MOCK_STORE_NAME).unwrap());
    let work = service.update(1, new_store);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, 1);
    assert_eq!(result.name.to_string(), MOCK_STORE_NAME.to_string());
}

#[test]
#[ignore]
fn test_deactivate() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_store_service(Some(MOCK_USER_ID), handle);
    let work = service.deactivate(1);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, 1);
    assert_eq!(result.is_active, false);
}
