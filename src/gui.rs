use std::sync::mpsc::{channel, Receiver};
use std::thread;
use eframe::egui::{Context, TextureHandle};
use eframe::{egui, Frame};
use eframe::egui::load::SizedTexture;
use crate::gui::State::Choose;
use crate::{receiver, sender};

enum State { Choose, Sender, Receiver, Sending, Receiving }

impl Default for State {
    fn default() -> Self {
        Choose
    }
}

#[derive(Default)]
pub struct EframeApp {
    state: State,
    pub ip_addr: String,
    channel_r: Option<Receiver<TextureHandle>>, // for receiver mode only!
    texture: Option<TextureHandle>,
}

impl EframeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // _cc.egui_ctx.clone();
        egui_extras::install_image_loaders(&cc.egui_ctx);
        Self::default()
    }
}

impl eframe::App for EframeApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Menu", |ui| {
                    if ui.button("Reset").clicked() {
                        self.state = Choose;
                    }
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.add_space(16.0);
                egui::widgets::global_dark_light_mode_buttons(ui);
            })
        });

        match self.state {
            State::Choose => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("Wellcome!");
                    ui.horizontal(|ui| {
                        ui.label("Do you want to send or receive a screen casting?");
                        if ui.button("Send").clicked() { self.state = State::Sender; }
                        if ui.button("Receive").clicked() { self.state = State::Receiver; }
                    });
                });
            }
            State::Sender => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("Sender!");
                    ui.horizontal(|ui| {
                        ui.label("Insert Receiver's IP address: ");
                        ui.text_edit_singleline(&mut self.ip_addr)
                    });
                    if ui.button("Send").clicked() {
                        self.state = State::Sending;
                        thread::spawn(|| {
                            sender::send();
                        });
                    }
                });
            }
            State::Receiver => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("Receiver!");
                    if ui.button("Receive").clicked() {
                        self.state = State::Receiving;

                        let (s, r) = channel();
                        self.channel_r = Some(r);
                        let ctx_clone = ctx.clone();
                        thread::spawn(move || {
                            receiver::start(s, ctx_clone);
                        });
                    }
                });
                // ctx.request_repaint();
            }
            State::Sending => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("Sending!");
                });
            }
            State::Receiving => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("Receiving!");
                    if let Some(r) = &mut self.channel_r {
                        if let Ok(texture) = r.try_recv() {
                            self.texture = Some(texture);
                        }
                    }
                    if let Some(texture) = &self.texture {
                        ui.image(SizedTexture::new(texture, texture.size_vec2()));
                    }
                });
            }
        }

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label("Screen casting app developed with rust");
            })
        });
    }
}