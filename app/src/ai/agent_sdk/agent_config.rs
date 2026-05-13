//! Commands to interact with available agents via the public API.
//!
//! OpenWarp:云端 GitHub OAuth + repo 授权检查已下线。
//! `list` 命令直接调用 `fetch_and_display_agents`,不再执行
//! `check_user_repo_auth_status` GraphQL 与 OAuth 轮询。
//! `--repo` 参数仍可识别并透传到 `ai_client.list_agents`,由后端自行处理。

use crate::server::server_api::ai::AgentListItem;
use crate::server::server_api::ServerApiProvider;
use warp_cli::agent::ListAgentConfigsArgs;
use warpui::{platform::TerminationMode, AppContext, ModelContext, SingletonEntity};

const MAX_LINE_WIDTH: usize = 90;

/// Singleton model that runs async work for agent CLI commands.
struct AgentConfigRunner;

/// List all available agents.
pub fn list_agents(ctx: &mut AppContext, args: ListAgentConfigsArgs) -> anyhow::Result<()> {
    let runner = ctx.add_singleton_model(|_ctx| AgentConfigRunner);
    runner.update(ctx, |runner, ctx| runner.list(args.repo.clone(), ctx))
}

impl AgentConfigRunner {
    fn list(&self, repo: Option<String>, ctx: &mut ModelContext<Self>) -> anyhow::Result<()> {
        // OpenWarp:云端 GitHub repo 授权检查已下线,无论是否带 --repo
        // 都直接 fetch_and_display_agents,由后端处理可见性。
        self.fetch_and_display_agents(repo, ctx);
        Ok(())
    }

    fn fetch_and_display_agents(&self, repo: Option<String>, ctx: &mut ModelContext<Self>) {
        let ai_client = ServerApiProvider::handle(ctx).as_ref(ctx).get_ai_client();

        if repo.is_some() {
            println!("Fetching agent skills from the specified repository...");
        } else {
            println!("Fetching agent skills from your Warp environments...");
        }

        let list_future = async move { ai_client.list_agents(repo).await };

        ctx.spawn(list_future, |_, result, ctx| match result {
            Ok(agents) => {
                Self::print_agents_table(&agents);
                ctx.terminate_app(TerminationMode::ForceTerminate, None);
            }
            Err(err) => {
                super::report_fatal_error(err, ctx);
            }
        });
    }

    /// Print a list of agents in a card-style format.
    fn print_agents_table(agents: &[AgentListItem]) {
        if agents.is_empty() {
            println!("No agents found.");
            return;
        }

        if agents.len() == 1 {
            println!("\nAgent:");
        } else {
            println!("\nAgents ({}):", agents.len());
        }

        for agent in agents {
            println!("\n{}", agent.name);

            for variant in &agent.variants {
                let mut table = super::output::standard_table();

                // ID
                table.add_row(vec![format!("ID: {}", variant.id)]);

                // Description
                if !variant.description.is_empty() {
                    let description_cell = super::text_layout::render_labeled_wrapped_field(
                        "Description",
                        &variant.description,
                        MAX_LINE_WIDTH,
                    );
                    table.add_row(vec![description_cell]);
                }

                // Base prompt (truncated)
                if !variant.base_prompt.is_empty() {
                    let mut chars = variant.base_prompt.chars();
                    let truncated: String = chars.by_ref().take(100).collect();
                    let truncated_prompt = if chars.next().is_some() {
                        format!("{truncated}...")
                    } else {
                        truncated
                    };
                    let prompt_cell = super::text_layout::render_labeled_wrapped_field(
                        "Base Prompt",
                        &truncated_prompt,
                        MAX_LINE_WIDTH,
                    );
                    table.add_row(vec![prompt_cell]);
                }

                // Source
                table.add_row(vec![format!(
                    "Source: {}/{}",
                    variant.source.owner, variant.source.name
                )]);

                // Environments
                if !variant.environments.is_empty() {
                    let env_entries: Vec<_> = variant
                        .environments
                        .iter()
                        .map(|e| format!("{} ({})", e.name, e.uid))
                        .collect();
                    table.add_row(vec![format!("Environments: {}", env_entries.join(", "))]);
                }

                println!("{table}");
            }
        }
    }
}

impl warpui::Entity for AgentConfigRunner {
    type Event = ();
}

impl SingletonEntity for AgentConfigRunner {}
