use crate::util::{ChannelFrame, Message};
use crate::{receiver, sender};
use eframe::egui::load::SizedTexture;
use eframe::egui::{Color32, ColorImage, Context, ImageData, Key, TextureHandle, TextureOptions, Ui};
use eframe::{egui, Frame};
use scap::capturer::{Area, Point, Size};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc};
use std::thread;
use std::thread::JoinHandle;
use std::time::SystemTime;

#[derive(Debug, Default, Serialize, Deserialize)]
enum State {
    #[default]
    Home,
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
    quit_button: String,
}
impl Backup {
    fn new(
        ip_addr: String,
        home_button: String,
        send_button: String,
        receive_button: String,
        quit_button: String,
    ) -> Self {
        Self {
            ip_addr,
            home_button,
            send_button,
            receive_button,
            quit_button,
        }
    }
}
#[derive(Default)]
pub struct EframeApp {
    state: State,
    ip_addr: String,

    // hotkeys: if you add a new one, remember to add it in backup struct for persistence purpose
    home_button: String,
    send_button: String,
    receive_button: String,
    quit_button: String,

    //selection options support
    area: Area,
    screen_width_max: u32,
    screen_height_max: u32,
    sel_opt_modify: bool,

    // utils to manage stream of frames
    texture_handle: Option<TextureHandle>,
    frame_r: Option<Receiver<ChannelFrame>>, // for receiver mode only!
    msg_s: Option<Sender<Message>>,
    join_handle: Option<JoinHandle<()>>,
    save_option: bool,
}

impl EframeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (width, height) = rdev::display_size().unwrap();

        let mut app = Self {
            texture_handle: Some(cc.egui_ctx.load_texture("screencasting", ImageData::Color(Arc::new(ColorImage::new([1920, 1080], Color32::TRANSPARENT))), TextureOptions::default())),
            screen_width_max: width as u32,
            screen_height_max: height as u32,
            area: Area {
                origin: Point { x: 0.0, y: 0.0 },
                size: Size {
                    width: width as f64,
                    height: height as f64,
                },
            },
            sel_opt_modify: true,
            ..Default::default()
        };

        if let Some(storage) = cc.storage {
            let backup: Backup = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();

            app.ip_addr = backup.ip_addr;
            app.home_button = backup.home_button;
            app.send_button = backup.send_button;
            app.receive_button = backup.receive_button;
            app.quit_button = backup.quit_button;
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
                        go_home(self);
                    }
                    if ui.button("Hotkey").clicked() {
                        go_hotkey(self)
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
                        self.state = State::Home;
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
                State::Home => {
                    ui.heading("Wellcome!");
                    ui.add_space(10.0);
                    ui.label("Do you want to send or receive a screencasting?");
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Send").clicked() {
                            self.state = State::Sender;
                        }
                        if ui.button("Receive").clicked() {
                            self.state = State::Receiver;
                        }
                    });
                }
                State::Sender => {
                    ui.heading("Sender!");
                    ui.add_space(10.0);
                    selection_options(self, ui);
                    ui.add_space(10.0);
                    if ui.button("Start").clicked() {
                        let ip_addr = self.ip_addr.clone();
                        let area = self.area.clone();
                        let (s, r) = channel();
                        self.msg_s = Some(s);
                        let handle = thread::spawn(|| {
                            sender::start(ip_addr, area, r);
                        });
                        self.join_handle = Some(handle);
                        self.sel_opt_modify = false;
                        self.state = State::Sending;
                    }
                }
                State::Receiver => {
                    ui.heading("Receiver!");
                    ui.add_space(10.0);
                    ui.checkbox(&mut self.save_option, "Save streaming")
                        .on_hover_text("If checked, the stream will be saved.");
                    ui.add_space(10.0);
                    if ui.button("Start").clicked() {
                        let (msg_s, msg_r) = channel();
                        let (frame_s, frame_r) = channel();
                        self.frame_r = Some(frame_r);
                        self.msg_s = Some(msg_s);
                        let ctx_clone = ctx.clone();
                        let save_option = self.save_option;
                        let handle = thread::spawn(move || {
                            receiver::start(frame_s, msg_r, ctx_clone, save_option);
                        });
                        self.join_handle = Some(handle);
                        self.state = State::Receiving;
                    }
                }
                State::Sending => {
                    ui.heading("Sending!");
                    selection_options(self, ui);
                    if self.sel_opt_modify {
                        if ui.button("Apply").clicked() {
                            if let Some(s) = self.msg_s.as_mut() {
                                if let Ok(_) = s.send(Message::area_request(self.area.clone())) {
                                    self.sel_opt_modify = false;
                                } else {
                                    println!("impossible sending area request")
                                }
                            }
                        }
                    } else {
                        if ui.button("Modify").clicked() {
                            self.sel_opt_modify = true;
                        }
                    }
                    if ui.button("Stop").clicked() {
                        stop_receiving_or_sending(self);
                    }
                    check_if_streaming_is_finished(self);
                }
                State::Receiving => {
                    ui.heading("Receiving!");
                    ui.add_space(10.0);
                    let checkbox = ui.checkbox(&mut self.save_option, "Save streaming")
                        .on_hover_text("If checked, the stream will be saved.");
                    if checkbox.clicked() {
                        if let Some(s) = self.msg_s.as_mut() {
                            if let Err(e) = s.send(Message::save_request(self.save_option)) {
                                println!("Impossible sending save_request: {e}");
                            }
                        }
                    }
                    if ui.button("Stop").clicked() {
                        stop_receiving_or_sending(self);
                    }

                    //get new frame
                    if let Some(r) = &mut self.frame_r {
                        if let Ok(channel_frame) = r.try_recv() {
                            if let Some(texture) = &mut self.texture_handle {
                                texture.set(
                                    // ColorImage::from_rgba_unmultiplied([channel_frame.w, channel_frame.h], &channel_frame.data),
                                    // ColorImage::from_rgb([channel_frame.w, channel_frame.h], &channel_frame.data),
                                    ColorImage::from_rgba_premultiplied(
                                        [channel_frame.w as usize, channel_frame.h as usize],
                                        &channel_frame.data,
                                    ),
                                    TextureOptions::default(),
                                );
                                println!(
                                    "Gui got frame {}",
                                    SystemTime::now()
                                        .duration_since(SystemTime::UNIX_EPOCH)
                                        .unwrap()
                                        .as_millis()
                                );
                            }
                        }
                    }

                    //show currently frame
                    if let Some(texture) = &mut self.texture_handle {
                        ui.add(egui::Image::from_texture(SizedTexture::from_handle(texture))
                                   .max_height(600.0)
                                   .max_width(800.0)
                                   .rounding(10.0), );
                    }
                }
                State::Hotkey => {
                    view_or_customize_hotkey("Home key", &mut self.home_button, ui);
                    view_or_customize_hotkey("Send key", &mut self.send_button, ui);
                    view_or_customize_hotkey("Receive key", &mut self.receive_button, ui);
                    view_or_customize_hotkey("Quit key", &mut self.quit_button, ui);
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
            self.quit_button.clone(),
        );

        eframe::set_value(storage, eframe::APP_KEY, &backup);
    }
}

// util functions
fn check_if_streaming_is_finished(app: &mut EframeApp) {
    if let Some(handle) = app.join_handle.as_mut() {
        if handle.is_finished() {
            app.join_handle.take();
            app.state = State::Home;
        }
    }
}

// function for gui building
fn view_or_customize_hotkey(key: &str, value: &mut String, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label(key);
        ui.text_edit_singleline(value);
    });
}
fn selection_options(app: &mut EframeApp, ui: &mut Ui) {
    ui.group(|ui| {
        if !app.sel_opt_modify {
            ui.disable();
        }

        ui.horizontal(|ui| {
            ui.label("Insert Receiver's IP address: ");
            ui.text_edit_singleline(&mut app.ip_addr)
        });
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label("Origin:");
            ui.add_space(155.0);
            ui.label("x");
            ui.add(
                egui::DragValue::new(&mut app.area.origin.x)
                    .speed(10)
                    .range(0..=app.screen_width_max),
            );
            ui.add_space(60.0);
            ui.label("y");
            ui.add(
                egui::DragValue::new(&mut app.area.origin.y)
                    .speed(10)
                    .range(0..=app.screen_height_max),
            );
        });
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label("Dimensions:");
            ui.add_space(100.0);
            ui.label("width");
            ui.add(
                egui::DragValue::new(&mut app.area.size.width)
                    .speed(10)
                    .range(0..=app.screen_width_max),
            );
            ui.add_space(30.0);
            ui.label("height");
            ui.add(
                egui::DragValue::new(&mut app.area.size.height)
                    .speed(10)
                    .range(0..=app.screen_height_max),
            );
        });
    });
}

// functions called for user interaction
fn stop_receiving_or_sending(app: &mut EframeApp) {
    if let Some(s) = app.msg_s.as_mut() {
        match s.send(Message::stop_request()) {
            Ok(_) => { app.msg_s.take(); }
            Err(e) => { println!("Impossible sending stop request: {e}"); }
        }
    }
}
fn go_home(app: &mut EframeApp) {
    match app.state {
        State::Sending | State::Receiving => {
            stop_receiving_or_sending(app);
        }
        _ => (),
    }
    app.state = State::Home;
}
fn go_hotkey(app: &mut EframeApp) {
    app.state = State::Hotkey;
}