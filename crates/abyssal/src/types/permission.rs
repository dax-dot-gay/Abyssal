use std::sync::Arc;

use parking_lot::RwLock;
use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionCapability {
    /// View existing <resource>
    Read,

    /// Edit existing <resource>
    Edit,

    /// Create/destroy existing <resource>
    Manage,
}

impl PermissionCapability {
    pub fn has_at_least(&self, capability: Self) -> bool {
        match capability.clone() {
            PermissionCapability::Read => true,
            PermissionCapability::Edit => self.clone() != Self::Read,
            PermissionCapability::Manage => self.clone() == Self::Manage,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum RootTopLevel {
    Root,
    Home { parent: String },
    Directory { path: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionKind {
    Administrator,
    Invites,
    UploadTargets,
    RootDirectory,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "permission")]
pub enum Permission {
    /// Supersedes all other permissions
    /// Also is *required* to allow access to:
    /// - OIDC management
    /// - User/group management
    /// - Permission assignment
    Administrator,

    /// Invite management
    Invites {
        /// Root allowed to create invites to
        root: Uuid,

        /// What level of access to grant
        capability: PermissionCapability,

        /// Whether to allow access to invites owned by other users
        administrate: bool,
    },

    /// Upload dropbox management
    UploadTargets {
        /// Roots allowed to create upload targets for
        root: Uuid,

        /// What level of access to grant
        capability: PermissionCapability,

        /// Whether to allow access to upload targets owned by other users
        administrate: bool,
    },

    /// Access to a root directory
    RootDirectory {
        /// Root ID
        root: Uuid,

        /// Top-level directory
        top_level: RootTopLevel,

        /// What level of access to grant
        capability: PermissionCapability,
    },
}

impl Permission {
    pub fn kind(&self) -> PermissionKind {
        match self.clone() {
            Permission::Administrator => PermissionKind::Administrator,
            Permission::Invites { .. } => PermissionKind::Invites,
            Permission::UploadTargets { .. } => PermissionKind::UploadTargets,
            Permission::RootDirectory { .. } => PermissionKind::RootDirectory,
        }
    }

    pub fn capability(&self) -> PermissionCapability {
        match self.clone() {
            Permission::Administrator => PermissionCapability::Manage,
            Permission::Invites { capability, .. } => capability,
            Permission::UploadTargets { capability, .. } => capability,
            Permission::RootDirectory { capability, .. } => capability,
        }
    }

    pub fn root(&self) -> Option<Uuid> {
        match self.clone() {
            Permission::Administrator => None,
            Permission::Invites { root, .. }
            | Permission::UploadTargets { root, .. }
            | Permission::RootDirectory { root, .. } => Some(root),
        }
    }

    pub fn administrate(&self) -> bool {
        match self.clone() {
            Permission::Administrator => true,
            Permission::Invites { administrate, .. }
            | Permission::UploadTargets { administrate, .. } => administrate,
            Permission::RootDirectory { .. } => false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(from = "Vec<Permission>", into = "Vec<Permission>")]
pub struct PermissionSet(Arc<RwLock<Vec<Permission>>>);

impl Default for PermissionSet {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonSchema for PermissionSet {
    fn schema_name() -> String {
        Vec::<Permission>::schema_name()
    }

    fn json_schema(generator: &mut schemars::r#gen::SchemaGenerator) -> schemars::schema::Schema {
        Vec::<Permission>::json_schema(generator)
    }
}

impl From<Vec<Permission>> for PermissionSet {
    fn from(value: Vec<Permission>) -> Self {
        Self(Arc::new(RwLock::new(value)))
    }
}

impl Into<Vec<Permission>> for PermissionSet {
    fn into(self) -> Vec<Permission> {
        self.0.write().clone()
    }
}

impl PermissionSet {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(Vec::new())))
    }

    pub fn set_permission(&self, permission: Permission) -> () {
        let mut set = self.0.write_arc();
        match permission.clone() {
            Permission::Administrator => {
                set.clear();
                set.push(Permission::Administrator);
            }
            Permission::Invites { root, .. } => {
                *set = set
                    .clone()
                    .into_iter()
                    .filter(|perm| match perm.clone() {
                        Permission::Invites {
                            root: existing_root,
                            ..
                        } => root != existing_root,
                        _ => true,
                    })
                    .collect();
                set.push(permission);
            }
            Permission::UploadTargets { root, .. } => {
                *set = set
                    .clone()
                    .into_iter()
                    .filter(|perm| match perm.clone() {
                        Permission::UploadTargets {
                            root: existing_root,
                            ..
                        } => root != existing_root,
                        _ => true,
                    })
                    .collect();
                set.push(permission);
            }
            Permission::RootDirectory { root, .. } => {
                *set = set
                    .clone()
                    .into_iter()
                    .filter(|perm| match perm.clone() {
                        Permission::RootDirectory {
                            root: existing_root,
                            ..
                        } => root != existing_root,
                        _ => true,
                    })
                    .collect();
                set.push(permission);
            }
        }
    }

    pub fn remove_permission(&self, permission: Permission) -> () {
        let mut set = self.0.write_arc();
        *set = set
            .clone()
            .into_iter()
            .filter(|v| v.clone() != permission)
            .collect();
    }

    pub fn is_administrator(&self) -> bool {
        let set = self.0.read();
        set.contains(&Permission::Administrator)
    }

    pub fn has_permission(&self, permission: Permission) -> bool {
        let set = self.0.read();
        if self.is_administrator() {
            true
        } else {
            for perm in set.clone() {
                if perm.kind() == permission.kind()
                    && perm.root() == permission.root()
                    && perm.administrate() == permission.administrate()
                {
                    if perm.capability().has_at_least(permission.capability()) {
                        return true;
                    }
                }
            }

            false
        }
    }
}
