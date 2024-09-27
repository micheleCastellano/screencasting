use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::SystemTime;
use eframe::egui::{Color32, ColorImage, Context, ImageData, Key, TextureHandle, TextureOptions, Ui};
use eframe::{egui, Frame};
use eframe::egui::load::SizedTexture;
use serde::{Deserialize, Serialize};
use crate::{receiver, sender};
use crate::util::ChannelFrame;

#[derive(Debug, Default, Serialize, Deserialize)]
enum State {
    #[default]
    Choose,
    Sender,
    Receiver,
    Sending,
    Receiving,
    Hotkey,
}
#[derive(Default, Serialize, Deserialize)]
struct Backup {
    pub ip_addr: String,
    home_button: String,
    send_button: String,
    receive_button: String,
}
impl Backup {
    fn new(
        ip_addr: String,
        home_button: String,
        send_button: String,
        receive_button: String,
    ) -> Self {
        Self {
            ip_addr,
            home_button,
            send_button,
            receive_button,
        }
    }
}
#[derive(Default)]
pub struct EframeApp {
    state: State,
    pub ip_addr: String,

    home_button: String,
    send_button: String,
    receive_button: String,

    texture_handle: Option<TextureHandle>,
    channel_r: Option<Receiver<ChannelFrame>>, // for receiver mode only!
    stop_request: Arc<Mutex<bool>>,
}


impl EframeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self {
            texture_handle: Some(cc.egui_ctx.load_texture(
                "screencasting",
                ImageData::Color(Arc::new(ColorImage::new([1920, 1080], Color32::TRANSPARENT))),
                TextureOptions::default(),
            )),
            ..Default::default()
        };

        if let Some(storage) = cc.storage {
            let backup: Backup = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();

            app.ip_addr = backup.ip_addr;
            app.home_button = backup.home_button;
            app.send_button = backup.send_button;
            app.receive_button = backup.receive_button;
        }

        app
    }
}


impl eframe::App for EframeApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        //top panel
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                //menu
                ui.menu_button("Menu", |ui| {
                    if ui.button("Home").clicked() {
                        match self.state {
                            State::Sending | State::Receiving => {
                                *self.stop_request.lock().unwrap() = true;
                            }
                            _ => ()
                        }
                        self.state = State::Choose;
                    }
                    if ui.button("Hotkey").clicked() {
                        self.state = State::Hotkey;
                    }
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.add_space(16.0);
                egui::widgets::global_dark_light_mode_buttons(ui);
            })
        });

        //central panel
        egui::CentralPanel::default().show(ctx, |ui| {

            // hotkey support
            {
                if !self.home_button.is_empty() {
                    if ctx.input(|i| i.key_pressed(Key::from_name(&self.home_button).unwrap())) {
                        self.state = State::Choose;
                    }
                }
                if !self.send_button.is_empty() {
                    if ctx.input(|i| i.key_pressed(Key::from_name(&self.send_button).unwrap())) {
                        self.state = State::Sender;
                    }
                }
                if !self.receive_button.is_empty() {
                    if ctx.input(|i| i.key_pressed(Key::from_name(&self.receive_button).unwrap())) {
                        self.state = State::Receiver;
                    }
                }
            }

            match self.state {
                State::Choose => {
                    ui.heading("Wellcome!");
                    ui.horizontal(|ui| {
                        ui.label("Do you want to send or receive a screen casting?");
                        if ui.button("Send").clicked() { self.state = State::Sender; }
                        if ui.button("Receive").clicked() { self.state = State::Receiver; }
                    });
                }
                State::Sender => {
                    ui.heading("Sender!");
                    ui.horizontal(|ui| {
                        ui.label("Insert Receiver's IP address: ");
                        ui.text_edit_singleline(&mut self.ip_addr)
                    });
                    if ui.button("Send").clicked() {
                        self.state = State::Sending;
                        let ip_addr = self.ip_addr.clone();
                        let mut mutex = self.stop_request.lock().unwrap();
                        *mutex = false;
                        drop(mutex);
                        let stop_request = self.stop_request.clone();
                        thread::spawn(|| {
                            sender::send(ip_addr, stop_request);
                        });
                    }
                }
                State::Receiver => {
                    ui.heading("Receiver!");
                    if ui.button("Receive").clicked() {
                        self.state = State::Receiving;
                        let (s, r) = channel();
                        self.channel_r = Some(r);
                        let ctx_clone = ctx.clone();
                        let mut mutex = self.stop_request.lock().unwrap();
                        *mutex = false;
                        drop(mutex);
                        let stop_request = self.stop_request.clone();
                        thread::spawn(move || {
                            receiver::start(s, ctx_clone, stop_request);
                        });
                    }
                }
                State::Sending => {
                    ui.heading("Sending!");
                }
                State::Receiving => {
                    ui.heading("Receiving!");

                    //get new frame
                    if let Some(r) = &mut self.channel_r {
                        if let Ok(channel_frame) = r.try_recv() {
                            if let Some(texture) = &mut self.texture_handle {
                                texture.set(
                                    // ColorImage::from_rgba_unmultiplied([channel_frame.w, channel_frame.h], &channel_frame.data),
                                    // ColorImage::from_rgb([channel_frame.w, channel_frame.h], &channel_frame.data),
                                    ColorImage::from_rgba_premultiplied([channel_frame.w as usize, channel_frame.h as usize], &channel_frame.data),
                                    TextureOptions::default(),
                                );
                                println!("Gui got frame {}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis());
                            }
                        }
                    }

                    //show currently frame
                    if let Some(texture) = &mut self.texture_handle {
                        ui.add(
                            egui::Image::from_texture(SizedTexture::from_handle(texture))
                                .max_height(600.0)
                                .max_width(800.0)
                                .rounding(10.0),
                        );
                    }
                }
                State::Hotkey => {
                    view_or_customize_hotkey("Home key", &mut self.home_button, ui);
                    view_or_customize_hotkey("Send key", &mut self.send_button, ui);
                    view_or_customize_hotkey("Receive key", &mut self.receive_button, ui);
                }
            }
        });

        //bottom panel
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label("Screen casting app developed with rust");
            });
        });
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let backup = Backup::new(
            self.ip_addr.clone(),
            self.home_button.clone(),
            self.send_button.clone(),
            self.receive_button.clone(),
        );

        eframe::set_value(storage, eframe::APP_KEY, &backup);
    }
}

fn view_or_customize_hotkey(key: &str, value: &mut String, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label(key);
        ui.text_edit_singleline(value);
    });
}