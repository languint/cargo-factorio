//! Hot-reload generation marker and control.lua probe for live script reload.

use std::net::UdpSocket;
use std::path::Path;
use std::time::Duration;

use crate::error::{CliError, CliResult};
use crate::status::{self, Status};
use crate::write_if_changed::write_if_changed;

const PROBE_MARKER: &str = "-- factorio-rs hot-reload probe";
const GEN_LUA: &str = "factorio_rs_reload_gen.lua";
/// Default localhost UDP port for CLI -> Factorio reload pings.
pub const DEFAULT_RELOAD_UDP_PORT: u16 = 34_201;
/// Payload prefix the in-game probe accepts.
pub const RELOAD_UDP_PAYLOAD: &str = "factorio-rs-reload";
const STAGE_FILES: &[&str] = &[
    "data.lua",
    "data-updates.lua",
    "data-final-fixes.lua",
    "settings.lua",
    "settings-updates.lua",
    "settings-final-fixes.lua",
];

/// How the in-Lua probe should trigger a live reload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReloadProbeMode {
    /// Single reload when a UDP ping arrives (test listen/rerun).
    Once,
    Twice,
}

/// UDP port Factorio must listen on (`--enable-lua-udp=<port>`).
///
/// Override with `FACTORIO_RS_UDP_PORT`.
pub fn reload_udp_port() -> u16 {
    std::env::var("FACTORIO_RS_UDP_PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_RELOAD_UDP_PORT)
}

/// CLI argument that enables Factorio's localhost UDP receiver for hot-reload.
pub fn enable_lua_udp_arg(port: u16) -> String {
    format!("--enable-lua-udp={port}")
}

/// Ask a running Factorio instance (with UDP enabled) to call `game.reload_mods()`.
///
/// Returns `Ok(true)` when a datagram was sent. A quiet network failure still
/// returns `Ok(false)` so sync can continue with a note, Factorio may simply not be running yet.
pub fn send_reload_ping(port: u16) -> std::io::Result<bool> {
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    socket.set_write_timeout(Some(Duration::from_millis(500)))?;
    let sent = socket.send_to(RELOAD_UDP_PAYLOAD.as_bytes(), ("127.0.0.1", port))?;
    Ok(sent > 0)
}

/// Result of [`inject_hot_reload`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HotReloadInject {
    pub generation: u64,
    /// `true` when the generation counter advanced (content changed).
    pub bumped: bool,
}

/// Options for [`inject_hot_reload`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HotReloadOptions {
    pub probe: ReloadProbeMode,
    /// When `false`, update `.factorio-rs` state and the control probe but do not
    /// write `lua/factorio_rs_reload_gen.lua` yet. Call [`publish_reload_gen`] after
    /// deploy so Factorio only sees the bump once the mod tree is ready.
    pub publish_gen: bool,
}

impl Default for HotReloadOptions {
    fn default() -> Self {
        Self {
            probe: ReloadProbeMode::Twice,
            publish_gen: true,
        }
    }
}

/// Ensure the control probe exists and advance the reload generation.
///
/// Generation only advances when **project sources** change (`src/**/*.rs`,
/// `Factorio.toml`, `Cargo.toml`). Rebuilding identical sources keeps the same
/// generation so Bacon/Factorio are not spuriously reloaded.
///
/// Convenience wrapper around [`inject_hot_reload_with`] with
/// [`HotReloadOptions::default`] (double-reload probe, publish gen immediately).
#[allow(dead_code)] // kept as the simple API; CLI paths use `inject_hot_reload_with`
pub fn inject_hot_reload(
    project_root: &Path,
    output_dir: &Path,
    mod_name: &str,
) -> CliResult<HotReloadInject> {
    inject_hot_reload_with(
        project_root,
        output_dir,
        mod_name,
        HotReloadOptions::default(),
    )
}

/// Like [`inject_hot_reload`] with explicit probe / publish behaviour.
pub fn inject_hot_reload_with(
    project_root: &Path,
    output_dir: &Path,
    mod_name: &str,
    options: HotReloadOptions,
) -> CliResult<HotReloadInject> {
    ensure_probe(output_dir, mod_name, options.probe)?;

    let fingerprint = source_fingerprint(project_root)?;
    let state_dir = project_root.join(".factorio-rs");
    std::fs::create_dir_all(&state_dir).map_err(|source| CliError::CreateDir {
        path: state_dir.clone(),
        source,
    })?;

    let fp_path = state_dir.join("reload_content_fp");
    let gen_path = state_dir.join("reload_gen");
    let previous_fp = std::fs::read_to_string(&fp_path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let previous_gen = std::fs::read_to_string(&gen_path)
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0);

    let (generation, bumped) =
        if previous_fp.as_deref() == Some(fingerprint.as_str()) && previous_gen > 0 {
            (previous_gen, false)
        } else {
            (previous_gen.saturating_add(1).max(1), true)
        };

    write_if_changed(&fp_path, &format!("{fingerprint}\n"))?;
    write_if_changed(&gen_path, &format!("{generation}\n"))?;

    if options.publish_gen {
        publish_reload_gen(output_dir, generation)?;
    }

    Ok(HotReloadInject { generation, bumped })
}

/// Write `lua/factorio_rs_reload_gen.lua` (build marker; reload is triggered via UDP).
///
/// Call this **after** deploy when using deferred publish, so the generation bump is
/// not visible while `dist/` is mid-rebuild or the mods entry is being replaced.
pub fn publish_reload_gen(output_dir: &Path, generation: u64) -> CliResult<()> {
    let lua_dir = output_dir.join("lua");
    std::fs::create_dir_all(&lua_dir).map_err(|source| CliError::CreateDir {
        path: lua_dir.clone(),
        source,
    })?;
    let gen_lua_path = lua_dir.join(GEN_LUA);
    let gen_body =
        format!("-- factorio-rs hot-reload generation\nreturn {{ gen = {generation} }}\n");
    write_if_changed(&gen_lua_path, &gen_body)?;
    Ok(())
}

/// Compare data/settings stage fingerprints; note when a full Factorio restart is needed.
pub fn note_stage_restart_if_needed(project_root: &Path, output_dir: &Path) -> CliResult<()> {
    let fingerprint = stage_fingerprint(output_dir)?;
    let state_path = project_root.join(".factorio-rs").join("sync_stage_fp");
    let previous = std::fs::read_to_string(&state_path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    write_if_changed(&state_path, &format!("{fingerprint}\n"))?;

    if fingerprint.is_empty() {
        return Ok(());
    }

    if previous.as_deref() != Some(fingerprint.as_str()) {
        status::status(
            Status::Note,
            "data/settings stage changed - restart Factorio to apply prototypes/settings \
             (control hot-reload cannot)",
        );
    }
    Ok(())
}

fn ensure_probe(output_dir: &Path, _mod_name: &str, mode: ReloadProbeMode) -> CliResult<()> {
    let control_path = output_dir.join("control.lua");
    let mut control =
        std::fs::read_to_string(&control_path).map_err(|source| CliError::ReadFile {
            path: control_path.clone(),
            source,
        })?;
    if let Some(idx) = control.find(PROBE_MARKER) {
        control.truncate(idx);
        while control.ends_with('\n') {
            control.pop();
        }
        control.push('\n');
    }
    control.push('\n');
    control.push_str(&generate_probe_lua(mode));
    write_if_changed(&control_path, &control)?;
    Ok(())
}

/// Hash project inputs that should trigger a Factorio control hot-reload.
fn source_fingerprint(project_root: &Path) -> CliResult<String> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    let mut paths = Vec::new();

    for name in ["Factorio.toml", "Cargo.toml"] {
        let path = project_root.join(name);
        if path.is_file() {
            paths.push(path);
        }
    }

    let source_dir = crate::config::Config::load(project_root).map_or_else(
        |_| project_root.join("src"),
        |config| project_root.join(&config.source),
    );
    if source_dir.is_dir() {
        collect_rust_sources(&source_dir, &mut paths)?;
    }
    collect_path_dep_sources(project_root, &mut paths)?;

    paths.sort();
    for path in &paths {
        let relative = path.strip_prefix(project_root).unwrap_or(path.as_path());
        relative.hash(&mut hasher);
        let bytes = std::fs::read(path).map_err(|source| CliError::ReadFile {
            path: path.clone(),
            source,
        })?;
        hasher.write(&bytes);
    }
    Ok(format!("{:016x}", hasher.finish()))
}

/// Include `path = "..."` Cargo dependencies so library edits bump reload gen.
fn collect_path_dep_sources(
    project_root: &Path,
    out: &mut Vec<std::path::PathBuf>,
) -> CliResult<()> {
    let manifest = project_root.join("Cargo.toml");
    if !manifest.is_file() {
        return Ok(());
    }
    let contents = std::fs::read_to_string(&manifest).map_err(|source| CliError::ReadFile {
        path: manifest.clone(),
        source,
    })?;
    let value: toml::Value =
        toml::from_str(&contents).map_err(|source| CliError::CargoManifestParse {
            path: manifest.clone(),
            source,
        })?;
    let Some(deps) = value.get("dependencies").and_then(toml::Value::as_table) else {
        return Ok(());
    };
    for dep in deps.values() {
        let Some(path) = dep.get("path").and_then(toml::Value::as_str) else {
            continue;
        };
        let dep_root = project_root.join(path);
        if !dep_root.join("Factorio.toml").is_file() {
            continue;
        }
        let src = dep_root.join("src");
        if src.is_dir() {
            collect_rust_sources(&src, out)?;
        }
    }
    Ok(())
}

fn collect_rust_sources(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> CliResult<()> {
    for entry in std::fs::read_dir(dir).map_err(|source| CliError::ReadDir {
        path: dir.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| CliError::ReadDir {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_rust_sources(&path, out)?;
        } else if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("rs"))
            && path
                .file_name()
                .is_some_and(|name| name != "factorio_exports.rs")
        {
            out.push(path);
        }
    }
    Ok(())
}

fn stage_fingerprint(output_dir: &Path) -> CliResult<String> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    let mut any = false;
    for name in STAGE_FILES {
        let path = output_dir.join(name);
        if !path.is_file() {
            continue;
        }
        any = true;
        name.hash(&mut hasher);
        let bytes = std::fs::read(&path).map_err(|source| CliError::ReadFile {
            path: path.clone(),
            source,
        })?;
        hasher.write(&bytes);
    }
    if any {
        Ok(format!("{:016x}", hasher.finish()))
    } else {
        Ok(String::new())
    }
}

fn generate_probe_lua(mode: ReloadProbeMode) -> String {
    let payload = RELOAD_UDP_PAYLOAD;
    let second_reload = match mode {
        ReloadProbeMode::Once => String::new(),
        ReloadProbeMode::Twice => r"
    if storage.__frs_reload_again then
      storage.__frs_reload_again = nil
      __frs_do_reload()
      return
    end
"
        .to_string(),
    };
    let arm_second = match mode {
        ReloadProbeMode::Once => String::new(),
        ReloadProbeMode::Twice => "    storage.__frs_reload_again = true\n".to_string(),
    };

    format!(
        r#"{PROBE_MARKER}
do
  -- CLI pings localhost UDP (`--enable-lua-udp`); Factorio's mod VFS does not
  -- re-read generation files from disk until reload_mods / reload_script.
  local function __frs_do_reload()
    if game.reload_mods then
      game.reload_mods()
    elseif game.reload_script then
      game.reload_script()
    end
  end
  local function __frs_on_reload_ping()
    if not game then
      return
    end
    if not game.reload_mods and not game.reload_script then
      return
    end
{arm_second}    __frs_do_reload()
  end
  script.on_event(defines.events.on_udp_packet_received, function(event)
    local payload = event.payload
    if type(payload) == "string" and payload:find("{payload}", 1, true) == 1 then
      __frs_on_reload_ping()
    end
  end)
  script.on_nth_tick(5, function()
    if not game then
      return
    end
{second_reload}    if helpers and helpers.recv_udp then
      pcall(function()
        helpers.recv_udp()
      end)
    end
  end)
end
"#
    )
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn inject_bumps_only_when_sources_change() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let out = root.join("dist");
        let src = root.join("src");
        std::fs::create_dir_all(out.join("lua")).unwrap();
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(
            root.join("Factorio.toml"),
            "source = \"src\"\noutput_dir = \"dist\"\n",
        )
        .unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        std::fs::write(src.join("lib.rs"), "fn main() {}\n").unwrap();
        std::fs::write(out.join("control.lua"), "-- base\n").unwrap();

        let first = inject_hot_reload(root, &out, "demo").unwrap();
        assert_eq!(first.generation, 1);
        assert!(first.bumped);

        let second = inject_hot_reload(root, &out, "demo").unwrap();
        assert_eq!(second.generation, 1);
        assert!(!second.bumped);

        // Rebuilding dist alone must not bump.
        std::fs::write(out.join("control.lua"), "-- rebuilt\n").unwrap();
        let third = inject_hot_reload(root, &out, "demo").unwrap();
        assert_eq!(third.generation, 1);
        assert!(!third.bumped);

        std::fs::write(src.join("lib.rs"), "fn main() { /* changed */ }\n").unwrap();
        let fourth = inject_hot_reload(root, &out, "demo").unwrap();
        assert_eq!(fourth.generation, 2);
        assert!(fourth.bumped);
    }

    #[test]
    fn deferred_publish_writes_gen_only_when_requested() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let out = root.join("dist");
        let src = root.join("src");
        std::fs::create_dir_all(out.join("lua")).unwrap();
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(
            root.join("Factorio.toml"),
            "source = \"src\"\noutput_dir = \"dist\"\n",
        )
        .unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        std::fs::write(src.join("lib.rs"), "fn main() {}\n").unwrap();
        std::fs::write(out.join("control.lua"), "-- base\n").unwrap();

        let injected = inject_hot_reload_with(
            root,
            &out,
            "demo",
            HotReloadOptions {
                probe: ReloadProbeMode::Twice,
                publish_gen: false,
            },
        )
        .unwrap();
        assert!(!out.join("lua").join(GEN_LUA).exists());

        publish_reload_gen(&out, injected.generation).unwrap();
        let body = std::fs::read_to_string(out.join("lua").join(GEN_LUA)).unwrap();
        assert!(body.contains(&format!("gen = {}", injected.generation)));

        let control = std::fs::read_to_string(out.join("control.lua")).unwrap();
        assert!(control.contains("storage.__frs_reload_again"));
    }

    #[test]
    fn once_probe_omits_second_reload_flag() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let out = root.join("dist");
        let src = root.join("src");
        std::fs::create_dir_all(out.join("lua")).unwrap();
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(
            root.join("Factorio.toml"),
            "source = \"src\"\noutput_dir = \"dist\"\n",
        )
        .unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        std::fs::write(src.join("lib.rs"), "fn main() {}\n").unwrap();
        std::fs::write(out.join("control.lua"), "-- base\n").unwrap();

        inject_hot_reload_with(
            root,
            &out,
            "demo",
            HotReloadOptions {
                probe: ReloadProbeMode::Once,
                publish_gen: true,
            },
        )
        .unwrap();

        let control = std::fs::read_to_string(out.join("control.lua")).unwrap();
        assert!(control.contains(PROBE_MARKER));
        assert!(control.contains("on_udp_packet_received"));
        assert!(control.contains(RELOAD_UDP_PAYLOAD));
        assert!(!control.contains("storage.__frs_reload_again"));
    }

    #[test]
    fn enable_lua_udp_arg_uses_port() {
        assert_eq!(enable_lua_udp_arg(34_201), "--enable-lua-udp=34201");
    }
}
