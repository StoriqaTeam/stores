//! Moderator store comments repo, presents CRUD operations with db for moderator store comments
use std::convert::From;

use diesel;
use diesel::Connection;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::LoadQuery;
use diesel::query_dsl::RunQueryDsl;

use stq_acl::*;

use super::acl;
use super::error::RepoError as Error;
use super::types::RepoResult;
use models::authorization::*;
use models::moderator_store_comment::moderator_store_comments::dsl::*;
use models::{ModeratorStoreComments, NewModeratorStoreComments};

/// Moderator store comments repository
pub struct ModeratorStoreRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, Error, ModeratorStoreComments>>,
}

pub trait ModeratorStoreRepo {
    /// Find specific comments by store ID
    fn find_by_store_id(&self, store_id: i32) -> RepoResult<ModeratorStoreComments>;

    /// Creates new comment
    fn create(&self, payload: NewModeratorStoreComments) -> RepoResult<ModeratorStoreComments>;
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ModeratorStoreRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, Error, ModeratorStoreComments>>) -> Self {
        Self { db_conn, acl }
    }

    fn execute_query<Ty: Send + 'static, U: LoadQuery<T, Ty> + Send + 'static>(&self, query: U) -> RepoResult<Ty> {
        query.get_result::<Ty>(self.db_conn).map_err(Error::from)
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ModeratorStoreRepo
    for ModeratorStoreRepoImpl<'a, T>
{
    /// Find specific comments by store ID
    fn find_by_store_id(&self, store_id_arg: i32) -> RepoResult<ModeratorStoreComments> {
        debug!("Find moderator comments for store id {}.", store_id_arg);
        self.execute_query(
            moderator_store_comments
                .filter(store_id.eq(store_id_arg))
                .order_by(id.desc())
                .limit(1),
        ).and_then(|comment: ModeratorStoreComments| {
            acl::check(&*self.acl, &Resource::ModeratorStoreComments, &Action::Read, self, Some(&comment)).and_then(|_| Ok(comment))
        })
    }

    /// Creates new comment
    fn create(&self, payload: NewModeratorStoreComments) -> RepoResult<ModeratorStoreComments> {
        debug!("Create moderator comments for store {:?}.", payload);
        let query_store = diesel::insert_into(moderator_store_comments).values(&payload);
        query_store
            .get_result::<ModeratorStoreComments>(self.db_conn)
            .map_err(Error::from)
            .and_then(|comment| {
                acl::check(&*self.acl, &Resource::ModeratorStoreComments, &Action::Create, self, Some(&comment)).and_then(|_| Ok(comment))
            })
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, ModeratorStoreComments>
    for ModeratorStoreRepoImpl<'a, T>
{
    fn is_in_scope(&self, user_id_arg: i32, scope: &Scope, obj: Option<&ModeratorStoreComments>) -> bool {
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
