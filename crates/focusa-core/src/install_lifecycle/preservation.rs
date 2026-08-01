use serde::{Deserialize, Serialize};

use super::{InstallLifecycleValidationError, LifecycleTransactionKind, MaintenanceAction};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreservationDisposition {
    Preserve,
    RemoveManagedArtifact,
    NotTouched,
    PurgeConfirmed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleDataClass {
    ManagedBinaries,
    Services,
    Integrations,
    FocusaState,
    LogsCaches,
    LicenseState,
    ProviderHarnessState,
    ProjectFiles,
    ProjectTaskData,
    OperatorAuthoredInstructions,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreservationItem {
    pub data_class: LifecycleDataClass,
    pub disposition: PreservationDisposition,
    pub owner_authorized: bool,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreservationDeclaration {
    pub action: MaintenanceAction,
    pub items: Vec<PreservationItem>,
    pub destructive_purge_confirmed: bool,
}

impl PreservationDeclaration {
    pub fn validate(&self) -> Result<(), InstallLifecycleValidationError> {
        const ALL_CLASSES: [LifecycleDataClass; 10] = [
            LifecycleDataClass::ManagedBinaries,
            LifecycleDataClass::Services,
            LifecycleDataClass::Integrations,
            LifecycleDataClass::FocusaState,
            LifecycleDataClass::LogsCaches,
            LifecycleDataClass::LicenseState,
            LifecycleDataClass::ProviderHarnessState,
            LifecycleDataClass::ProjectFiles,
            LifecycleDataClass::ProjectTaskData,
            LifecycleDataClass::OperatorAuthoredInstructions,
        ];
        if ALL_CLASSES.iter().any(|class| {
            self.items
                .iter()
                .filter(|item| item.data_class == *class)
                .count()
                != 1
        }) {
            return Err(InstallLifecycleValidationError::PreservationDeclarationIncomplete);
        }
        for item in &self.items {
            if item.disposition == PreservationDisposition::PurgeConfirmed
                && (!self.destructive_purge_confirmed || !item.owner_authorized)
            {
                return Err(InstallLifecycleValidationError::DestructiveActionNotAuthorized);
            }
            if matches!(
                item.data_class,
                LifecycleDataClass::ProviderHarnessState
                    | LifecycleDataClass::ProjectFiles
                    | LifecycleDataClass::ProjectTaskData
                    | LifecycleDataClass::OperatorAuthoredInstructions
            ) && matches!(
                item.disposition,
                PreservationDisposition::RemoveManagedArtifact
                    | PreservationDisposition::PurgeConfirmed
            ) && !item.owner_authorized
            {
                return Err(InstallLifecycleValidationError::DestructiveActionNotAuthorized);
            }
        }
        if self.action == MaintenanceAction::Uninstall
            && self.items.iter().any(|item| {
                matches!(
                    item.data_class,
                    LifecycleDataClass::FocusaState
                        | LifecycleDataClass::LicenseState
                        | LifecycleDataClass::ProviderHarnessState
                        | LifecycleDataClass::ProjectFiles
                        | LifecycleDataClass::ProjectTaskData
                        | LifecycleDataClass::OperatorAuthoredInstructions
                ) && item.disposition != PreservationDisposition::Preserve
            })
        {
            return Err(InstallLifecycleValidationError::UninstallMustPreserveUserData);
        }
        Ok(())
    }
}
