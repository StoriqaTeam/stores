include!("diesel_mock.rs");

extern crate stq_acl;

use stq_acl::SystemACL;

use stores_lib::repos::{StoresRepo, StoresRepoImpl};

#[test]
#[ignore]
fn test_find() {
    let conn = connection_with_stores_db_with_stores_table();
    let acl = Box::new(SystemACL {});
    let repo = StoresRepoImpl::new(&conn, acl);
    let _res = repo.find(1);
}
