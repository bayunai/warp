use anyhow::Result;
use warp_cli::artifact::ArtifactCommand;
use warp_cli::GlobalOptions;
use warpui::AppContext;

pub fn run(
    _ctx: &mut AppContext,
    _global_options: GlobalOptions,
    command: ArtifactCommand,
) -> Result<()> {
    let action = match command {
        ArtifactCommand::Upload(_) => "upload",
        ArtifactCommand::Get(_) => "get",
        ArtifactCommand::Download(_) => "download",
    };
    Err(anyhow::anyhow!(
        "Artifact {action} is disabled in OpenWarp because cloud artifact storage is removed"
    ))
}
