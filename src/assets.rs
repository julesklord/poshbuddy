use crate::app::{PluginAsset, SegmentAsset};

/// Returns the predefined list of legacy PowerShell plugins/modules supported by PoshBuddy.
pub fn get_default_plugins() -> Vec<PluginAsset> {
    vec![
        PluginAsset {
            name: "Terminal-Icons".to_string(),
            description: "Adds file and folder icons to your terminal outputs (ls, dir).".to_string(),
            module_name: "Terminal-Icons".to_string(),
            init_script: None,
        },
        PluginAsset {
            name: "zoxide (z Explorer)".to_string(),
            description: "A smarter cd command. It remembers which directories you use most often.".to_string(),
            module_name: "zoxide".to_string(),
            init_script: Some("if (Get-Command zoxide -ErrorAction SilentlyContinue) { zoxide init powershell --hook pwd | Out-String | Invoke-Expression }".to_string()),
        },
        PluginAsset {
            name: "PSReadLine Mastery".to_string(),
            description: "Enables Predictive IntelliSense (fish-like) and syntax highlighting.".to_string(),
            module_name: "PSReadLine".to_string(),
            init_script: Some("Set-PSReadLineOption -PredictionSource History\nSet-PSReadLineOption -PredictionViewStyle ListView".to_string()),
        },
    ]
}

/// Returns the comprehensive predefined list of official Oh My Posh segments.
pub fn get_default_segments() -> Vec<SegmentAsset> {
    vec![
        // --- Version Control ---
        SegmentAsset {
            name: "Git Status".to_string(),
            segment_type: "git".to_string(),
            description: "Shows current branch, working state, and Git file status.".to_string(),
            category: "Version Control".to_string(),
        },
        SegmentAsset {
            name: "GitVersion".to_string(),
            segment_type: "gitversion".to_string(),
            description: "Displays semantic version determined by GitVersion.".to_string(),
            category: "Version Control".to_string(),
        },
        SegmentAsset {
            name: "Fossil".to_string(),
            segment_type: "fossil".to_string(),
            description: "Shows status for Fossil version control repositories.".to_string(),
            category: "Version Control".to_string(),
        },
        SegmentAsset {
            name: "Mercurial (hg)".to_string(),
            segment_type: "mercurial".to_string(),
            description: "Shows branch and status for Mercurial repositories.".to_string(),
            category: "Version Control".to_string(),
        },
        SegmentAsset {
            name: "Subversion (svn)".to_string(),
            segment_type: "subversion".to_string(),
            description: "Shows working copy status for Subversion repositories.".to_string(),
            category: "Version Control".to_string(),
        },
        SegmentAsset {
            name: "Plastic SCM".to_string(),
            segment_type: "plastic".to_string(),
            description: "Shows active branch and changeset for Plastic SCM.".to_string(),
            category: "Version Control".to_string(),
        },
        SegmentAsset {
            name: "Yadm".to_string(),
            segment_type: "yadm".to_string(),
            description: "Displays status of Yadm dotfile repository.".to_string(),
            category: "Version Control".to_string(),
        },
        // --- System & General ---
        SegmentAsset {
            name: "Path".to_string(),
            segment_type: "path".to_string(),
            description:
                "Shows current location in the file system with customizable folder depth."
                    .to_string(),
            category: "System".to_string(),
        },
        SegmentAsset {
            name: "Session (User/Host)".to_string(),
            segment_type: "session".to_string(),
            description: "Displays current user name, hostname, or SSH indicator.".to_string(),
            category: "System".to_string(),
        },
        SegmentAsset {
            name: "Operating System".to_string(),
            segment_type: "os".to_string(),
            description: "Displays OS logo/icon (Windows, macOS, Linux distributions).".to_string(),
            category: "System".to_string(),
        },
        SegmentAsset {
            name: "Shell info".to_string(),
            segment_type: "shell".to_string(),
            description: "Shows active shell name and version (pwsh, bash, zsh, fish).".to_string(),
            category: "System".to_string(),
        },
        SegmentAsset {
            name: "Execution Time".to_string(),
            segment_type: "executiontime".to_string(),
            description: "Shows duration of the last executed command.".to_string(),
            category: "System".to_string(),
        },
        SegmentAsset {
            name: "Command Status".to_string(),
            segment_type: "status".to_string(),
            description: "Displays success icon or error code of the last command.".to_string(),
            category: "System".to_string(),
        },
        SegmentAsset {
            name: "Exit Code".to_string(),
            segment_type: "exit".to_string(),
            description: "Displays numerical exit status code when last command failed."
                .to_string(),
            category: "System".to_string(),
        },
        SegmentAsset {
            name: "Root / Admin".to_string(),
            segment_type: "root".to_string(),
            description: "Displays indicator when running as Administrator or root user."
                .to_string(),
            category: "System".to_string(),
        },
        SegmentAsset {
            name: "Battery Status".to_string(),
            segment_type: "battery".to_string(),
            description: "Displays battery percentage, health, and charging status.".to_string(),
            category: "System".to_string(),
        },
        SegmentAsset {
            name: "System Memory / CPU".to_string(),
            segment_type: "sysinfo".to_string(),
            description: "Displays active RAM usage, CPU load, or system memory stats.".to_string(),
            category: "System".to_string(),
        },
        SegmentAsset {
            name: "Spotify Track".to_string(),
            segment_type: "spotify".to_string(),
            description: "Shows currently playing Spotify track and artist.".to_string(),
            category: "System".to_string(),
        },
        SegmentAsset {
            name: "Weather Info".to_string(),
            segment_type: "weather".to_string(),
            description: "Shows current local weather conditions and temperature.".to_string(),
            category: "System".to_string(),
        },
        SegmentAsset {
            name: "WakaTime".to_string(),
            segment_type: "wakatime".to_string(),
            description: "Shows active WakaTime coding time statistics.".to_string(),
            category: "System".to_string(),
        },
        SegmentAsset {
            name: "Custom Command".to_string(),
            segment_type: "command".to_string(),
            description: "Runs custom shell command and displays its output.".to_string(),
            category: "System".to_string(),
        },
        // --- Development & Runtimes ---
        SegmentAsset {
            name: "Node.js info".to_string(),
            segment_type: "node".to_string(),
            description: "Shows active Node.js version in directory.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Python info".to_string(),
            segment_type: "python".to_string(),
            description: "Shows active Python version and virtualenv/conda environment."
                .to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Rust info".to_string(),
            segment_type: "rust".to_string(),
            description: "Shows active Rust compiler toolchain version.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Go info".to_string(),
            segment_type: "go".to_string(),
            description: "Shows active Go language version.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: ".NET info".to_string(),
            segment_type: "dotnet".to_string(),
            description: "Shows active .NET SDK / framework version.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "PHP version".to_string(),
            segment_type: "php".to_string(),
            description: "Shows active PHP version.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Ruby version".to_string(),
            segment_type: "ruby".to_string(),
            description: "Shows active Ruby version and chruby/rbenv gemset.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Java version".to_string(),
            segment_type: "java".to_string(),
            description: "Shows active Java JDK version.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Flutter info".to_string(),
            segment_type: "flutter".to_string(),
            description: "Shows active Flutter SDK version.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Zig version".to_string(),
            segment_type: "zig".to_string(),
            description: "Shows active Zig compiler version.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Bun info".to_string(),
            segment_type: "bun".to_string(),
            description: "Shows active Bun runtime version.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Deno info".to_string(),
            segment_type: "deno".to_string(),
            description: "Shows active Deno runtime version.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Angular info".to_string(),
            segment_type: "angular".to_string(),
            description: "Shows active Angular CLI version.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Elixir version".to_string(),
            segment_type: "elixir".to_string(),
            description: "Shows active Elixir / Erlang version.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Haskell info".to_string(),
            segment_type: "haskell".to_string(),
            description: "Shows active GHC / Haskell toolchain version.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Julia version".to_string(),
            segment_type: "julia".to_string(),
            description: "Shows active Julia version.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Kotlin version".to_string(),
            segment_type: "kotlin".to_string(),
            description: "Shows active Kotlin version.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Lua version".to_string(),
            segment_type: "lua".to_string(),
            description: "Shows active Lua interpreter version.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Nim version".to_string(),
            segment_type: "nim".to_string(),
            description: "Shows active Nim compiler version.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Perl version".to_string(),
            segment_type: "perl".to_string(),
            description: "Shows active Perl interpreter version.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "R info".to_string(),
            segment_type: "r".to_string(),
            description: "Shows active R language environment version.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Scala version".to_string(),
            segment_type: "scala".to_string(),
            description: "Shows active Scala version.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Swift version".to_string(),
            segment_type: "swift".to_string(),
            description: "Shows active Swift toolchain version.".to_string(),
            category: "Development".to_string(),
        },
        SegmentAsset {
            name: "Project Version".to_string(),
            segment_type: "project".to_string(),
            description: "Displays current project version from package.json, Cargo.toml, etc."
                .to_string(),
            category: "Development".to_string(),
        },
        // --- Package Managers ---
        SegmentAsset {
            name: "npm version".to_string(),
            segment_type: "npm".to_string(),
            description: "Shows active npm version.".to_string(),
            category: "Package Managers".to_string(),
        },
        SegmentAsset {
            name: "pnpm version".to_string(),
            segment_type: "pnpm".to_string(),
            description: "Shows active pnpm version.".to_string(),
            category: "Package Managers".to_string(),
        },
        SegmentAsset {
            name: "Yarn version".to_string(),
            segment_type: "yarn".to_string(),
            description: "Shows active Yarn package manager version.".to_string(),
            category: "Package Managers".to_string(),
        },
        SegmentAsset {
            name: "Cargo version".to_string(),
            segment_type: "cargo".to_string(),
            description: "Shows active Cargo package manager version.".to_string(),
            category: "Package Managers".to_string(),
        },
        SegmentAsset {
            name: "Composer version".to_string(),
            segment_type: "composer".to_string(),
            description: "Shows active PHP Composer version.".to_string(),
            category: "Package Managers".to_string(),
        },
        SegmentAsset {
            name: "Poetry environment".to_string(),
            segment_type: "poetry".to_string(),
            description: "Shows active Python Poetry environment.".to_string(),
            category: "Package Managers".to_string(),
        },
        // --- Cloud & Containers ---
        SegmentAsset {
            name: "Docker context".to_string(),
            segment_type: "docker".to_string(),
            description: "Shows current Docker status, context, and active daemon.".to_string(),
            category: "Cloud".to_string(),
        },
        SegmentAsset {
            name: "AWS Profile".to_string(),
            segment_type: "aws".to_string(),
            description: "Shows active AWS profile and region.".to_string(),
            category: "Cloud".to_string(),
        },
        SegmentAsset {
            name: "Azure Subscription".to_string(),
            segment_type: "az".to_string(),
            description: "Shows active Azure subscription name.".to_string(),
            category: "Cloud".to_string(),
        },
        SegmentAsset {
            name: "GCP Project".to_string(),
            segment_type: "gcp".to_string(),
            description: "Shows active Google Cloud Platform project.".to_string(),
            category: "Cloud".to_string(),
        },
        SegmentAsset {
            name: "Kubernetes (kubectl)".to_string(),
            segment_type: "kubectl".to_string(),
            description: "Shows active Kubernetes context and namespace.".to_string(),
            category: "Cloud".to_string(),
        },
        SegmentAsset {
            name: "Terraform Workspace".to_string(),
            segment_type: "terraform".to_string(),
            description: "Shows active Terraform workspace.".to_string(),
            category: "Cloud".to_string(),
        },
        SegmentAsset {
            name: "Cloud Foundry".to_string(),
            segment_type: "cf".to_string(),
            description: "Shows active Cloud Foundry target.".to_string(),
            category: "Cloud".to_string(),
        },
        SegmentAsset {
            name: "Helm Status".to_string(),
            segment_type: "helm".to_string(),
            description: "Shows active Helm chart version.".to_string(),
            category: "Cloud".to_string(),
        },
        SegmentAsset {
            name: "Tailscale".to_string(),
            segment_type: "tailscale".to_string(),
            description: "Shows active Tailscale mesh VPN connection status.".to_string(),
            category: "Cloud".to_string(),
        },
        // --- Time ---
        SegmentAsset {
            name: "System Time".to_string(),
            segment_type: "time".to_string(),
            description: "Displays current system time in formatted 12h/24h style.".to_string(),
            category: "Time".to_string(),
        },
        SegmentAsset {
            name: "Nightscout".to_string(),
            segment_type: "nightscout".to_string(),
            description: "Displays blood glucose metrics from Nightscout.".to_string(),
            category: "Time".to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_default_plugins() {
        let plugins = get_default_plugins();
        assert!(!plugins.is_empty());
        assert!(plugins.iter().any(|p| p.name == "Terminal-Icons"));
    }

    #[test]
    fn test_get_default_segments() {
        let segments = get_default_segments();
        assert!(!segments.is_empty());
        assert!(segments.iter().any(|s| s.segment_type == "git"));
        assert!(segments.iter().any(|s| s.segment_type == "path"));
        assert!(segments.iter().any(|s| s.segment_type == "node"));
    }
}
