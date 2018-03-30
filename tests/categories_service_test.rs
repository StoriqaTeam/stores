include!("tests_setup.rs");

use tokio_core::reactor::Core;

#[test]
fn test_get_categories() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_categories_service(Some(MOCK_USER_ID), handle);
    let work = service.get(1);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, 1);
}

#[test]
fn test_create_categories() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_categories_service(Some(MOCK_USER_ID), handle);
    let new_categories = create_new_categories(MOCK_BASE_PRODUCT_NAME_JSON);
    let work = service.create(new_categories);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, MOCK_BASE_PRODUCT_ID);
}

#[test]
fn test_update() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_categories_service(Some(MOCK_USER_ID), handle);
    let new_categories = create_update_categories(MOCK_BASE_PRODUCT_NAME_JSON);
    let work = service.update(1, new_categories);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, 1);
}
