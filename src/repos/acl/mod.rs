//! Repos is a module responsible for interacting with access control lists
//! Authorization module contains authorization logic for the repo layer app

#[macro_use]
pub mod macros;
pub mod legacy_acl;
pub mod roles_cache;

pub use self::roles_cache::RolesCacheImpl;

use std::collections::HashMap;
use std::rc::Rc;

use errors::Error;
use failure::Error as FailureError;

use stq_static_resources::ModerationStatus;
use stq_types::{StoresRole, UserId};

use self::legacy_acl::{Acl, CheckScope};

use models::authorization::*;

pub fn check<T>(
    acl: &Acl<Resource, Action, Scope, Rule, FailureError, T>,
    resource: Resource,
    action: Action,
    scope_checker: &CheckScope<Scope, T>,
    obj: Option<&T>,
) -> Result<(), FailureError> {
    acl.allows(resource, action, scope_checker, None, obj).and_then(|allowed| {
        if allowed {
            Ok(())
        } else {
            Err(format_err!("Denied request to do {:?} on {:?}", action, resource)
                .context(Error::Forbidden)
                .into())
        }
    })
}

pub fn check_with_rule<T>(
    acl: &Acl<Resource, Action, Scope, Rule, FailureError, T>,
    resource: Resource,
    action: Action,
    scope_checker: &CheckScope<Scope, T>,
    rule: Rule,
    obj: Option<&T>,
) -> Result<(), FailureError> {
    acl.allows(resource, action, scope_checker, Some(rule), obj).and_then(|allowed| {
        if allowed {
            Ok(())
        } else {
            Err(
                format_err!("Denied request to do {:?} on {:?} by rule: {:?}", action, resource, rule)
                    .context(Error::Forbidden)
                    .into(),
            )
        }
    })
}

/// ApplicationAcl contains main logic for manipulation with resources
#[derive(Clone)]
pub struct ApplicationAcl {
    acls: Rc<HashMap<StoresRole, Vec<Permission>>>,
    roles: Vec<StoresRole>,
    user_id: UserId,
}

impl ApplicationAcl {
    pub fn new(roles: Vec<StoresRole>, user_id: UserId) -> Self {
        let mut hash = ::std::collections::HashMap::new();
        hash.insert(
            StoresRole::Superuser,
            vec![
                permission!(Resource::Attributes),
                permission!(Resource::AttributeValues),
                permission!(Resource::BaseProducts),
                permission!(Resource::Categories),
                permission!(Resource::CategoryAttrs),
                permission!(Resource::CurrencyExchange),
                permission!(Resource::CustomAttributes),
                permission!(Resource::ModeratorProductComments),
                permission!(Resource::ModeratorStoreComments),
                permission!(Resource::ProductAttrs),
                permission!(Resource::Products),
                permission!(Resource::Stores),
                permission!(Resource::UserRoles),
                permission!(Resource::WizardStores),
                permission!(Resource::Coupons),
                permission!(Resource::CouponScopeBaseProducts),
                permission!(Resource::CouponScopeCategories),
                permission!(Resource::UsedCoupons),
            ],
        );
        hash.insert(
            StoresRole::User,
            vec![
                permission!(Resource::Attributes, Action::Read),
                permission!(Resource::AttributeValues, Action::Read),
                permission!(Resource::BaseProducts, Action::Create, Scope::Owned),
                permission!(Resource::BaseProducts, Action::Delete, Scope::Owned),
                permission!(
                    Resource::BaseProducts,
                    Action::Read,
                    Scope::All,
                    Rule::ModerationStatus(ModerationStatus::Published)
                ),
                permission!(Resource::BaseProducts, Action::Read, Scope::Owned, Rule::Any),
                permission!(
                    Resource::BaseProducts,
                    Action::Update,
                    Scope::Owned,
                    Rule::ModerationStatus(ModerationStatus::Draft)
                ),
                permission!(
                    Resource::BaseProducts,
                    Action::Update,
                    Scope::Owned,
                    Rule::ModerationStatus(ModerationStatus::Decline)
                ),
                permission!(
                    Resource::BaseProducts,
                    Action::Update,
                    Scope::Owned,
                    Rule::ModerationStatus(ModerationStatus::Published)
                ),
                permission!(Resource::Categories, Action::Read),
                permission!(Resource::CategoryAttrs, Action::Read),
                permission!(Resource::CurrencyExchange, Action::Read),
                permission!(Resource::CustomAttributes, Action::All, Scope::Owned),
                permission!(Resource::CustomAttributes, Action::Read),
                permission!(Resource::ModeratorProductComments, Action::All, Scope::Owned),
                permission!(Resource::ModeratorProductComments, Action::Read),
                permission!(Resource::ModeratorStoreComments, Action::All, Scope::Owned),
                permission!(Resource::ModeratorStoreComments, Action::Read),
                permission!(Resource::ProductAttrs, Action::All, Scope::Owned),
                permission!(Resource::ProductAttrs, Action::Read),
                permission!(Resource::Products, Action::All, Scope::Owned),
                permission!(Resource::Products, Action::Read),
                permission!(Resource::Stores, Action::Create, Scope::Owned),
                permission!(Resource::Stores, Action::Delete, Scope::Owned),
                permission!(
                    Resource::Stores,
                    Action::Read,
                    Scope::All,
                    Rule::ModerationStatus(ModerationStatus::Published)
                ),
                permission!(Resource::Stores, Action::Read, Scope::Owned, Rule::Any),
                permission!(
                    Resource::Stores,
                    Action::Update,
                    Scope::Owned,
                    Rule::ModerationStatus(ModerationStatus::Draft)
                ),
                permission!(
                    Resource::Stores,
                    Action::Update,
                    Scope::Owned,
                    Rule::ModerationStatus(ModerationStatus::Decline)
                ),
                permission!(
                    Resource::Stores,
                    Action::Update,
                    Scope::Owned,
                    Rule::ModerationStatus(ModerationStatus::Published)
                ),
                permission!(Resource::UserRoles, Action::Read, Scope::Owned),
                permission!(Resource::WizardStores, Action::All, Scope::Owned),
                permission!(Resource::WizardStores, Action::Read),
                permission!(Resource::Coupons, Action::All, Scope::Owned),
                permission!(Resource::Coupons, Action::Read),
                permission!(Resource::CouponScopeBaseProducts, Action::All, Scope::Owned),
                permission!(Resource::CouponScopeBaseProducts, Action::Read),
                permission!(Resource::CouponScopeCategories, Action::All, Scope::Owned),
                permission!(Resource::CouponScopeCategories, Action::Read),
                permission!(Resource::UsedCoupons, Action::Read),
            ],
        );

        hash.insert(
            StoresRole::Moderator,
            vec![
                permission!(Resource::BaseProducts, Action::Moderate),
                permission!(
                    Resource::BaseProducts,
                    Action::Read,
                    Scope::All,
                    Rule::ModerationStatus(ModerationStatus::Moderation)
                ),
                permission!(
                    Resource::BaseProducts,
                    Action::Read,
                    Scope::All,
                    Rule::ModerationStatus(ModerationStatus::Published)
                ),
                permission!(
                    Resource::BaseProducts,
                    Action::Read,
                    Scope::All,
                    Rule::ModerationStatus(ModerationStatus::Decline)
                ),
                permission!(
                    Resource::BaseProducts,
                    Action::Read,
                    Scope::All,
                    Rule::ModerationStatus(ModerationStatus::Blocked)
                ),
                permission!(Resource::ModeratorProductComments),
                permission!(Resource::ModeratorStoreComments),
                permission!(Resource::Stores, Action::Moderate),
                permission!(
                    Resource::Stores,
                    Action::Read,
                    Scope::All,
                    Rule::ModerationStatus(ModerationStatus::Moderation)
                ),
                permission!(
                    Resource::Stores,
                    Action::Read,
                    Scope::All,
                    Rule::ModerationStatus(ModerationStatus::Published)
                ),
                permission!(
                    Resource::Stores,
                    Action::Read,
                    Scope::All,
                    Rule::ModerationStatus(ModerationStatus::Decline)
                ),
                permission!(
                    Resource::Stores,
                    Action::Read,
                    Scope::All,
                    Rule::ModerationStatus(ModerationStatus::Blocked)
                ),
            ],
        );

        hash.insert(
            StoresRole::PlatformAdmin,
            vec![
                permission!(Resource::Attributes),
                permission!(Resource::Categories),
                permission!(Resource::CategoryAttrs),
            ],
        );

        ApplicationAcl {
            acls: Rc::new(hash),
            roles,
            user_id,
        }
    }
}

impl<T> Acl<Resource, Action, Scope, Rule, FailureError, T> for ApplicationAcl {
    fn allows(
        &self,
        resource: Resource,
        action: Action,
        scope_checker: &CheckScope<Scope, T>,
        rule: Option<Rule>,
        obj: Option<&T>,
    ) -> Result<bool, FailureError> {
        let empty: Vec<Permission> = Vec::new();
        let user_id = &self.user_id;
        let hashed_acls = self.acls.clone();
        let acls = self
            .roles
            .iter()
            .flat_map(|role| hashed_acls.get(role).unwrap_or(&empty))
            .filter(|permission| {
                let check_result =
                    (permission.resource == resource) && ((permission.action == action) || (permission.action == Action::All));

                let check_rule = match (rule, permission.rule) {
                    (Some(rule), Some(permission_rule)) => ((permission_rule == rule) || (permission_rule == Rule::Any)),
                    _ => true,
                };

                check_result && check_rule
            }).filter(|permission| scope_checker.is_in_scope(*user_id, &permission.scope, obj));

        if acls.count() > 0 {
            Ok(true)
        } else {
            error!(
                "Denied request from user {} to do {} on {} by rule: {:?}.",
                user_id, action, resource, rule
            );
            Ok(false)
        }
    }
}

/// UnauthorizedAcl contains main logic for manipulation with resources
#[derive(Clone, Default)]
pub struct UnauthorizedAcl;

impl<T> Acl<Resource, Action, Scope, Rule, FailureError, T> for UnauthorizedAcl {
    fn allows(
        &self,
        resource: Resource,
        action: Action,
        _scope_checker: &CheckScope<Scope, T>,
        rule: Option<Rule>,
        _obj: Option<&T>,
    ) -> Result<bool, FailureError> {
        if action == Action::Read {
            match resource {
                Resource::Categories
                | Resource::Products
                | Resource::ProductAttrs
                | Resource::Attributes
                | Resource::AttributeValues
                | Resource::CurrencyExchange
                | Resource::WizardStores
                | Resource::ModeratorProductComments
                | Resource::ModeratorStoreComments
                | Resource::CategoryAttrs => Ok(true),

                Resource::Stores | Resource::BaseProducts => match rule {
                    Some(value) => match value {
                        Rule::Any => Ok(true),
                        Rule::ModerationStatus(status) => Ok(status == ModerationStatus::Published),
                    },
                    _ => Ok(true),
                },
                _ => Ok(false),
            }
        } else {
            error!("Denied unauthorized request to do {} on {} by rule: {:?}.", action, resource, rule);
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use repos::legacy_acl::{Acl, CheckScope};
    use serde_json;

    use stq_static_resources::*;
    use stq_types::*;

    use models::*;
    use repos::*;

    fn create_store(user_id: UserId) -> Store {
        Store {
            id: StoreId(1),
            user_id,
            name: serde_json::from_str("{}").unwrap(),
            is_active: true,
            short_description: serde_json::from_str("{}").unwrap(),
            long_description: None,
            slug: "myname".to_string(),
            cover: None,
            logo: None,
            phone: Some("1234567".to_string()),
            email: Some("example@mail.com".to_string()),
            address: Some("town city street".to_string()),
            default_language: "en".to_string(),
            slogan: Some("fdsf".to_string()),
            facebook_url: None,
            twitter_url: None,
            instagram_url: None,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            country: None,
            country_code: None,
            rating: 0f64,
            product_categories: Some(serde_json::from_str("{}").unwrap()),
            status: ModerationStatus::Published,
            administrative_area_level_1: None,
            administrative_area_level_2: None,
            locality: None,
            political: None,
            postal_code: None,
            route: None,
            street_number: None,
            place_id: None,
            kafka_update_no: 0,
        }
    }

    #[derive(Default)]
    struct ScopeChecker;

    impl CheckScope<Scope, Store> for ScopeChecker {
        fn is_in_scope(&self, user_id: UserId, scope: &Scope, obj: Option<&Store>) -> bool {
            match *scope {
                Scope::All => true,
                Scope::Owned => {
                    if let Some(store) = obj {
                        store.user_id == user_id
                    } else {
                        false
                    }
                }
            }
        }
    }

    impl CheckScope<Scope, UserRole> for ScopeChecker {
        fn is_in_scope(&self, user_id: UserId, scope: &Scope, obj: Option<&UserRole>) -> bool {
            match *scope {
                Scope::All => true,
                Scope::Owned => {
                    if let Some(user_role) = obj {
                        user_role.user_id == user_id
                    } else {
                        false
                    }
                }
            }
        }
    }

    #[test]
    fn test_super_user_for_stores() {
        let acl = ApplicationAcl::new(vec![StoresRole::Superuser], UserId(1232));
        let s = ScopeChecker::default();
        let resource = create_store(UserId(1));

        assert_eq!(
            acl.allows(Resource::Stores, Action::All, &s, Some(Rule::Any), Some(&resource))
                .unwrap(),
            true,
            "ACL does not allow all actions on store for superuser."
        );
        assert_eq!(
            acl.allows(Resource::Stores, Action::Read, &s, Some(Rule::Any), Some(&resource))
                .unwrap(),
            true,
            "ACL does not allow read action on store for superuser."
        );
        assert_eq!(
            acl.allows(Resource::Stores, Action::Create, &s, Some(Rule::Any), Some(&resource))
                .unwrap(),
            true,
            "ACL does not allow  create actions on store for superuser."
        );
        assert_eq!(
            acl.allows(Resource::Stores, Action::Update, &s, Some(Rule::Any), Some(&resource))
                .unwrap(),
            true,
            "ACL does not allow update actions on store for superuser."
        );
        assert_eq!(
            acl.allows(Resource::Stores, Action::Delete, &s, Some(Rule::Any), Some(&resource))
                .unwrap(),
            true,
            "ACL does not allow  delete actions on store for superuser."
        );
        assert_eq!(
            acl.allows(Resource::Stores, Action::Moderate, &s, Some(Rule::Any), Some(&resource))
                .unwrap(),
            true,
            "ACL does not allow moderate actions on store for superuser."
        );
    }

    #[test]
    fn test_ordinary_user_for_stores() {
        let user_id = UserId(2);
        let acl = ApplicationAcl::new(vec![StoresRole::User], user_id);
        let s = ScopeChecker::default();
        let resource = create_store(user_id);

        assert_eq!(
            acl.allows(Resource::Stores, Action::All, &s, Some(Rule::Any), Some(&resource))
                .unwrap(),
            false,
            "ACL allows all actions on store for ordinary_user."
        );
        assert_eq!(
            acl.allows(Resource::Stores, Action::Read, &s, Some(Rule::Any), Some(&resource))
                .unwrap(),
            true,
            "ACL does not allow read action on store for ordinary_user."
        );
        assert_eq!(
            acl.allows(Resource::Stores, Action::Create, &s, Some(Rule::Any), Some(&resource))
                .unwrap(),
            true,
            "ACL does not allow create actions on store for ordinary_user."
        );
        assert_eq!(
            acl.allows(
                Resource::Stores,
                Action::Update,
                &s,
                Some(Rule::ModerationStatus(ModerationStatus::Draft)),
                Some(&resource)
            ).unwrap(),
            true,
            "ACL does not allow update actions on store for ordinary_user."
        );
        assert_eq!(
            acl.allows(Resource::Stores, Action::Delete, &s, Some(Rule::Any), Some(&resource))
                .unwrap(),
            true,
            "ACL does not allow delete actions on store for ordinary_user."
        );
        assert_eq!(
            acl.allows(Resource::Stores, Action::Moderate, &s, Some(Rule::Any), Some(&resource))
                .unwrap(),
            false,
            "ACL allows moderate actions on store for ordinary_user."
        );
    }

    #[test]
    fn test_moderator_for_stores() {
        let acl = ApplicationAcl::new(vec![StoresRole::Moderator], UserId(32));
        let s = ScopeChecker::default();
        let resource = create_store(UserId(1));

        assert_eq!(
            acl.allows(Resource::Stores, Action::All, &s, Some(Rule::Any), Some(&resource))
                .unwrap(),
            false,
            "ACL allows all actions on store for moderator."
        );
        assert_eq!(
            acl.allows(
                Resource::Stores,
                Action::Read,
                &s,
                Some(Rule::ModerationStatus(ModerationStatus::Moderation)),
                Some(&resource)
            ).unwrap(),
            true,
            "ACL does not allow read action on store for moderator."
        );
        assert_eq!(
            acl.allows(Resource::Stores, Action::Create, &s, Some(Rule::Any), Some(&resource))
                .unwrap(),
            false,
            "ACL allows create actions on store for moderator."
        );
        assert_eq!(
            acl.allows(Resource::Stores, Action::Update, &s, Some(Rule::Any), Some(&resource))
                .unwrap(),
            false,
            "ACL allows update actions on store for moderator."
        );
        assert_eq!(
            acl.allows(Resource::Stores, Action::Delete, &s, Some(Rule::Any), Some(&resource))
                .unwrap(),
            false,
            "ACL allows delete actions on store for moderator."
        );
        assert_eq!(
            acl.allows(Resource::Stores, Action::Moderate, &s, Some(Rule::Any), Some(&resource))
                .unwrap(),
            true,
            "ACL does not allow moderate actions on store for moderator."
        );
    }

    #[test]
    fn test_super_user_for_user_roles() {
        let acl = ApplicationAcl::new(vec![StoresRole::Superuser], UserId(1232));
        let s = ScopeChecker::default();

        let resource = UserRole {
            id: RoleId::new(),
            user_id: UserId(1),
            name: StoresRole::User,
            data: None,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        };

        assert_eq!(
            acl.allows(Resource::UserRoles, Action::All, &s, Some(Rule::Any), Some(&resource))
                .unwrap(),
            true
        );
        assert_eq!(
            acl.allows(Resource::UserRoles, Action::Read, &s, Some(Rule::Any), Some(&resource))
                .unwrap(),
            true
        );
        assert_eq!(
            acl.allows(Resource::UserRoles, Action::Create, &s, Some(Rule::Any), Some(&resource))
                .unwrap(),
            true
        );
    }

    #[test]
    fn test_user_for_user_roles() {
        let acl = ApplicationAcl::new(vec![StoresRole::User], UserId(2));
        let s = ScopeChecker::default();

        let resource = UserRole {
            id: RoleId::new(),
            user_id: UserId(1),
            name: StoresRole::User,
            data: None,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        };

        assert_eq!(
            acl.allows(Resource::UserRoles, Action::All, &s, Some(Rule::Any), Some(&resource))
                .unwrap(),
            false
        );
        assert_eq!(
            acl.allows(Resource::UserRoles, Action::Read, &s, Some(Rule::Any), Some(&resource))
                .unwrap(),
            false
        );
        assert_eq!(
            acl.allows(Resource::UserRoles, Action::Create, &s, Some(Rule::Any), Some(&resource))
                .unwrap(),
            false
        );
    }

}
