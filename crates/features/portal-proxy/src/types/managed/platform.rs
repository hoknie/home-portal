#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Platform {
    pub system: &'static str,
    pub processor: &'static str,
}

impl Platform {
    pub fn of(system: &str, processor: &str) -> Result<Platform, String> {
        let system = match system {
            "macos" => "mac",
            "linux" => "linux",
            other => {
                return Err(format!(
                    "Caddy cannot be downloaded for the operating system {other}; install it by hand"
                ));
            }
        };
        let processor = match processor {
            "aarch64" => "arm64",
            "x86_64" => "amd64",
            "arm" => "armv7",
            other => {
                return Err(format!(
                    "Caddy cannot be downloaded for the processor {other}; install it by hand"
                ));
            }
        };
        Ok(Platform { system, processor })
    }

    pub fn current() -> Result<Platform, String> {
        Self::of(std::env::consts::OS, std::env::consts::ARCH)
    }

    pub fn name(&self) -> String {
        format!("{}_{}", self.system, self.processor)
    }

    pub fn archive(&self, version: &str) -> String {
        format!("caddy_{version}_{}_{}.tar.gz", self.system, self.processor)
    }

    pub fn checksums(version: &str) -> String {
        format!("caddy_{version}_checksums.txt")
    }
}
