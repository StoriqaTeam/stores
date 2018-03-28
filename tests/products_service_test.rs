include!("tests_setup.rs");

use tokio_core::reactor::Core;

#[test]
fn test_get_product() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_product_service(Some(MOCK_USER_ID), handle);
    let work = service.get(1);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, 1);
}

#[test]
fn test_list() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_product_service(Some(MOCK_USER_ID), handle);
    let work = service.list(1, 5);
    let result = core.run(work).unwrap();
    assert_eq!(result.len(), 5);
}

#[test]
fn test_create_product() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_product_service(Some(MOCK_USER_ID), handle);
    let new_product = create_new_product_with_attributes(MOCK_BASE_PRODUCT_ID);
    let work = service.create(new_product);
    let result = core.run(work).unwrap();
    assert_eq!(result.base_product_id, MOCK_BASE_PRODUCT_ID);
}

#[test]
fn test_update() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_product_service(Some(MOCK_USER_ID), handle);
    let new_product = create_update_product_with_attributes();
    let work = service.update(1, new_product);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, 1);
    assert_eq!(result.base_product_id, MOCK_BASE_PRODUCT_ID);
}

#[test]
fn test_deactivate() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_product_service(Some(MOCK_USER_ID), handle);
    let work = service.deactivate(1);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, 1);
    assert_eq!(result.is_active, false);
}
