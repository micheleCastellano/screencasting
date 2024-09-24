use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use eframe::egui::{Color32, ColorImage, Context, ImageData, TextureHandle, TextureOptions};
use eframe::{egui, Frame};
use eframe::egui::load::SizedTexture;
use crate::gui::State::Choose;
use crate::{receiver, sender};
use crate::util::ChannelFrame;

enum State { Choose, Sender, Receiver, Sending, Receiving }

impl Default for State {
    fn default() -> Self { Choose }
}

#[derive(Default)]
pub struct EframeApp {
    state: State,
    pub ip_addr: String,
    channel_r: Option<Receiver<ChannelFrame>>, // for receiver mode only!
    texture: Option<TextureHandle>,
}

impl EframeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut rect =cc.egui_ctx.input(|i: &egui::InputState| i.screen_rect());
        rect.set_height(800.0);
        rect.set_width(1200.0);
        Self {
            texture: Some(cc.egui_ctx.load_texture(
                "screencasting",
                ImageData::Color(Arc::new(ColorImage::new([1920, 1080], Color32::TRANSPARENT))),
                TextureOptions::default(),
            )),
            ..Default::default()
        }
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
            }
            State::Sending => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("Sending!");
                });
            }
            State::Receiving => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("Receiving!");

                    //get new frame
                    if let Some(r) = &mut self.channel_r {
                        if let Ok(channel_frame) = r.try_recv() {
                            if let Some(texture) = &mut self.texture {
                                texture.set(
                                    ColorImage::from_rgba_unmultiplied(
                                        [channel_frame.w as usize, channel_frame.h as usize],
                                        &channel_frame.data),
                                    // ColorImage::from_rgba_premultiplied ([1920, 1080], &buffer),
                                    TextureOptions::default(),
                                );
                            }
                        }
                    }

                    //show currently frame
                    if let Some(texture) = &mut self.texture {
                        ui.add(
                            egui::Image::from_texture(SizedTexture::from_handle(texture))
                                .max_height(600.0)
                                .max_width(800.0)
                                .rounding(10.0),
                        );
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