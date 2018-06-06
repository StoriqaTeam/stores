//! Moderator product comments repo, presents CRUD operations with db for moderator product comments
use diesel;
use diesel::Connection;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use failure::Error as FailureError;

use stq_acl::*;

use super::acl;
use super::types::RepoResult;
use models::authorization::*;
use models::moderator_product_comment::moderator_product_comments::dsl::*;
use models::{ModeratorProductComments, NewModeratorProductComments};

/// Moderator product comments repository
pub struct ModeratorProductRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, ModeratorProductComments>>,
}

pub trait ModeratorProductRepo {
    /// Find comments by base_product ID
    fn find_by_base_product_id(&self, base_product_id: i32) -> RepoResult<Option<ModeratorProductComments>>;

    /// Creates new comment
    fn create(&self, payload: NewModeratorProductComments) -> RepoResult<ModeratorProductComments>;
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ModeratorProductRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, ModeratorProductComments>>) -> Self {
        Self { db_conn, acl }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ModeratorProductRepo
    for ModeratorProductRepoImpl<'a, T>
{
    /// Find comments by base_product ID
    fn find_by_base_product_id(&self, base_product_id_arg: i32) -> RepoResult<Option<ModeratorProductComments>> {
        debug!("Find moderator comments for base product id {}.", base_product_id_arg);
        let query = moderator_product_comments
            .filter(base_product_id.eq(base_product_id_arg))
            .order_by(id.desc())
            .limit(1);
        query
            .get_result(self.db_conn)
            .optional()
            .map_err(|e| e.into())
            .and_then(|comment: Option<ModeratorProductComments>| {
                if let Some(ref comment) = comment {
                    acl::check(&*self.acl, &Resource::ModeratorProductComments, &Action::Read, self, Some(comment))?;
                };
                Ok(comment)
            })
            .map_err(|e: FailureError| {
                e.context(format!("Find moderator comments for base product id {}", base_product_id_arg))
                    .into()
            })
    }

    /// Creates new comment
    fn create(&self, payload: NewModeratorProductComments) -> RepoResult<ModeratorProductComments> {
        debug!("Create moderator comments for base product {:?}.", payload);
        let query_store = diesel::insert_into(moderator_product_comments).values(&payload);
        query_store
            .get_result::<ModeratorProductComments>(self.db_conn)
            .map_err(|e| e.into())
            .and_then(|comment| {
                acl::check(&*self.acl, &Resource::ModeratorProductComments, &Action::Create, self, None)?;
                Ok(comment)
            })
            .map_err(|e: FailureError| {
                e.context(format!("Create moderator comments for base product {:?}.", payload))
                    .into()
            })
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, ModeratorProductComments>
    for ModeratorProductRepoImpl<'a, T>
{
    fn is_in_scope(&self, user_id_arg: i32, scope: &Scope, obj: Option<&ModeratorProductComments>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => {
                if let Some(comment) = obj {
                    comment.moderator_id == user_id_arg
                } else {
                    false
                }
            }
        }
    }
}
