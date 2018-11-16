//! Moderator product comments repo, presents CRUD operations with db for moderator product comments
use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;
use failure::Error as FailureError;

use stq_types::{StoreId, UserId};

use models::authorization::*;
use models::{ModeratorStoreComments, NewModeratorStoreComments};
use repos::acl;
use repos::legacy_acl::*;
use repos::types::{RepoAcl, RepoResult};
use schema::moderator_store_comments::dsl::*;

/// Moderator product comments repository
pub struct ModeratorStoreRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<RepoAcl<ModeratorStoreComments>>,
}

pub trait ModeratorStoreRepo {
    /// Find comments by store ID
    fn find_by_store_id(&self, store_id: StoreId) -> RepoResult<Option<ModeratorStoreComments>>;

    /// Creates new comment
    fn create(&self, payload: NewModeratorStoreComments) -> RepoResult<ModeratorStoreComments>;
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ModeratorStoreRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<RepoAcl<ModeratorStoreComments>>) -> Self {
        Self { db_conn, acl }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ModeratorStoreRepo
    for ModeratorStoreRepoImpl<'a, T>
{
    /// Find comments by store ID
    fn find_by_store_id(&self, store_id_arg: StoreId) -> RepoResult<Option<ModeratorStoreComments>> {
        debug!("Find moderator comments for store id {}.", store_id_arg);
        let query = moderator_store_comments
            .filter(store_id.eq(store_id_arg))
            .order_by(id.desc())
            .limit(1);
        query
            .get_result(self.db_conn)
            .optional()
            .map_err(From::from)
            .and_then(|comment: Option<ModeratorStoreComments>| {
                if let Some(ref comment) = comment {
                    acl::check(&*self.acl, Resource::ModeratorStoreComments, Action::Read, self, Some(comment))?;
                };
                Ok(comment)
            }).map_err(|e: FailureError| e.context(format!("Find moderator comments for store id {}", store_id_arg)).into())
    }

    /// Creates new comment
    fn create(&self, payload: NewModeratorStoreComments) -> RepoResult<ModeratorStoreComments> {
        debug!("Create moderator comments for store {:?}.", payload);
        let query_store = diesel::insert_into(moderator_store_comments).values(&payload);
        query_store
            .get_result::<ModeratorStoreComments>(self.db_conn)
            .map_err(From::from)
            .and_then(|comment| {
                acl::check(&*self.acl, Resource::ModeratorStoreComments, Action::Create, self, None)?;
                Ok(comment)
            }).map_err(|e: FailureError| e.context(format!("Create moderator comments for store {:?}.", payload)).into())
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, ModeratorStoreComments>
    for ModeratorStoreRepoImpl<'a, T>
{
    fn is_in_scope(&self, user_id_arg: UserId, scope: &Scope, obj: Option<&ModeratorStoreComments>) -> bool {
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
