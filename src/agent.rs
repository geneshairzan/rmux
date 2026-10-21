//! AI coding agent detection and process tracking.

use std::fs;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AgentActivity {
    pub name: String,
    pub command: String,
    pub requires_attention: bool,
}

#[allow(dead_code)]
pub fn detect_agent(parent_pid: i32) -> Option<AgentActivity> {
    if parent_pid <= 0 {
        return None;
    }

    // Inspect child processes of parent_pid
    let mut to_visit = vec![parent_pid];
    let mut visited = Vec::new();

    while let Some(pid) = to_visit.pop() {
        if visited.contains(&pid) {
            continue;
        }
        visited.push(pid);

        // Read cmdline & comm
        let comm_path = format!("/proc/{}/comm", pid);
        let cmdline_path = format!("/proc/{}/cmdline", pid);

        let name = fs::read_to_string(&comm_path).unwrap_or_default().trim().to_string();
        let cmdline = fs::read(&cmdline_path)
            .map(|bytes| {
                bytes
                    .split(|&b| b == 0)
                    .filter_map(|s| std::str::from_utf8(s).ok())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default();

        let lower_cmd = cmdline.to_lowercase();
        let lower_name = name.to_lowercase();

        let agents = [
            ("claude", "Claude Code"),
            ("codex", "Codex"),
            ("aider", "Aider"),
            ("opencode", "OpenCode"),
            ("gemini", "Gemini CLI"),
            ("ollama", "Ollama"),
            ("bun", "Bun"),
            ("npm", "npm"),
            ("cargo", "Cargo"),
        ];

        for (key, display) in agents {
            if lower_name == key || lower_cmd.contains(&format!("/{}", key)) || lower_cmd.contains(&format!("{} ", key)) {
                let attention = lower_cmd.contains("waiting") || lower_cmd.contains("input");
                return Some(AgentActivity {
                    name: display.to_string(),
                    command: cmdline,
                    requires_attention: attention,
                });
            }
        }

        // Find child PIDs
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if let Ok(c_pid) = file_name.parse::<i32>() {
                        if c_pid != pid {
                            let status_path = format!("/proc/{}/status", c_pid);
                            if let Ok(status) = fs::read_to_string(status_path) {
                                for line in status.lines() {
                                    if line.starts_with("PPid:") {
                                        if let Some(ppid_str) = line.split_whitespace().nth(1) {
                                            if ppid_str.parse::<i32>() == Ok(pid) {
                                                to_visit.push(c_pid);
                                            }
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}
