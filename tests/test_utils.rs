use std::sync::Once;

static INIT: Once = Once::new();

pub fn init_test_env() {
    INIT.call_once(|| {
        // Disable TUI for tests
        std::env::set_var("LAZY_HISTORY_NO_TUI", "1");
    });
}
