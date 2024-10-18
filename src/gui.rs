use crate::capturer::{Area, Frame};
use crate::util::Message;
use crate::{receiver, sender};
use device_query::{DeviceQuery, DeviceState};
use eframe::egui::load::SizedTexture;
use eframe::egui::{
    Color32, ColorImage, Context, Id, ImageData, Key, LayerId, Pos2, Rect, Sense, Stroke,
    TextureHandle, TextureOptions, Ui, UiBuilder, Vec2,
};
use eframe::{egui, emath};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::SystemTime;
use std::{mem, thread};
use scrap::Display;

// section names
const SECT_HOME: &str = "Home";
const SECT_SEND: &str = "Send";
const SECT_RECEIVE: &str = "Receive";
const SECT_HOTKEY: &str = "Hotkey";
const SECT_ANNOTATION: &str = "Annotation";
const SECT_QUIT: &str = "Quit";

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
enum State {
    #[default]
    Home,
    Sender,
    Receiver,
    Sending,
    Receiving,
    Hotkey,
    Annotation,
}
#[derive(Default, Serialize, Deserialize)]
struct Backup {
    pub ip_addr: String,
    hotkeys: HashMap<String, String>,
}
impl Backup {
    fn new(ip_addr: String, hotkeys: HashMap<String, String>) -> Self {
        Self { ip_addr, hotkeys }
    }
}
#[derive(Default)]
pub struct EframeApp {
    state: State,
    prev_state: State,
    ip_addr: String,
    local_ip_addr: String,
    alert: bool,

    // hotkeys support:
    // if you want to add a new one, you have to modify "new", "update" and
    // "hotkey support" functions. You may also need to disable backup functionality for the first
    // launch only. Sorry for the inconvenience.
    hotkeys: HashMap<String, String>,

    // selection options support
    displays: Vec<Display>,
    selected_display: u32,
    area: Area,
    screen_width_max: u32,
    screen_height_max: u32,
    sel_opt_modify: bool,
    drag_state: DragState,
    modify_by_drag: bool,

    // annotation tool support
    lines: Vec<Vec<Pos2>>,
    stroke: Stroke,

    // utils to manage stream of frames
    texture_handle: Option<TextureHandle>,
    frame_r: Option<Receiver<Frame>>, // for receiver mode only!
    msg_s: Option<Sender<Message>>,
    join_handle: Option<JoinHandle<()>>,
    save_option: bool,
}

#[derive(Default)]
struct DragState {
    start: Option<(i32, i32)>,
    end: Option<(i32, i32)>,
    is_dragging: bool,
}

impl EframeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ds = Display::all().unwrap();
        let width = ds[0].width();
        let height = ds[0].height();

        let mut app = Self {
            texture_handle: Some(cc.egui_ctx.load_texture(
                "screencasting",
                ImageData::Color(Arc::new(ColorImage::new(
                    [1920, 1080],
                    Color32::TRANSPARENT,
                ))),
                TextureOptions::default(),
            )),
            screen_width_max: width as u32,
            screen_height_max: height as u32,
            stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
            displays: ds,
            area: Area::new(0, 0, width as u32, height as u32, 0),
            local_ip_addr: local_ip_address::local_ip().unwrap().to_string(),
            ..Default::default()
        };

        if let Some(storage) = cc.storage {
            let backup: Backup = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            app.ip_addr = backup.ip_addr;
            app.hotkeys = backup.hotkeys;
        } else {
            app.hotkeys.insert(SECT_HOME.to_string(), "".to_string());
            app.hotkeys.insert(SECT_SEND.to_string(), "".to_string());
            app.hotkeys.insert(SECT_RECEIVE.to_string(), "".to_string());
            app.hotkeys
                .insert(SECT_ANNOTATION.to_string(), "".to_string());
            app.hotkeys.insert(SECT_QUIT.to_string(), "".to_string());
        }

        app
    }
    fn selection_options(&mut self, ui: &mut Ui, ctx: &Context) {
        ui.group(|ui| {
            if !self.sel_opt_modify {
                ui.disable();
            }
            egui::Grid::new("selection_option_grid")
                .min_col_width(150.0)
                .spacing(Vec2::new(15.0, 15.0))
                .max_col_width(200.0)
                .show(ui, |ui| {
                    if self.displays.len() == 2 {
                        ui.radio_value(&mut self.selected_display, 0, "Primary");
                        ui.radio_value(&mut self.selected_display, 1, "Secondary");
                        ui.end_row();

                        self.screen_width_max = self.displays[self.selected_display as usize].width() as u32;
                        self.screen_height_max = self.displays[self.selected_display as usize].height() as u32;
                    }
                    ui.label("Insert Receiver's IP address: ");
                    ui.text_edit_singleline(&mut self.ip_addr);
                    ui.end_row();
                    ui.label("Origin:");
                    ui.horizontal(|ui| {
                        ui.label("x");
                        ui.add_space(27.0);
                        ui.add_enabled(
                            !self.modify_by_drag,
                            egui::DragValue::new(&mut self.area.x)
                                .speed(10)
                                .range(0..=self.screen_width_max),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("y");
                        ui.add_space(28.0);
                        ui.add_enabled(
                            !self.modify_by_drag,
                            egui::DragValue::new(&mut self.area.y)
                                .speed(10)
                                .range(0..=self.screen_height_max),
                        );
                    });
                    ui.end_row();
                    ui.label("Dimensions:");
                    ui.horizontal(|ui| {
                        ui.label("width");
                        ui.add_enabled(
                            !self.modify_by_drag,
                            egui::DragValue::new(&mut self.area.width)
                                .speed(10)
                                .range(0..=self.screen_width_max),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("height");
                        ui.add_enabled(
                            !self.modify_by_drag,
                            egui::DragValue::new(&mut self.area.height)
                                .speed(10)
                                .range(0..=self.screen_height_max),
                        );
                    });
                    ui.end_row();
                    if ui.button("modify by drag").clicked() {
                        self.modify_by_drag = !self.modify_by_drag;
                    }
                });
            ctx.request_repaint();
        });

        self.update_drag_state();
    }
    fn update_drag_state(&mut self) {
        let device_state = DeviceState::new();
        let mouse = device_state.get_mouse();

        if self.modify_by_drag {
            if *mouse.button_pressed.get(1).expect("button not found") {
                if !self.drag_state.is_dragging {
                    self.drag_state.start = Some((mouse.coords.0, mouse.coords.1));
                    self.drag_state.is_dragging = true;
                }
            }
            // se finito drag
            else if self.drag_state.is_dragging {
                self.drag_state.end = Some((mouse.coords.0, mouse.coords.1));
                self.drag_state.is_dragging = false;
                if let (Some(coords_start), Some(coords_end)) =
                    (self.drag_state.start, self.drag_state.end)
                {
                    self.area.x = std::cmp::min(coords_start.0, coords_end.0) as u32;
                    self.area.y = std::cmp::min(coords_start.1, coords_end.1) as u32;
                    self.area.width = (coords_end.0 - coords_start.0).abs() as u32;
                    self.area.height = (coords_end.1 - coords_start.1).abs() as u32;
                }
                self.modify_by_drag = false;
            }
        }
    }
    fn hotkey_support(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            egui::Grid::new("hotkey_grid")
                .min_col_width(100.0)
                .spacing(Vec2::new(15.0, 15.0))
                .max_col_width(150.0)
                .show(ui, |ui| {
                    if let Some(value) = self.hotkeys.get_mut(&SECT_HOME.to_string()) {
                        ui.label(SECT_HOME);
                        ui.text_edit_singleline(value);
                        ui.end_row();
                    }
                    if let Some(value) = self.hotkeys.get_mut(&SECT_RECEIVE.to_string()) {
                        ui.label(SECT_RECEIVE);
                        ui.text_edit_singleline(value);
                        ui.end_row();
                    }
                    if let Some(value) = self.hotkeys.get_mut(&SECT_SEND.to_string()) {
                        ui.label(SECT_SEND);
                        ui.text_edit_singleline(value);
                        ui.end_row();
                    }
                    if let Some(value) = self.hotkeys.get_mut(&SECT_ANNOTATION.to_string()) {
                        ui.label(SECT_ANNOTATION);
                        ui.text_edit_singleline(value);
                        ui.end_row();
                    }
                    if let Some(value) = self.hotkeys.get_mut(&SECT_HOTKEY.to_string()) {
                        ui.label(SECT_HOTKEY);
                        ui.text_edit_singleline(value);
                        ui.end_row();
                    }

                    if let Some(value) = self.hotkeys.get_mut(&SECT_QUIT.to_string()) {
                        ui.label(SECT_QUIT);
                        ui.text_edit_singleline(value);
                        ui.end_row();
                    }
                });
        });
    }
    fn annotation_tool(&mut self, ctx: &Context) {
        //initialization
        let layer_id = LayerId::background();
        let ui_builder = UiBuilder::default();
        let id = Id::new("prova");
        let mut ui = Ui::new(ctx.clone(), layer_id, id, ui_builder);
        ui.add_space(30.0);

        // annotation tool core
        let (mut response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::drag());
        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.square_proportions()),
            response.rect,
        );
        let from_screen = to_screen.inverse();
        if self.lines.is_empty() {
            self.lines.push(vec![]);
        }
        let current_line = self.lines.last_mut().unwrap();
        if let Some(pointer_pos) = response.interact_pointer_pos() {
            let canvas_pos = from_screen * pointer_pos;
            if current_line.last() != Some(&canvas_pos) {
                current_line.push(canvas_pos);
                response.mark_changed();
            }
        } else if !current_line.is_empty() {
            self.lines.push(vec![]);
            response.mark_changed();
        }
        let shapes = self
            .lines
            .iter()
            .filter(|line| line.len() >= 2)
            .map(|line| {
                let points: Vec<Pos2> = line.iter().map(|p| to_screen * *p).collect();
                egui::Shape::line(points, self.stroke)
            });
        painter.extend(shapes);
    }
    fn show_alert(&mut self) {
        self.alert = true;
    }
    fn go_back(&mut self) {
        let state = mem::replace(&mut self.state, self.prev_state.clone());
        self.prev_state = state;
    }
    fn go_home(&mut self) {
        if self.check_if_streaming_is_finished() {
            self.prev_state = self.state.clone();
            self.state = State::Home;
        } else {
            self.show_alert();
        }
    }
    fn go_hotkey(&mut self) {
        if self.check_if_streaming_is_finished() {
            self.prev_state = self.state.clone();
            self.state = State::Hotkey;
        } else {
            self.show_alert();
        }
    }
    fn go_annotation(&mut self) {
        self.prev_state = self.state.clone();
        self.state = State::Annotation
    }
    fn go_send(&mut self) {
        if self.check_if_streaming_is_finished() {
            self.prev_state = self.state.clone();
            self.state = State::Sender;
        } else {
            self.show_alert();
        }
    }
    fn go_receive(&mut self) {
        if self.check_if_streaming_is_finished() {
            self.prev_state = self.state.clone();
            self.state = State::Receiver;
        } else {
            self.show_alert();
        }
    }
    fn start_sending(&mut self) {
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
    fn start_receiving(&mut self, ctx: &Context) {
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
    fn check_if_streaming_is_finished(&mut self) -> bool {
        if let Some(handle) = self.join_handle.as_mut() {
            if handle.is_finished() {
                self.join_handle.take();
            } else {
                return false;
            }
        }
        return true;
    }
    fn stop_receiving_or_sending(&mut self) {
        if let Some(s) = self.msg_s.as_mut() {
            match s.send(Message::stop_request()) {
                Ok(_) => {
                    self.msg_s.take();
                }
                Err(e) => {
                    println!("Impossible sending stop request: {e}");
                }
            }
        }
    }
}
impl eframe::App for EframeApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // hotkey support
        for (action, shortcut) in self.hotkeys.clone().iter() {
            if !shortcut.is_empty() {
                if let Some(key) = Key::from_name(&shortcut) {
                    if ctx.input(|i| i.key_pressed(key)) {
                        if action.contains(SECT_HOME) {
                            self.go_home();
                        }
                        if action.contains(SECT_SEND) {
                            self.go_send();
                        }
                        if action.contains(SECT_RECEIVE) {
                            self.go_receive();
                        }
                        if action.contains(SECT_ANNOTATION) {
                            self.go_annotation();
                        }
                        if action.contains(SECT_HOTKEY) {
                            self.go_hotkey();
                        }
                        if action.contains(SECT_QUIT) {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                }
            }
        }

        egui::Window::new("Attention")
            .open(&mut self.alert)
            .collapsible(false)
            .default_height(50.0)
            .default_width(150.0)
            .default_pos(Pos2::new(
                self.screen_width_max as f32 / 2.0,
                self.screen_height_max as f32 / 2.0,
            ))
            .show(ctx, |ui| {
                ui.label("Please, stop the streaming before change section.");
            });

        //top panel
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                //menu
                ui.menu_button("Menu", |ui| {
                    if ui.button(SECT_HOME).clicked() {
                        self.go_home();
                    }
                    if ui.button(SECT_HOTKEY).clicked() {
                        self.go_hotkey();
                    }
                    if ui.button(SECT_ANNOTATION).clicked() {
                        self.go_annotation();
                    }
                    if ui.button(SECT_QUIT).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.add_space(10.0);
                egui::widgets::global_theme_preference_switch(ui);
                ui.add_space(10.0);
                if let State::Annotation = self.state {
                    //control commands
                    ui.label("Stroke:");
                    ui.add(&mut self.stroke);
                    ui.separator();
                    if ui.button("Clear Painting").clicked() {
                        self.lines.clear();
                    }
                    if ui.button("Back").clicked() {
                        self.go_back();
                    }
                }
            })
        });

        if let State::Annotation = self.state {
            // Annotation tool does not work with CentralPanel
            self.annotation_tool(ctx);
        } else {
            //central panel
            egui::CentralPanel::default().show(ctx, |ui| {
                // gui state management
                match self.state {
                    State::Home => {
                        ui.heading("Wellcome!");
                        ui.add_space(10.0);
                        ui.label("Do you want to send or receive a screencasting?");
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            if ui.button("Send").clicked() {
                                self.go_send();
                            }
                            if ui.button("Receive").clicked() {
                                self.go_receive();
                            }
                        });
                    }
                    State::Sender => {
                        self.sel_opt_modify = true;

                        ui.heading("Sender!");
                        ui.add_space(10.0);
                        self.selection_options(ui, ctx);
                        ui.add_space(10.0);
                        if ui.button("Start").clicked() {
                            self.start_sending();
                        }
                    }
                    State::Receiver => {
                        ui.heading("Receiver!");
                        ui.add_space(10.0);
                        ui.checkbox(&mut self.save_option, "Save streaming")
                            .on_hover_text("If checked, the stream will be saved.");
                        ui.add_space(10.0);
                        if ui.button("Start").clicked() {
                            self.start_receiving(ctx);
                        }
                    }
                    State::Sending => {
                        ui.heading("Sending!");
                        self.selection_options(ui, ctx);
                        if self.sel_opt_modify {
                            if ui.button("Apply").clicked() {
                                if let Some(s) = self.msg_s.as_mut() {
                                    if let Ok(_) = s.send(Message::area_request(self.area.clone()))
                                    {
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
                            self.stop_receiving_or_sending();
                        }
                        if self.check_if_streaming_is_finished() {
                            self.go_home();
                        }
                    }
                    State::Receiving => {
                        ui.heading(format!("Receiving on {}!", self.local_ip_addr));
                        ui.add_space(10.0);
                        let checkbox = ui
                            .checkbox(&mut self.save_option, "Save streaming")
                            .on_hover_text("If checked, the stream will be saved.");
                        if checkbox.clicked() {
                            if let Some(s) = self.msg_s.as_mut() {
                                if let Err(e) = s.send(Message::save_request(self.save_option)) {
                                    println!("Impossible sending save_request: {e}");
                                }
                            }
                        }
                        if ui.button("Stop").clicked() {
                            self.stop_receiving_or_sending();
                        }
                        if self.check_if_streaming_is_finished() {
                            self.go_home();
                        }
                        //get new frame if available
                        if let Some(r) = &mut self.frame_r {
                            if let Ok(frame) = r.try_recv() {
                                if let Some(texture) = &mut self.texture_handle {
                                    texture.set(
                                        ColorImage::from_rgb(
                                            [frame.w as usize, frame.h as usize],
                                            &frame.data,
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
                        self.hotkey_support(ui);
                    }
                    State::Annotation => {
                        panic!("Annotation tool does not work with CentralPanel");
                    }
                }
            });
        }

        //bottom panel
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label("Screen casting app developed with rust");
            });
        });
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let backup = Backup::new(self.ip_addr.clone(), self.hotkeys.clone());

        eframe::set_value(storage, eframe::APP_KEY, &backup);
    }
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // NOTE: a bright gray makes the shadows of the windows look weird.
        // We use a bit of transparency so that if the user switches on the
        // `transparent()` option they get immediate results.
        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 0).to_normalized_gamma_f32()

        // _visuals.window_fill() would also be a natural choice
    }
}
