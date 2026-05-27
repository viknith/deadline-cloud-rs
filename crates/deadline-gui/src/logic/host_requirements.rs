//! Host requirements serialization logic.
//!
//! Converts UI state into the JSON format expected by the job template's
//! `hostRequirements` field.

/// OS requirements (operating system family and CPU architecture).
#[derive(Debug, Clone, Default)]
pub struct OsRequirements {
    pub operating_systems: Vec<String>,
    pub cpu_architectures: Vec<String>,
}

impl OsRequirements {
    pub fn serialize(&self) -> Vec<serde_json::Value> {
        let mut result = Vec::new();
        if !self.operating_systems.is_empty() {
            result.push(serde_json::json!({
                "name": "attr.worker.os.family",
                "anyOf": self.operating_systems,
            }));
        }
        if !self.cpu_architectures.is_empty() {
            result.push(serde_json::json!({
                "name": "attr.worker.cpu.arch",
                "anyOf": self.cpu_architectures,
            }));
        }
        result
    }
}

/// Hardware requirements (CPU, memory, GPU, scratch space).
#[derive(Debug, Clone, Default)]
pub struct HardwareRequirements {
    pub cpu_min: Option<i32>,
    pub cpu_max: Option<i32>,
    pub memory_min: Option<i32>,
    pub memory_max: Option<i32>,
    pub gpu_min: Option<i32>,
    pub gpu_max: Option<i32>,
    pub gpu_memory_min: Option<i32>,
    pub gpu_memory_max: Option<i32>,
    pub scratch_min: Option<i32>,
    pub scratch_max: Option<i32>,
}

impl HardwareRequirements {
    pub fn serialize(&self) -> Vec<serde_json::Value> {
        let mut result = Vec::new();
        Self::push_amount(
            &mut result,
            "amount.worker.vcpu",
            self.cpu_min,
            self.cpu_max,
        );
        Self::push_amount(
            &mut result,
            "amount.worker.memory",
            self.memory_min,
            self.memory_max,
        );
        Self::push_amount(&mut result, "amount.worker.gpu", self.gpu_min, self.gpu_max);
        Self::push_amount(
            &mut result,
            "amount.worker.gpu.memory",
            self.gpu_memory_min,
            self.gpu_memory_max,
        );
        Self::push_amount(
            &mut result,
            "amount.worker.disk.scratch",
            self.scratch_min,
            self.scratch_max,
        );
        result
    }

    fn push_amount(
        result: &mut Vec<serde_json::Value>,
        name: &str,
        min: Option<i32>,
        max: Option<i32>,
    ) {
        if min.is_none() && max.is_none() {
            return;
        }
        let mut obj = serde_json::json!({"name": name});
        if let Some(v) = min {
            obj["min"] = serde_json::json!(v);
        }
        if let Some(v) = max {
            obj["max"] = serde_json::json!(v);
        }
        result.push(obj);
    }
}

/// A custom amount requirement (e.g. render slots, licenses).
#[derive(Debug, Clone)]
pub struct CustomAmountRequirement {
    pub name: String,
    pub min: Option<i32>,
    pub max: Option<i32>,
}

impl CustomAmountRequirement {
    pub fn serialize(&self) -> serde_json::Value {
        let mut obj = serde_json::json!({"name": format!("amount.worker.{}", self.name)});
        if let Some(v) = self.min {
            obj["min"] = serde_json::json!(v);
        }
        if let Some(v) = self.max {
            obj["max"] = serde_json::json!(v);
        }
        obj
    }
}

/// A custom attribute requirement (e.g. department, software).
#[derive(Debug, Clone)]
pub struct CustomAttributeRequirement {
    pub name: String,
    pub option: String, // "anyOf" or "allOf"
    pub values: Vec<String>,
}

impl CustomAttributeRequirement {
    pub fn serialize(&self) -> serde_json::Value {
        let mut obj = serde_json::json!({"name": format!("attr.worker.{}", self.name)});
        obj[&self.option] = serde_json::json!(self.values);
        obj
    }
}

/// Combined host requirements.
#[derive(Debug, Clone, Default)]
pub struct HostRequirements {
    pub os: OsRequirements,
    pub hardware: HardwareRequirements,
    pub custom_amounts: Vec<CustomAmountRequirement>,
    pub custom_attributes: Vec<CustomAttributeRequirement>,
}

impl HostRequirements {
    pub fn serialize(&self) -> serde_json::Value {
        let mut amounts: Vec<serde_json::Value> = Vec::new();
        let mut attributes: Vec<serde_json::Value> = Vec::new();

        attributes.extend(self.os.serialize());
        amounts.extend(self.hardware.serialize());

        for ca in &self.custom_amounts {
            amounts.push(ca.serialize());
        }
        for ca in &self.custom_attributes {
            attributes.push(ca.serialize());
        }

        let mut result = serde_json::Map::new();
        if !amounts.is_empty() {
            result.insert("amounts".to_string(), serde_json::json!(amounts));
        }
        if !attributes.is_empty() {
            result.insert("attributes".to_string(), serde_json::json!(attributes));
        }
        serde_json::Value::Object(result)
    }
}

/// UI model state used by `serialize_from_model_state`.
#[derive(Debug, Clone, Default)]
pub struct HostRequirementsModelState {
    pub use_custom: bool,
    pub os_linux: bool,
    pub os_macos: bool,
    pub os_windows: bool,
    pub cpu_x86_64: bool,
    pub cpu_arm64: bool,
    /// Hardware values in GiB (for memory/gpu_memory/scratch). -1 = unset.
    pub cpu_min: i32,
    pub cpu_max: i32,
    pub memory_gib_min: i32,
    pub memory_gib_max: i32,
    pub gpu_min: i32,
    pub gpu_max: i32,
    pub gpu_memory_gib_min: i32,
    pub gpu_memory_gib_max: i32,
    pub scratch_gib_min: i32,
    pub scratch_gib_max: i32,
    /// Pipe-separated entries: "name;min;max|name;min;max"
    pub custom_amounts: String,
    /// Pipe-separated entries: "name;option;v1,v2|name;option;v1,v2"
    pub custom_attributes: String,
}

/// Convert model state to serialized JSON, or None if custom requirements disabled.
pub fn serialize_from_model_state(state: &HostRequirementsModelState) -> Option<serde_json::Value> {
    if !state.use_custom {
        return None;
    }

    fn opt(v: i32) -> Option<i32> {
        if v < 0 { None } else { Some(v) }
    }

    let mut os_list = Vec::new();
    if state.os_linux {
        os_list.push("linux".to_string());
    }
    if state.os_macos {
        os_list.push("macos".to_string());
    }
    if state.os_windows {
        os_list.push("windows".to_string());
    }

    let mut arch_list = Vec::new();
    if state.cpu_x86_64 {
        arch_list.push("x86_64".to_string());
    }
    if state.cpu_arm64 {
        arch_list.push("arm64".to_string());
    }

    let hr = HostRequirements {
        os: OsRequirements {
            operating_systems: os_list,
            cpu_architectures: arch_list,
        },
        hardware: HardwareRequirements {
            cpu_min: opt(state.cpu_min),
            cpu_max: opt(state.cpu_max),
            memory_min: opt(state.memory_gib_min).map(|v| v * 1024),
            memory_max: opt(state.memory_gib_max).map(|v| v * 1024),
            gpu_min: opt(state.gpu_min),
            gpu_max: opt(state.gpu_max),
            gpu_memory_min: opt(state.gpu_memory_gib_min).map(|v| v * 1024),
            gpu_memory_max: opt(state.gpu_memory_gib_max).map(|v| v * 1024),
            scratch_min: opt(state.scratch_gib_min).map(|v| v * 1024),
            scratch_max: opt(state.scratch_gib_max).map(|v| v * 1024),
        },
        custom_amounts: parse_custom_amounts(&state.custom_amounts),
        custom_attributes: parse_custom_attributes(&state.custom_attributes),
    };

    let result = hr.serialize();
    if result.as_object().unwrap().is_empty() {
        None
    } else {
        Some(result)
    }
}

fn parse_custom_amounts(encoded: &str) -> Vec<CustomAmountRequirement> {
    if encoded.is_empty() {
        return Vec::new();
    }
    encoded
        .split('|')
        .filter_map(|entry| {
            let parts: Vec<&str> = entry.split(';').collect();
            if parts.len() != 3 || parts[0].is_empty() {
                return None;
            }
            let min = parts[1].parse::<i32>().ok().filter(|&v| v >= 0);
            let max = parts[2].parse::<i32>().ok().filter(|&v| v >= 0);
            Some(CustomAmountRequirement {
                name: parts[0].to_string(),
                min,
                max,
            })
        })
        .collect()
}

fn parse_custom_attributes(encoded: &str) -> Vec<CustomAttributeRequirement> {
    if encoded.is_empty() {
        return Vec::new();
    }
    encoded
        .split('|')
        .filter_map(|entry| {
            let parts: Vec<&str> = entry.split(';').collect();
            if parts.len() != 3 || parts[0].is_empty() {
                return None;
            }
            let values: Vec<String> = parts[2]
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
            Some(CustomAttributeRequirement {
                name: parts[0].to_string(),
                option: parts[1].to_string(),
                values,
            })
        })
        .collect()
}

/// Validate a custom requirement name against the OpenJD naming rules.
/// Pattern: `^([a-zA-Z_][a-zA-Z0-9_]{0,63})(\.[a-zA-Z_][a-zA-Z0-9_]{0,63})*$`
pub fn validate_custom_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    name.split('.').all(|segment| {
        !segment.is_empty()
            && segment.len() <= 64
            && segment
                .chars()
                .next()
                .map_or(false, |c| c.is_ascii_alphabetic() || c == '_')
            && segment
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_requirements_empty_serializes_to_empty() {
        let req = OsRequirements::default();
        assert!(req.serialize().is_empty());
    }

    #[test]
    fn os_requirements_single_os() {
        let req = OsRequirements {
            operating_systems: vec!["linux".to_string()],
            cpu_architectures: vec![],
        };
        let result = req.serialize();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["name"], "attr.worker.os.family");
        assert_eq!(result[0]["anyOf"], serde_json::json!(["linux"]));
    }

    #[test]
    fn os_requirements_multiple_os_and_arch() {
        let req = OsRequirements {
            operating_systems: vec!["linux".to_string(), "windows".to_string()],
            cpu_architectures: vec!["x86_64".to_string()],
        };
        let result = req.serialize();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["anyOf"], serde_json::json!(["linux", "windows"]));
        assert_eq!(result[1]["name"], "attr.worker.cpu.arch");
    }

    #[test]
    fn hardware_requirements_default_serializes_to_empty() {
        assert!(HardwareRequirements::default().serialize().is_empty());
    }

    #[test]
    fn hardware_requirements_cpu_min_only() {
        let req = HardwareRequirements {
            cpu_min: Some(4),
            ..Default::default()
        };
        let result = req.serialize();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["name"], "amount.worker.vcpu");
        assert_eq!(result[0]["min"], 4);
        assert!(result[0].get("max").is_none());
    }

    #[test]
    fn hardware_requirements_cpu_min_and_max() {
        let req = HardwareRequirements {
            cpu_min: Some(2),
            cpu_max: Some(16),
            ..Default::default()
        };
        let result = req.serialize();
        assert_eq!(result[0]["min"], 2);
        assert_eq!(result[0]["max"], 16);
    }

    #[test]
    fn hardware_requirements_memory_gpu_scratch() {
        let req = HardwareRequirements {
            memory_min: Some(8192),
            gpu_min: Some(1),
            gpu_memory_min: Some(4096),
            scratch_min: Some(100),
            ..Default::default()
        };
        let result = req.serialize();
        assert_eq!(result.len(), 4);
        let names: Vec<&str> = result.iter().map(|r| r["name"].as_str().unwrap()).collect();
        assert!(names.contains(&"amount.worker.memory"));
        assert!(names.contains(&"amount.worker.gpu"));
        assert!(names.contains(&"amount.worker.gpu.memory"));
        assert!(names.contains(&"amount.worker.disk.scratch"));
    }

    #[test]
    fn custom_amount_min_only() {
        let req = CustomAmountRequirement {
            name: "render.slots".to_string(),
            min: Some(2),
            max: None,
        };
        let result = req.serialize();
        assert_eq!(result["name"], "amount.worker.render.slots");
        assert_eq!(result["min"], 2);
        assert!(result.get("max").is_none());
    }

    #[test]
    fn custom_amount_min_and_max() {
        let req = CustomAmountRequirement {
            name: "licenses".to_string(),
            min: Some(1),
            max: Some(10),
        };
        let result = req.serialize();
        assert_eq!(result["min"], 1);
        assert_eq!(result["max"], 10);
    }

    #[test]
    fn custom_attribute_any_of() {
        let req = CustomAttributeRequirement {
            name: "department".to_string(),
            option: "anyOf".to_string(),
            values: vec!["lighting".to_string(), "compositing".to_string()],
        };
        let result = req.serialize();
        assert_eq!(result["name"], "attr.worker.department");
        assert_eq!(
            result["anyOf"],
            serde_json::json!(["lighting", "compositing"])
        );
    }

    #[test]
    fn custom_attribute_all_of() {
        let req = CustomAttributeRequirement {
            name: "software".to_string(),
            option: "allOf".to_string(),
            values: vec!["maya".to_string(), "arnold".to_string()],
        };
        let result = req.serialize();
        assert_eq!(result["name"], "attr.worker.software");
        assert_eq!(result["allOf"], serde_json::json!(["maya", "arnold"]));
    }

    #[test]
    fn host_requirements_full_serialization() {
        let req = HostRequirements {
            os: OsRequirements {
                operating_systems: vec!["linux".to_string()],
                cpu_architectures: vec!["x86_64".to_string()],
            },
            hardware: HardwareRequirements {
                cpu_min: Some(4),
                memory_min: Some(8192),
                ..Default::default()
            },
            custom_amounts: vec![CustomAmountRequirement {
                name: "slots".to_string(),
                min: Some(1),
                max: None,
            }],
            custom_attributes: vec![CustomAttributeRequirement {
                name: "pool".to_string(),
                option: "anyOf".to_string(),
                values: vec!["render".to_string()],
            }],
        };
        let result = req.serialize();
        assert!(result.get("amounts").is_some());
        assert!(result.get("attributes").is_some());
        assert!(result["amounts"].as_array().unwrap().len() >= 3);
        assert!(result["attributes"].as_array().unwrap().len() >= 3);
    }

    #[test]
    fn host_requirements_empty_serializes_to_empty() {
        let req = HostRequirements::default();
        assert!(req.serialize().as_object().unwrap().is_empty());
    }

    // ── Batch 2c: Model state tests ──

    #[test]
    fn serialize_from_model_state_disabled_returns_none() {
        let state = HostRequirementsModelState {
            use_custom: false,
            os_linux: true, // even with values set, disabled → None
            cpu_min: 4,
            ..Default::default()
        };
        assert!(serialize_from_model_state(&state).is_none());
    }

    #[test]
    fn serialize_from_model_state_gib_to_mib() {
        let state = HostRequirementsModelState {
            use_custom: true,
            memory_gib_min: 8,
            memory_gib_max: -1,
            gpu_memory_gib_min: 4,
            gpu_memory_gib_max: -1,
            scratch_gib_min: 100,
            scratch_gib_max: -1,
            cpu_min: -1,
            cpu_max: -1,
            gpu_min: -1,
            gpu_max: -1,
            ..Default::default()
        };
        let result = serialize_from_model_state(&state).unwrap();
        let amounts = result["amounts"].as_array().unwrap();
        let mem = amounts.iter().find(|a| a["name"] == "amount.worker.memory").unwrap();
        assert_eq!(mem["min"], 8192); // 8 GiB × 1024
        let gpu_mem = amounts.iter().find(|a| a["name"] == "amount.worker.gpu.memory").unwrap();
        assert_eq!(gpu_mem["min"], 4096); // 4 GiB × 1024
        let scratch = amounts.iter().find(|a| a["name"] == "amount.worker.disk.scratch").unwrap();
        assert_eq!(scratch["min"], 100 * 1024); // 100 GiB × 1024
    }

    #[test]
    fn serialize_from_model_state_full() {
        let state = HostRequirementsModelState {
            use_custom: true,
            os_linux: true,
            os_windows: true,
            os_macos: false,
            cpu_x86_64: true,
            cpu_arm64: false,
            cpu_min: 4,
            cpu_max: 16,
            memory_gib_min: 8,
            memory_gib_max: -1,
            gpu_min: -1,
            gpu_max: -1,
            gpu_memory_gib_min: -1,
            gpu_memory_gib_max: -1,
            scratch_gib_min: -1,
            scratch_gib_max: -1,
            custom_amounts: "render.slots;2;10".to_string(),
            custom_attributes: "dept;anyOf;lighting,compositing".to_string(),
        };
        let result = serialize_from_model_state(&state).unwrap();

        // OS attributes
        let attrs = result["attributes"].as_array().unwrap();
        let os = attrs.iter().find(|a| a["name"] == "attr.worker.os.family").unwrap();
        assert_eq!(os["anyOf"], serde_json::json!(["linux", "windows"]));
        let arch = attrs.iter().find(|a| a["name"] == "attr.worker.cpu.arch").unwrap();
        assert_eq!(arch["anyOf"], serde_json::json!(["x86_64"]));

        // Custom attribute
        let dept = attrs.iter().find(|a| a["name"] == "attr.worker.dept").unwrap();
        assert_eq!(dept["anyOf"], serde_json::json!(["lighting", "compositing"]));

        // Hardware amounts
        let amounts = result["amounts"].as_array().unwrap();
        let cpu = amounts.iter().find(|a| a["name"] == "amount.worker.vcpu").unwrap();
        assert_eq!(cpu["min"], 4);
        assert_eq!(cpu["max"], 16);

        // Custom amount
        let slots = amounts.iter().find(|a| a["name"] == "amount.worker.render.slots").unwrap();
        assert_eq!(slots["min"], 2);
        assert_eq!(slots["max"], 10);
    }

    #[test]
    fn validate_custom_name_valid() {
        assert!(validate_custom_name("render.slots"));
        assert!(validate_custom_name("my_name"));
        assert!(validate_custom_name("a"));
        assert!(validate_custom_name("_private"));
        assert!(validate_custom_name("segment1.segment2.segment3"));
    }

    #[test]
    fn validate_custom_name_invalid() {
        assert!(!validate_custom_name("")); // empty
        assert!(!validate_custom_name("1starts_with_digit"));
        assert!(!validate_custom_name("has space"));
        assert!(!validate_custom_name("has-dash"));
        assert!(!validate_custom_name(".starts_with_dot"));
        assert!(!validate_custom_name("ends_with_dot."));
        assert!(!validate_custom_name("a.1digit"));
        // 65-char segment exceeds 64-char limit
        let long_segment = "a".repeat(65);
        assert!(!validate_custom_name(&long_segment));
    }
}
