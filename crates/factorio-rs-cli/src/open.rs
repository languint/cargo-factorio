use std::process::{Command, Stdio};

use crate::{
    commands::hot_reload::{enable_lua_udp_arg, reload_udp_port},
    error::{CliError, CliResult},
    paths::{FACTORIO_STEAM_APP_ID, FactorioLaunchTarget, find_factorio},
};

/// Locate Factorio on this system and launch it with hot-reload UDP enabled.
pub fn open() -> CliResult<FactorioLaunchTarget> {
    let target = find_factorio()?;
    launch(&target)?;
    Ok(target)
}

fn launch(target: &FactorioLaunchTarget) -> CliResult<()> {
    let udp = enable_lua_udp_arg(reload_udp_port());
    let mut command = match target {
        FactorioLaunchTarget::Binary {
            path,
            steam_run: true,
        } => {
            let mut command = Command::new("steam-run");
            command.arg(path);
            command.arg(&udp);
            command
        }
        FactorioLaunchTarget::Binary {
            path,
            steam_run: false,
        } => {
            let mut command = Command::new(path);
            command.arg(&udp);
            command
        }
        FactorioLaunchTarget::Steam => {
            // Steam appends args after `//`.
            let mut command = Command::new("steam");
            command.arg(format!("steam://rungameid/{FACTORIO_STEAM_APP_ID}//{udp}"));
            command
        }
    };

    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|source| CliError::LaunchFactorio {
            target: target.display(),
            source,
        })?;

    Ok(())
}
