use crate::model::ContainerInfo;
use std::process::Command;

const FIELD_DELIM: &str = "|";

pub struct DockerClient;

impl DockerClient {
    pub fn list_containers() -> Result<Vec<ContainerInfo>, String> {
        let output = Self::run(&[
            "ps",
            "-a",
            "--format",
            "{{.ID}}|{{.Names}}|{{.Status}}|{{.Image}}",
        ])?;
        Ok(parse_container_lines(&output))
    }

    pub fn start_container(name: &str) -> Result<String, String> {
        Self::run(&["start", name]).map(|out| {
            if out.is_empty() {
                format!("Container '{name}' started.")
            } else {
                out
            }
        })
    }

    pub fn stop_container(name: &str) -> Result<String, String> {
        Self::run(&["stop", name]).map(|out| {
            if out.is_empty() {
                format!("Container '{name}' stopped.")
            } else {
                out
            }
        })
    }

    pub fn remove_container(name: &str) -> Result<String, String> {
        Self::run(&["rm", name]).map(|out| {
            if out.is_empty() {
                format!("Container '{name}' removed.")
            } else {
                out
            }
        })
    }

    fn run(args: &[&str]) -> Result<String, String> {
        let output = Command::new("docker")
            .args(args)
            .output()
            .map_err(|err| format!("Unable to run docker command: {err}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

        if output.status.success() {
            Ok(stdout)
        } else if !stderr.is_empty() {
            Err(stderr)
        } else if !stdout.is_empty() {
            Err(stdout)
        } else {
            Err(format!("Docker command failed with status {}", output.status))
        }
    }
}

fn parse_container_lines(raw: &str) -> Vec<ContainerInfo> {
    raw.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }

            let mut fields = line.splitn(4, FIELD_DELIM);
            let id = fields.next()?.trim();
            let name = fields.next()?.trim();
            let status = fields.next()?.trim();
            let image = fields.next()?.trim();

            if id.is_empty() || name.is_empty() {
                return None;
            }

            Some(ContainerInfo {
                id: id.to_string(),
                name: name.to_string(),
                status: status.to_string(),
                image: image.to_string(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_container_lines;

    #[test]
    fn parses_well_formed_lines() {
        let raw = "1a2b|web|Up 2 minutes|nginx:latest\n3c4d|db|Exited (0)|postgres:16";
        let containers = parse_container_lines(raw);

        assert_eq!(containers.len(), 2);
        assert_eq!(containers[0].name, "web");
        assert_eq!(containers[1].status, "Exited (0)");
    }

    #[test]
    fn skips_malformed_lines() {
        let raw = "badline\n|||\n6f7g|cache|Up|redis";
        let containers = parse_container_lines(raw);

        assert_eq!(containers.len(), 1);
        assert_eq!(containers[0].name, "cache");
    }
}
