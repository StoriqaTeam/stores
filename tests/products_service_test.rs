include!("tests_setup.rs");

use tokio_core::reactor::Core;

#[test]
fn test_get_product() {
    let service = create_product_service(Some(MOCK_PRODUCT_NAME.to_string()));
    let mut core = Core::new().unwrap();
    let work = service.get(1);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, 1);
}

#[test]
fn test_create_allready_existed() {
    let service = create_product_service(Some(MOCK_PRODUCT_NAME.to_string()));
    let mut core = Core::new().unwrap();
    let new_product = create_new_product(MOCK_PRODUCT_NAME.to_string());
    let work = service.create(new_product);
    let result = core.run(work);
    assert_eq!(result.is_err(), true);
}

#[test]
fn test_list() {
    let service = create_product_service(Some(MOCK_PRODUCT_NAME.to_string()));
    let mut core = Core::new().unwrap();
    let work = service.list(1, 5);
    let result = core.run(work).unwrap();
    assert_eq!(result.len(), 5);
}

#[test]
fn test_create_product() {
    let service = create_product_service(Some(MOCK_PRODUCT_NAME.to_string()));
    let mut core = Core::new().unwrap();
    let new_product = create_new_product("new product".to_string());
    let work = service.create(new_product);
    let result = core.run(work).unwrap();
    assert_eq!(result.name, "new product".to_string());
}

#[test]
fn test_update() {
    let service = create_product_service(Some(MOCK_PRODUCT_NAME.to_string()));
    let mut core = Core::new().unwrap();
    let new_product = create_update_product(MOCK_PRODUCT_NAME.to_string());
    let work = service.update(1, new_product);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, 1);
    assert_eq!(result.name, MOCK_PRODUCT_NAME.to_string());
}

#[test]
fn test_deactivate() {
    let service = create_product_service(Some(MOCK_PRODUCT_NAME.to_string()));
    let mut core = Core::new().unwrap();
    let work = service.deactivate(1);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, 1);
    assert_eq!(result.is_active, false);
}
