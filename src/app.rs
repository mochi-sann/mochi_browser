#[cfg(not(target_arch = "wasm32"))]
use std::sync::mpsc;

use crate::http::HttpResponse;

#[cfg(not(target_arch = "wasm32"))]
use crate::http::fetch_url;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,

    url_input: String,
    response: Option<HttpResponse>,
    loading: bool,

    #[serde(skip)]
    #[cfg(not(target_arch = "wasm32"))]
    receiver: Option<mpsc::Receiver<Result<HttpResponse, String>>>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            url_input: String::new(),
            response: None,
            loading: false,
            #[cfg(not(target_arch = "wasm32"))]
            receiver: None,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(receiver) = &self.receiver {
            if let Ok(result) = receiver.try_recv() {
                self.loading = false;
                self.receiver = None;
                match result {
                    Ok(response) => self.response = Some(response),
                    Err(e) => {
                        self.response = Some(HttpResponse {
                            status: 0,
                            headers: vec![],
                            body: format!("Error: {}", e),
                        });
                    }
                }
            }
        }

        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("URL Fetcher");

            ui.horizontal(|ui| {
                ui.label("URL: ");
                ui.text_edit_singleline(&mut self.url_input);
                if ui.button("Fetch").clicked() && !self.loading {
                    if self.url_input.trim().is_empty() {
                        self.response = Some(HttpResponse {
                            status: 0,
                            headers: vec![],
                            body: "Error: URL cannot be empty".to_string(),
                        });
                        return;
                    }

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        self.response = None;
                        self.loading = true;
                        let url = self.url_input.clone();
                        let (sender, receiver) = mpsc::channel();
                        self.receiver = Some(receiver);

                        std::thread::spawn(move || {
                            let result = fetch_url(&url).map_err(|e| e.to_string());
                            sender.send(result).ok();
                        });
                    }

                    #[cfg(target_arch = "wasm32")]
                    {
                        let _url = self.url_input.clone();
                        drop(_url);
                        self.response = Some(HttpResponse {
                            status: 0,
                            headers: vec![],
                            body: "WASM fetching not fully implemented. Use native build for full functionality.".to_string(),
                        });
                    }
                }
            });

            if self.loading {
                ui.spinner();
            }

            if let Some(response) = &self.response {
                ui.separator();

                ui.label(format!("Status: {}", response.status));

                ui.separator();

                ui.label("Headers:");
                for (name, value) in &response.headers {
                    ui.label(format!("{}: {}", name, value));
                }

                ui.separator();

                ui.label("Body:");
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.label(&response.body);
                    });
            }

            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/main/",
                "Source code."
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
