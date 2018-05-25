use std::sync::Arc;

use ::config::Config;

#[derive(Clone)]
pub struct State {
    pub config: Arc<Config>,
}
