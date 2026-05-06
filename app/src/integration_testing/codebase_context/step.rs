use warpui::integration::TestStep;

use crate::integration_testing::step::new_step_with_default_assertions;

/// openWarp no longer maintains a codebase embedding index.
pub fn sync_current_codebase_index() -> TestStep {
    new_step_with_default_assertions("Sync current codebase index")
}
