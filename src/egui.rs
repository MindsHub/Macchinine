use std::{f32::consts::PI, sync::mpsc::Receiver, time::Duration};

use eframe::{
    egui::{self, Sense, Painter},
    emath::RectTransform,
    epaint::{Color32, Pos2, Rect, RectShape, Rounding, Stroke, Shape, pos2},
};


pub enum RobotEvent{
    Motors(f32, f32)
}

pub fn start_gui(receiver: Receiver<RobotEvent>) -> Result<(), eframe::Error> {
    // Log to stdout (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Robot comand panel",
        options,
        Box::new(|_cc| Box::new(MyApp::default(receiver))),
    )
}

#[derive(Default)]
struct Car{
    sx: f32,
    dx: f32,
}
impl Car{
    fn update(&mut self, ev: RobotEvent){
        match ev{
            RobotEvent::Motors(sx, dx) => {
                println!("event{sx} {dx}");
                self.sx=sx;
                self.dx=dx;
            },
        }
    }
}

struct MyApp {
    receiver: Receiver<RobotEvent>,
    car: Car,
    //robot: Option<RobotInfo>,
}

impl MyApp {
    fn default(receiver: Receiver<RobotEvent>) -> Self {
        Self { receiver, car: Car::default()}
    }
}

struct CarDrawer<'a>{
    painter: &'a Painter,
    to_screen: &'a RectTransform,

}
impl<'a> CarDrawer<'a>{
    fn draw_line(&self, pos: [Pos2; 2]){
        let s=Shape::line_segment(
            [self.to_screen.transform_pos(pos[0]),
            self.to_screen.transform_pos(pos[1])], Stroke{width: 1.0, color: Color32::WHITE});
        self.painter.add(s);
    }
    fn draw_tier(&self, p: Pos2, power: f32){
        let s= 0.05;
        let green = (127.+127.*power) as u8;
        self.painter.add(Shape::Rect(RectShape { 
            rect: Rect{
                min: self.to_screen.transform_pos(pos2(p.x-s, f32::min(p.y, p.y-0.15*power))),
                max: self.to_screen.transform_pos(pos2(p.x+s, f32::max(p.y, p.y-0.15*power)))
            },
            rounding: Rounding::none(),
            fill: Color32::from_rgb(255-green, green , 0),
            stroke: Stroke{width: 0.0, color: Color32::WHITE} }));

        self.painter.add(Shape::Rect(RectShape { 
            rect: Rect{
                min: self.to_screen.transform_pos(pos2(p.x-s, p.y-0.15)),
                max: self.to_screen.transform_pos(pos2(p.x+s, p.y+0.15))
            },
            rounding: Rounding::none(),
            fill: Color32::TRANSPARENT,
            stroke: Stroke{width: 1.0, color: Color32::WHITE} }));
    }
    fn draw_arc(&self, s: f32, e: f32, r: f32){
        let mut prec= pos2(r*s.cos(), r*s.sin());
        for i in 1..=100{
            let a =(e-s)/100.0*i as f32+s;
            let cur = pos2(r*a.cos(), r*a.sin());
            self.draw_line([prec, cur]);
            prec=cur; 
        }
    }
    fn draw(painter: &'a Painter, to_screen: &'a RectTransform, car: &Car){
        let s=Self{painter, to_screen};
        s.draw_tier(pos2(0.22, 0.17), car.dx);
        s.draw_tier(pos2(-0.22, 0.17), car.sx);
        s.draw_tier(pos2(-0.22, -0.17), car.sx);
        s.draw_tier(pos2(0.22, -0.17), car.dx);
        s.draw_arc(PI/3., PI*2./3., 0.4);
        s.draw_arc(-PI/3., -PI*2./3., 0.4);
    }
}


impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(30));
        egui::CentralPanel::default().show(ctx, |ui| {
            let (_, painter) =
                ui.allocate_painter(egui::Vec2::new(ui.available_width(), ui.available_height()), Sense::hover());

            let rect = painter.clip_rect();
            let to_screen = RectTransform::from_to(
                Rect::from_center_size(Pos2::ZERO, rect.square_proportions()),
                rect,
            );
            while let Ok(e) = self.receiver.try_recv(){
                self.car.update(e);
            }
            CarDrawer::draw(&painter, &to_screen, &self.car);
                
            
        });
    }
}