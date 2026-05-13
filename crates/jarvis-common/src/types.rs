use serde::{Deserialize, Serialize};

/// Hardware profile matching Emir's machine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub cpu: String,
    pub ram_gb: u32,
    pub gpu: String,
    pub vram_gb: u32,
    #[serde(default = "default_vram_reserve")]
    pub vram_reserve_gb: f32,
    #[serde(default = "default_ram_reserve")]
    pub ram_reserve_gb: f32,
}

fn default_vram_reserve() -> f32 {
    1.0
}
fn default_ram_reserve() -> f32 {
    4.0
}

impl Default for HardwareProfile {
    fn default() -> Self {
        Self {
            cpu: "AMD Ryzen 5 7600X".into(),
            ram_gb: 32,
            gpu: "NVIDIA RTX 4060".into(),
            vram_gb: 8,
            vram_reserve_gb: 1.0,
            ram_reserve_gb: 4.0,
        }
    }
}

/// System identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub name: String,
    pub version_mark: u32,
    pub version_semver: String,
    pub creator: String,
    pub language: String,
    pub offline_mode_capable: bool,
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self {
            name: "J.A.R.V.I.S.".into(),
            version_mark: 1,
            version_semver: "2.0.1".into(),
            creator: "Emir".into(),
            language: "tr-TR".into(),
            offline_mode_capable: true,
        }
    }
}

/// Mark version identifier (Mark I, Mark II, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkVersion(pub u32);

impl MarkVersion {
    pub fn roman(&self) -> String {
        to_roman(self.0)
    }

    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

impl std::fmt::Display for MarkVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mark {}", self.roman())
    }
}

fn to_roman(mut n: u32) -> String {
    let numerals = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut result = String::new();
    for &(value, symbol) in &numerals {
        while n >= value {
            result.push_str(symbol);
            n -= value;
        }
    }
    result
}

/// Account permission level.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountRole {
    Admin,
    Pose,
    Guest,
}

/// Subsystem health status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Down,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roman_numerals() {
        assert_eq!(to_roman(1), "I");
        assert_eq!(to_roman(2), "II");
        assert_eq!(to_roman(4), "IV");
        assert_eq!(to_roman(39), "XXXIX");
    }

    #[test]
    fn mark_version_display() {
        let m = MarkVersion(1);
        assert_eq!(m.to_string(), "Mark I");
        assert_eq!(m.next().to_string(), "Mark II");
    }
}
