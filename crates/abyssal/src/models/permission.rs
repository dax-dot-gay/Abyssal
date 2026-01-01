use std::fmt::Debug;

use abyssal_macros::make_permissions;
use getset::{CloneGetters, WithSetters};
use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PermissionTarget {
    Group,
    User,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, CloneGetters, WithSetters)]
pub struct PermissionModel {
    #[serde(default)]
    id: Uuid,

    target_type: PermissionTarget,
    target_id: Uuid,
}

/*make_permissions! {
    Permissions => {
        Authentication => {
            Users => {
                Existing => {
                    Password [comment = "Reset user passwords";};
                    Groups "Manage user group membership";
                    Permissions => "prefix" "Manage user permissions within <prefix>"; // Cannot grant permissions not personally held
                } "Manage other users";
                Register "Register new users";
            } "Manage local/oidc users";
            Groups => {
                Create "Create new groups";
                Modify "Modify existing groups";
                Permissions => "prefix" "Manage group permissions within <prefix>"; // Cannot grant permissions not personally held
            } "Allows modification of user groups";
            Oidc => {
                View "Only allows viewing OIDC provider information";
            } "Allows modification of OIDC provider(s)";
            Applications => {
                Owned "Only allows management of owned applications";
                Others => "application" {
                    View "Only allows viewing this application";
                } "Allows management of other applications (app name in <application>)";
            } "Allows modification of API clients";
        } "Manage authentication";
        Roots => "root" {
            Manage "Manage this root directory's configuration";
            Read => "path" "Read files/folders in this <root> within <path>";
            Edit => "path" "Edit files/folders in this <root> within <path>";
            Create => "path" "Create files/folders in this <root> within <path>";
            Delete => "path" "Delete files/folders in this <root> within <path>";
        };
    }
}*/

make_permissions! {
    Permissions => {
        Authentication => {
            Users => {
                Existing => {
                    View [comment = "View administrative user information"];
                    Password [comment = "Reset user passwords", depends = ["view"]];
                    Groups [comment = "Manage group membership", depends = ["view"]];
                    Permissions => "prefix" [comment = "Manage user permissions within <prefix>", depends = ["view"]];
                } [comment = "Manage existing users"];
                Register [comment = "Register new users"];
            } [comment = "Manage local/oidc users"];
            Groups => {
                View [comment = "View existing groups"];
                Create [comment = "Create new groups", depends = ["view"]];
                Modify [comment = "Modify existing groups", depends = ["view"]];
                Permissions => "prefix" [comment = "Manage group permissions within <prefix>", depends = ["view"]];
            } [comment = "Manage user groups", depends = ["users.existing.view"]];
            Oidc [comment = "Manage OIDC providers", depends = ["users.*", "groups.*"]];
        } [comment = "Manage authentication"];
        Roots => "root" {
            Read => "path" [comment = "Read files/folders in this <root> within <path>"];
            Edit => "path" [comment = "Read files/folders in this <root> within <path>", depends = ["read"]];
            Create => "path" [comment = "Create files/folders in this <root> within <path>", depends = ["edit"]];
            Delete => "path" [comment = "Delete files/folders in this <root> within <path>", depends = ["edit"]];
            Manage [comment = "Manage this root's configuration"];
        } [comment = "Manage the root named <root>"];
    }
}

impl PermissionsDescription {
    pub fn at_path(&self, path: impl Into<String>) -> Option<Self> {
        let path = path.into();
        if let Some((head, tail)) = path.split_once(".") {
            match self.clone() {
                PermissionsDescription::Leaf { .. } => None,
                PermissionsDescription::Branch { nodes, .. } => {
                    if head == "*" {
                        None
                    } else if tail == "*" {
                        nodes.into_iter().find(|node| node.name() == head.to_string())
                    } else {
                        nodes.into_iter().find(|node| node.name() == head.to_string()).and_then(|desc| desc.at_path(tail))
                    }
                },
                PermissionsDescription::StringLeaf { .. } => None,
                PermissionsDescription::StringBranch { nodes, .. } => {
                    if tail == "*" {
                        Some(self.clone())
                    } else if tail.starts_with("*") {
                        None
                    } else {
                        if let Some((head, tail)) = tail.split_once(".") {
                            if tail.starts_with("*") {
                                nodes.into_iter().find(|node| node.name() == head.to_string())
                            } else {
                                nodes.into_iter().find(|node| node.name() == head.to_string()).and_then(|desc| desc.at_path(tail))
                            }
                        } else {
                            nodes.into_iter().find(|node| node.name() == head.to_string())
                        }
                    }
                },
            }
        } else {
            match self.clone() {
                PermissionsDescription::Leaf { .. } => Some(self.clone()),
                PermissionsDescription::Branch { nodes, .. } => nodes.into_iter().find(|node| node.name() == path),
                PermissionsDescription::StringLeaf { .. } => Some(self.clone()),
                PermissionsDescription::StringBranch { nodes, .. } => nodes.into_iter().find(|node| node.name() == path),
            }
        }
    }
}
