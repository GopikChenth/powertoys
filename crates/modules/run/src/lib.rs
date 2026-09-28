pub mod calculator;

slint::include_modules!();

use std::rc::Rc;
use std::sync::Arc;
use async_trait::async_trait;
use powertoys_core::{CompositorBridge, PowerToyModule};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};
use tracing::info;

pub struct RunModule {
    enabled: bool,
    compositor: Option<Arc<dyn CompositorBridge>>,
}

impl RunModule {
    pub fn new() -> Self {
        Self {
            enabled: true,
            compositor: None,
        }
    }

    /// Search active windows and calculations based on query string
    async fn search(&self, query: &str) -> Vec<SearchResultItem> {
        let mut results = Vec::new();
        let query_lower = query.trim().to_lowercase();

        // 1. Math calculation
        if let Some(val) = calculator::evaluate_math(query) {
            results.push(SearchResultItem {
                id: SharedString::from("math-result"),
                title: SharedString::from(format!("= {}", val)),
                subtitle: SharedString::from("Calculation Result (Press Enter to Copy)"),
                category: SharedString::from("Math"),
                action_type: SharedString::from("copy"),
            });
        }

        // 2. Window Jumping (Active Hyprland or Mock Clients)
        if let Some(comp) = &self.compositor {
            if let Ok(windows) = comp.get_windows().await {
                for win in windows {
                    if query_lower.is_empty()
                        || win.title.to_lowercase().contains(&query_lower)
                        || win.class_name.to_lowercase().contains(&query_lower)
                    {
                        results.push(SearchResultItem {
                            id: SharedString::from(win.id.clone()),
                            title: SharedString::from(win.title.clone()),
                            subtitle: SharedString::from(format!("App: {} | Workspace: {}", win.class_name, win.workspace_id)),
                            category: SharedString::from("Window"),
                            action_type: SharedString::from("focus"),
                        });
                    }
                }
            }
        }

        // 3. Shell execution if starts with `>`
        if query.starts_with('>') {
            let cmd = query.trim_start_matches('>').trim();
            results.push(SearchResultItem {
                id: SharedString::from(format!("cmd:{}", cmd)),
                title: SharedString::from(format!("Run command: {}", cmd)),
                subtitle: SharedString::from("Execute in desktop terminal/environment"),
                category: SharedString::from("Command"),
                action_type: SharedString::from("exec"),
            });
        }

        results
    }
}

#[async_trait]
impl PowerToyModule for RunModule {
    fn id(&self) -> &'static str {
        "run"
    }

    fn name(&self) -> &'static str {
        "PowerToys Run"
    }

    fn description(&self) -> &'static str {
        "Instant application launcher, window switcher, and math calculator"
    }

    fn default_hotkey(&self) -> &'static str {
        "Super+Space"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    async fn init(&mut self, compositor: Arc<dyn CompositorBridge>) -> anyhow::Result<()> {
        self.compositor = Some(compositor);
        info!("PowerToys Run module initialized");
        Ok(())
    }

    async fn trigger(&mut self) -> anyhow::Result<()> {
        info!("Triggering PowerToys Run UI");

        let compositor = self.compositor.clone();

        // Initial search results before creating UI handle
        let initial_items = self.search("").await;

        let window = RunWindow::new()?;
        let model = Rc::new(VecModel::from(initial_items));
        window.set_results(ModelRc::from(model));

        // Wire up query changed callback
        let window_weak = window.as_weak();
        let compositor_for_search = compositor.clone();
        window.on_query_changed(move |query| {
            let q = query.to_string();
            let comp = compositor_for_search.clone();
            let weak = window_weak.clone();

            tokio::spawn(async move {
                let mut items = Vec::new();
                if let Some(val) = calculator::evaluate_math(&q) {
                    items.push(SearchResultItem {
                        id: SharedString::from("math-result"),
                        title: SharedString::from(format!("= {}", val)),
                        subtitle: SharedString::from("Calculation Result"),
                        category: SharedString::from("Math"),
                        action_type: SharedString::from("copy"),
                    });
                }
                if let Some(c) = comp {
                    if let Ok(windows) = c.get_windows().await {
                        let q_lower = q.to_lowercase();
                        for w in windows {
                            if q_lower.is_empty()
                                || w.title.to_lowercase().contains(&q_lower)
                                || w.class_name.to_lowercase().contains(&q_lower)
                            {
                                items.push(SearchResultItem {
                                    id: SharedString::from(w.id),
                                    title: SharedString::from(w.title),
                                    subtitle: SharedString::from(format!("App: {} | Workspace: {}", w.class_name, w.workspace_id)),
                                    category: SharedString::from("Window"),
                                    action_type: SharedString::from("focus"),
                                });
                            }
                        }
                    }
                }

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(win) = weak.upgrade() {
                        let new_model = Rc::new(VecModel::from(items));
                        win.set_results(ModelRc::from(new_model));
                        win.set_selected_index(0);
                    }
                });
            });
        });

        // Wire up activation callback
        let window_weak = window.as_weak();
        let compositor_for_action = compositor.clone();
        window.on_item_activated(move |idx| {
            if let Some(win) = window_weak.upgrade() {
                let results = win.get_results();
                if let Some(item) = results.row_data(idx as usize) {
                    info!("Selected item: {} ({})", item.title, item.action_type);
                    if item.action_type == "focus" {
                        if let Some(comp) = &compositor_for_action {
                            let c = comp.clone();
                            let id = item.id.to_string();
                            tokio::spawn(async move {
                                let _ = c.focus_window(&id).await;
                            });
                        }
                    }
                }
                let _ = win.hide();
            }
        });

        // Wire up close callback
        let window_weak = window.as_weak();
        window.on_close_requested(move || {
            if let Some(win) = window_weak.upgrade() {
                let _ = win.hide();
            }
        });

        window.run()?;
        Ok(())
    }
}
