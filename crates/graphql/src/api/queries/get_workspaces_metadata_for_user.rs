use crate::{experiment::Experiment, schema, user::DiscoverableTeamData, workspace::Workspace};

#[derive(cynic::QueryFragment, Debug)]
pub struct User {
    pub workspaces: Vec<Workspace>,
    pub experiments: Option<Vec<Experiment>>,
    pub discoverable_teams: Vec<DiscoverableTeamData>,
}
