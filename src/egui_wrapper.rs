use btleplug::api::BDAddr;
use image;
use std::{
    f32::consts::PI,
    sync::mpsc::{Receiver, Sender},
    time::{Duration, Instant},
};

use eframe::{
    NativeOptions, Renderer,
    egui::{self, Painter, Sense},
    emath::RectTransform,
    epaint::{Color32, Pos2, Rect, RectShape, Shape, Stroke, pos2},
};
use egui::Rounding;
use uuid::Uuid;

pub enum GuiEvent {
    Motors(f32, f32),
    Connected,
    Disconnected,
}
pub enum GuiCommand {
    Connect(BDAddr, Uuid, Uuid),
}

pub fn start_gui(
    mut options: NativeOptions,
    receiver: Receiver<GuiEvent>,
    frame_receiver: Option<Receiver<Vec<u8>>>,
    send_command: Sender<GuiCommand>,
) -> eframe::Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    options.renderer = Renderer::Wgpu;
    //options.initial_window_size = Some(egui::vec2(320.0, 240.0));
    eframe::run_native(
        "Robot comand panel",
        options,
        Box::new(|_cc| Box::new(
            {
                //egui_extras::in(&cc.egui_ctx);
            MyApp::default(receiver, frame_receiver, send_command)
            }
        )),
    )
}

#[derive(Default)]
struct Car {
    sx: f32,
    dx: f32,
    connected: bool,
}
impl Car {
    fn update(&mut self, ev: GuiEvent) {
        match ev {
            GuiEvent::Motors(sx, dx) => {
                self.sx = sx;
                self.dx = dx;
            }
            GuiEvent::Connected => {
                self.connected = true;
            }
            GuiEvent::Disconnected => {
                self.connected = false;
            }
        }
    }
}

pub struct MyApp {
    receiver: Receiver<GuiEvent>,
    car: Car,
    frame_receiver: Option<Receiver<Vec<u8>>>,
    last_image: Option<egui::TextureHandle>,
    use_hc_08: bool,
    send_command: Sender<GuiCommand>,
    //robot: Option<RobotInfo>,
}

impl MyApp {
    pub fn default(
        receiver: Receiver<GuiEvent>,
        frame_receiver: Option<Receiver<Vec<u8>>>,
        send_command: Sender<GuiCommand>,
    ) -> Self {
        Self {
            receiver,
            car: Car::default(),
            frame_receiver,
            last_image: None,
            use_hc_08: false,
            send_command,
        }
    }
}

struct CarDrawer<'a> {
    painter: &'a Painter,
    to_screen: &'a RectTransform,
}
impl<'a> CarDrawer<'a> {
    fn draw_line(&self, pos: [Pos2; 2]) {
        let s = Shape::line_segment(
            [
                self.to_screen.transform_pos(pos[0]),
                self.to_screen.transform_pos(pos[1]),
            ],
            Stroke {
                width: 1.0,
                color: Color32::WHITE,
            },
        );
        self.painter.add(s);
    }
    fn draw_circle_filled(&self, p: Pos2, r: f32, color: Color32) {
        let f = self.to_screen.to();
        let r = f.width().min(f.height()) * r;
        self.painter.add(Shape::circle_filled(
            self.to_screen.transform_pos(p),
            r,
            color,
        ));
    }
    fn draw_tier(&self, p: Pos2, power: f32) {
        let s = 0.05;
        let green = (127. + 127. * power) as u8;
        self.painter.add(Shape::Rect(RectShape {
            rect: Rect {
                min: self
                    .to_screen
                    .transform_pos(pos2(p.x - s, f32::min(p.y, p.y - 0.15 * power))),
                max: self
                    .to_screen
                    .transform_pos(pos2(p.x + s, f32::max(p.y, p.y - 0.15 * power))),
            },
            rounding: Rounding::none(),
            fill: Color32::from_rgb(255 - green, green, 0),
            stroke: Stroke {
                width: 0.0,
                color: Color32::WHITE,
            },
            //StrokeKind::Inside, // doesn't matter
        }));

        self.painter.add(Shape::Rect(RectShape {
            rect: Rect {
                min: self.to_screen.transform_pos(pos2(p.x - s, p.y - 0.15)),
                max: self.to_screen.transform_pos(pos2(p.x + s, p.y + 0.15)),
            },
            rounding: Rounding::none(),
            fill: Color32::TRANSPARENT,
            stroke: Stroke {
                width: 1.0,
                color: Color32::WHITE,
            },
            //StrokeKind::Inside, // doesn't matter
        }));
    }
    fn draw_arc(&self, s: f32, e: f32, r: f32) {
        let mut prec = pos2(r * s.cos(), r * s.sin());
        for i in 1..=100 {
            let a = (e - s) / 100.0 * i as f32 + s;
            let cur = pos2(r * a.cos(), r * a.sin());
            self.draw_line([prec, cur]);
            prec = cur;
        }
    }
    fn draw_circle(&self, p: Pos2, r: f32) {
        let mut prec = pos2(p.x + r, p.y);
        for i in 1..=100 {
            let a = 2. * PI / 100.0 * i as f32;
            let cur = pos2(p.x + r * a.cos(), p.y + r * a.sin());
            self.draw_line([prec, cur]);
            prec = cur;
        }
    }
    fn draw(painter: &'a Painter, to_screen: &'a RectTransform, car: &Car) {
        let s = Self { painter, to_screen };
        s.draw_tier(pos2(0.22, 0.17), car.dx);
        s.draw_tier(pos2(-0.22, 0.17), car.sx);
        s.draw_tier(pos2(-0.22, -0.17), car.sx);
        s.draw_tier(pos2(0.22, -0.17), car.dx);
        s.draw_arc(PI / 3., PI * 2. / 3., 0.4);
        s.draw_arc(-PI / 3., -PI * 2. / 3., 0.4);
        if car.connected {
            s.draw_circle_filled(pos2(0.0, 0.0), 0.10, Color32::GREEN)
        } else {
            s.draw_circle_filled(pos2(0.0, 0.0), 0.10, Color32::RED);
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        /*let fps = 1.0/self.fps_counter.elapsed().as_secs_f32();
        println!("FPS: {}", fps);
        self.fps_counter = Instant::now();*/
        //load latest image
        let mut frame = None;
        while let Some(f2) = self.frame_receiver.as_mut().and_then(|x| x.try_recv().ok()) {
            frame = Some(f2);
        }

        ctx.request_repaint_after(Duration::from_millis(10));
        egui::CentralPanel::default().show(ctx, |ui| {
            let (_, painter) = ui.allocate_painter(
                egui::Vec2::new(ui.available_width(), ui.available_height()),
                Sense::hover(),
            );

            // convert image in something renderable
            if let Some(frame) = frame {
                if let Ok(decoded_image) = load_image_from_byte(&frame) {

                    self.last_image = Some(ui.ctx().load_texture(
                        "frame_image",
                        decoded_image,
                        Default::default(),
                    ));
                }
            }

            let mut rect = painter.clip_rect();

            //TODO LEAVAMI 
            /*if self.last_image.is_none(){
                let url = "https://upload.wikimedia.org/wikipedia/commons/thumb/4/41/Sunflower_from_Silesia2.jpg/960px-Sunflower_from_Silesia2.jpg?20091008132228";
                let image = load_image_from_url(url).unwrap();
                self.last_image = Some(ui.ctx().load_texture(
                    "frame_image",
                    image,
                    Default::default(),
                ));
                //let image = image.await.unwrap();
            }*/

            if let Some(img) = &self.last_image {
                let img_size = img.size_vec2();
                let available_size = rect.size();
                let scale = (available_size.x / img_size.x).min(available_size.y / img_size.y);
                let scaled_size = img_size * scale;
                let img_rect = Rect::from_center_size(rect.center(), scaled_size);

                painter.add(egui::Shape::image(img.id(), img_rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),  egui::Color32::WHITE));
            }


            if self.frame_receiver.is_some() {
                rect = rect
                    .split_top_bottom_at_fraction(0.8)
                    .1
                    .split_left_right_at_fraction(0.8)
                    .1;
            }

            let to_screen = RectTransform::from_to(
                Rect::from_center_size(Pos2::ZERO, rect.square_proportions()),
                rect,
            );
            while let Ok(e) = self.receiver.try_recv() {
                self.car.update(e);
            }
            CarDrawer::draw(&painter, &to_screen, &self.car);

            ui.allocate_ui_at_rect(painter.clip_rect(), |ui| {
                ui.checkbox(&mut self.use_hc_08, "Uso l'hc-08?");
                if ui.button("connetti").clicked() {
                    let cmd = if self.use_hc_08 {
                        GuiCommand::Connect(
                            BDAddr::from_str_no_delim("A8108767732A").unwrap(),
                            Uuid::from_u128(0x0000ffe000001000800000805f9b34fb),
                            Uuid::from_u128(0x0000ffe100001000800000805f9b34fb),
                        )
                    } else {
                        GuiCommand::Connect(
                            BDAddr::from_str_no_delim("01234567AA19").unwrap(),
                            Uuid::from_u128(0x0000ffe000001000800000805f9b34fb),
                            Uuid::from_u128(0x0000ffe100001000800000805f9b34fb),
                        )
                    };
                    self.send_command.send(cmd).unwrap();
                }
            });

            
        });
    }
}

/*
/// Loads image data from a URL and converts it to `egui::ColorImage`.
fn load_image_from_url(url: &str) -> Result<egui::ColorImage, Box<dyn std::error::Error>> {
    let response = reqwest::blocking::get(url)?;
    let bytes = response.bytes()?;

    let img = image::load_from_memory(&bytes)?.to_rgba8();
    let size = [img.width() as usize, img.height() as usize];

    let pixels = img
        .pixels()
        .map(|p| egui::Color32::from_rgba_premultiplied(p[0], p[1], p[2], p[3]))
        .collect();

    Ok(egui::ColorImage { size, pixels })
}*/

fn load_image_from_byte(bytes: &[u8]) -> Result<egui::ColorImage, Box<dyn std::error::Error>> {

    let img = image::load_from_memory(&bytes)?.to_rgba8();
    let size = [img.width() as usize, img.height() as usize];

    let pixels = img
        .pixels()
        .map(|p| egui::Color32::from_rgba_premultiplied(p[0], p[1], p[2], p[3]))
        .collect();

    Ok(egui::ColorImage { size, pixels })
}