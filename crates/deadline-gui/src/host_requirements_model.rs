//! Host requirements QObject model.
//!
//! Exposes host requirement UI state to QML and provides serialization
//! to the job template `hostRequirements` JSON format.

use core::pin::Pin;
use cxx_qt_lib::QString;

use crate::logic::host_requirements::{self, HostRequirementsModelState};

#[derive(Clone)]
pub struct HostRequirementsModelRust {
    use_custom_requirements: bool,
    // OS
    os_linux: bool,
    os_macos: bool,
    os_windows: bool,
    cpu_x86_64: bool,
    cpu_arm64: bool,
    // Hardware (-1 = unset)
    cpu_min: i32,
    cpu_max: i32,
    memory_gib_min: i32,
    memory_gib_max: i32,
    gpu_min: i32,
    gpu_max: i32,
    gpu_memory_gib_min: i32,
    gpu_memory_gib_max: i32,
    scratch_gib_min: i32,
    scratch_gib_max: i32,
    // Custom (pipe-separated)
    custom_amounts: QString,
    custom_attributes: QString,
}

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, use_custom_requirements)]
        #[qproperty(bool, os_linux)]
        #[qproperty(bool, os_macos)]
        #[qproperty(bool, os_windows)]
        #[qproperty(bool, cpu_x86_64)]
        #[qproperty(bool, cpu_arm64)]
        #[qproperty(i32, cpu_min)]
        #[qproperty(i32, cpu_max)]
        #[qproperty(i32, memory_gib_min)]
        #[qproperty(i32, memory_gib_max)]
        #[qproperty(i32, gpu_min)]
        #[qproperty(i32, gpu_max)]
        #[qproperty(i32, gpu_memory_gib_min)]
        #[qproperty(i32, gpu_memory_gib_max)]
        #[qproperty(i32, scratch_gib_min)]
        #[qproperty(i32, scratch_gib_max)]
        #[qproperty(QString, custom_amounts)]
        #[qproperty(QString, custom_attributes)]
        type HostRequirementsModel = super::HostRequirementsModelRust;

        /// Serialize current state to JSON string, or empty string if disabled.
        #[qinvokable]
        fn serialize(self: &HostRequirementsModel) -> QString;

        #[qinvokable]
        fn add_custom_amount(self: Pin<&mut HostRequirementsModel>);

        #[qinvokable]
        fn remove_custom_amount(self: Pin<&mut HostRequirementsModel>, index: i32);

        #[qinvokable]
        fn update_custom_amount(self: Pin<&mut HostRequirementsModel>, index: i32, name: QString, min: i32, max: i32);

        #[qinvokable]
        fn add_custom_attribute(self: Pin<&mut HostRequirementsModel>);

        #[qinvokable]
        fn remove_custom_attribute(self: Pin<&mut HostRequirementsModel>, index: i32);

        #[qinvokable]
        fn update_custom_attribute(self: Pin<&mut HostRequirementsModel>, index: i32, name: QString, option: QString, values: QString);
    }
}

impl Default for HostRequirementsModelRust {
    fn default() -> Self {
        Self {
            use_custom_requirements: false,
            os_linux: false,
            os_macos: false,
            os_windows: false,
            cpu_x86_64: false,
            cpu_arm64: false,
            cpu_min: -1,
            cpu_max: -1,
            memory_gib_min: -1,
            memory_gib_max: -1,
            gpu_min: -1,
            gpu_max: -1,
            gpu_memory_gib_min: -1,
            gpu_memory_gib_max: -1,
            scratch_gib_min: -1,
            scratch_gib_max: -1,
            custom_amounts: QString::default(),
            custom_attributes: QString::default(),
        }
    }
}

impl qobject::HostRequirementsModel {
    pub fn serialize(&self) -> QString {
        let state = HostRequirementsModelState {
            use_custom: *self.use_custom_requirements(),
            os_linux: *self.os_linux(),
            os_macos: *self.os_macos(),
            os_windows: *self.os_windows(),
            cpu_x86_64: *self.cpu_x86_64(),
            cpu_arm64: *self.cpu_arm64(),
            cpu_min: *self.cpu_min(),
            cpu_max: *self.cpu_max(),
            memory_gib_min: *self.memory_gib_min(),
            memory_gib_max: *self.memory_gib_max(),
            gpu_min: *self.gpu_min(),
            gpu_max: *self.gpu_max(),
            gpu_memory_gib_min: *self.gpu_memory_gib_min(),
            gpu_memory_gib_max: *self.gpu_memory_gib_max(),
            scratch_gib_min: *self.scratch_gib_min(),
            scratch_gib_max: *self.scratch_gib_max(),
            custom_amounts: self.custom_amounts().to_string(),
            custom_attributes: self.custom_attributes().to_string(),
        };
        match host_requirements::serialize_from_model_state(&state) {
            Some(v) => QString::from(&serde_json::to_string(&v).unwrap_or_default()),
            None => QString::default(),
        }
    }

    pub fn add_custom_amount(mut self: Pin<&mut Self>) {
        let current = self.as_ref().custom_amounts().to_string();
        let new = if current.is_empty() {
            ";-1;-1".to_string()
        } else {
            format!("{current}|;-1;-1")
        };
        self.as_mut().set_custom_amounts(QString::from(&new));
    }

    pub fn remove_custom_amount(mut self: Pin<&mut Self>, index: i32) {
        let current = self.as_ref().custom_amounts().to_string();
        let entries: Vec<&str> = current.split('|').collect();
        let new: Vec<&str> = entries
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != index as usize)
            .map(|(_, e)| *e)
            .collect();
        self.as_mut()
            .set_custom_amounts(QString::from(&new.join("|")));
    }

    pub fn update_custom_amount(mut self: Pin<&mut Self>, index: i32, name: QString, min: i32, max: i32) {
        let current = self.as_ref().custom_amounts().to_string();
        let mut entries: Vec<String> = current.split('|').map(|s| s.to_string()).collect();
        if (index as usize) < entries.len() {
            entries[index as usize] = format!("{};{};{}", name, min, max);
            self.as_mut()
                .set_custom_amounts(QString::from(&entries.join("|")));
        }
    }

    pub fn add_custom_attribute(mut self: Pin<&mut Self>) {
        let current = self.as_ref().custom_attributes().to_string();
        let new = if current.is_empty() {
            ";anyOf;".to_string()
        } else {
            format!("{current}|;anyOf;")
        };
        self.as_mut().set_custom_attributes(QString::from(&new));
    }

    pub fn remove_custom_attribute(mut self: Pin<&mut Self>, index: i32) {
        let current = self.as_ref().custom_attributes().to_string();
        let entries: Vec<&str> = current.split('|').collect();
        let new: Vec<&str> = entries
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != index as usize)
            .map(|(_, e)| *e)
            .collect();
        self.as_mut()
            .set_custom_attributes(QString::from(&new.join("|")));
    }

    pub fn update_custom_attribute(mut self: Pin<&mut Self>, index: i32, name: QString, option: QString, values: QString) {
        let current = self.as_ref().custom_attributes().to_string();
        let mut entries: Vec<String> = current.split('|').map(|s| s.to_string()).collect();
        if (index as usize) < entries.len() {
            entries[index as usize] = format!("{};{};{}", name, option, values);
            self.as_mut()
                .set_custom_attributes(QString::from(&entries.join("|")));
        }
    }
}
