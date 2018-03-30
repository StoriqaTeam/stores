include!("tests_setup.rs");

use tokio_core::reactor::Core;

#[test]
fn test_get_base_product() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_base_product_service(Some(MOCK_USER_ID), handle);
    let work = service.get(1);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, 1);
}

#[test]
fn test_list() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_base_product_service(Some(MOCK_USER_ID), handle);
    let work = service.list(1, 5);
    let result = core.run(work).unwrap();
    assert_eq!(result.len(), 5);
}

#[test]
fn test_create_base_product() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_base_product_service(Some(MOCK_USER_ID), handle);
    let new_base_product = create_new_base_product(MOCK_BASE_PRODUCT_NAME_JSON);
    let work = service.create(new_base_product);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, MOCK_BASE_PRODUCT_ID);
}

#[test]
fn test_update() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_base_product_service(Some(MOCK_USER_ID), handle);
    let new_base_product = create_update_base_product(MOCK_BASE_PRODUCT_NAME_JSON);
    let work = service.update(1, new_base_product);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, 1);
    assert_eq!(result.id, MOCK_BASE_PRODUCT_ID);
}

#[test]
fn test_deactivate() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_base_product_service(Some(MOCK_USER_ID), handle);
    let work = service.deactivate(1);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, 1);
    assert_eq!(result.is_active, false);
}
