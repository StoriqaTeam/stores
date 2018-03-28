include!("tests_setup.rs");

use tokio_core::reactor::Core;

#[test]
fn test_get_user_roles() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_user_roles_service(Some(MOCK_USER_ID), handle);
    let work = service.get_roles(1);
    let result = core.run(work).unwrap();
    assert_eq!(result[0], Role::Superuser);
}

#[test]
fn test_create_user_roles() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_user_roles_service(Some(MOCK_USER_ID), handle);
    let new_user_roles = create_new_user_roles(MOCK_USER_ID);
    let work = service.create(new_user_roles);
    let result = core.run(work).unwrap();
    assert_eq!(result.user_id, MOCK_BASE_PRODUCT_ID);
}
