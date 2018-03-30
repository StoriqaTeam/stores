include!("tests_setup.rs");

use tokio_core::reactor::Core;

#[test]
fn test_get_attribute() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_attribute_service(Some(MOCK_USER_ID), handle);
    let work = service.get(1);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, 1);
}

#[test]
fn test_create_attribute() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_attribute_service(Some(MOCK_USER_ID), handle);
    let new_attribute = create_new_attribute(MOCK_BASE_PRODUCT_NAME_JSON);
    let work = service.create(new_attribute);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, MOCK_BASE_PRODUCT_ID);
}

#[test]
fn test_update() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_attribute_service(Some(MOCK_USER_ID), handle);
    let new_attribute = create_update_attribute(MOCK_BASE_PRODUCT_NAME_JSON);
    let work = service.update(1, new_attribute);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, 1);
}
