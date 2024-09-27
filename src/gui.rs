use crate::util::ChannelFrame;
use crate::{receiver, sender};
use eframe::egui::load::SizedTexture;
use eframe::egui::{
    Color32, ColorImage, Context, ImageData, Key, TextureHandle, TextureOptions, Ui,
};
use eframe::{egui, Frame};
use scap::capturer::{Area, Point, Size};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::SystemTime;

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
    area: Arc<Mutex<Area>>,
    screen_width_max: u32,
    screen_height_max: u32,

    save_option: Arc<AtomicBool>,

    // utils to manage stream of frames
    texture_handle: Option<TextureHandle>,
    channel_r: Option<Receiver<ChannelFrame>>, // for receiver mode only!
    stop_request: Arc<AtomicBool>,
}

impl EframeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (width, height) = rdev::display_size().unwrap();
        let mut app = Self {
            texture_handle: Some(cc.egui_ctx.load_texture(
                "screencasting",
                ImageData::Color(Arc::new(ColorImage::new(
                    [1920, 1080],
                    Color32::TRANSPARENT,
                ))),
                TextureOptions::default(),
            )),
            screen_height_max:width as u32,
            screen_width_max: height as u32,
            area: Arc::new(Mutex::new(Area {
                    origin: Point { x: 0.0, y: 0.0 },
                    size: Size {
                        width: width as f64,
                        height: height as f64,
                    },
                })),
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
                        match self.state {
                            State::Sending | State::Receiving => {
                                self.stop_request.swap(true, Ordering::Relaxed);
                            }
                            _ => (),
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
                    ui.add_space(10.0);
                    ui.label("Do you want to send or receive a screencasting?");
                    ui.add_space(10.0);
                    ui.horizontal(|ui|{
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
                    selection_options(self, ui);
                    ui.add_space(10.0);
                    if ui.button("Send").clicked() {
                        self.state = State::Sending;
                        self.stop_request.swap(false, Ordering::Relaxed);
                        let ip_addr = self.ip_addr.clone();
                        let stop_request = self.stop_request.clone();
                        let area = self.area.clone();
                        thread::spawn(|| {
                            sender::start(ip_addr, stop_request,area);
                        });
                    }
                }
                State::Receiver => {
                    ui.heading("Receiver!");
                    ui.add_space(10.0);
                    if ui.button("Receive").clicked() {
                        self.stop_request.swap(false, Ordering::Relaxed);
                        let (s, r) = channel();
                        self.channel_r = Some(r);
                        let ctx_clone = ctx.clone();
                        let stop_request = self.stop_request.clone();
                        let save_option = self.save_option.clone();
                        thread::spawn(move || {
                            receiver::start(s, ctx_clone, stop_request, save_option);
                        });
                        self.state = State::Receiving;
                    }
                }
                State::Sending => {
                    ui.heading("Sending!");
                    selection_options(self, ui);
                }
                State::Receiving => {
                    ui.heading("Receiving!");
                    let mut save_option = self.save_option.load(Ordering::Relaxed);
                    ui.checkbox(&mut save_option, "Save option");
                    self.save_option.swap(save_option, Ordering::Relaxed);
                    //get new frame
                    if let Some(r) = &mut self.channel_r {
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
            self.quit_button.clone(),
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


fn selection_options(app: &mut EframeApp, ui: &mut Ui){
    let mut area_mutex = app.area.lock().unwrap();
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
            egui::DragValue::new(&mut area_mutex.origin.x)
            .speed(10)
            .range(0..=app.screen_width_max),
        );
        ui.add_space(60.0);
        ui.label("y");
        ui.add(
            egui::DragValue::new(&mut area_mutex.origin.y)
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
            egui::DragValue::new(&mut area_mutex.size.width)
                .speed(10)
                .range(0..=app.screen_width_max),
        );
        ui.add_space(30.0);
        ui.label("height");
        ui.add(
            egui::DragValue::new(&mut area_mutex.size.height)
                .speed(10)
                .range(0..=app.screen_height_max),
        );
    });
}