use gilrs::{Event, Gilrs};
use tokio::{spawn, time::sleep};
use std::{f32::consts::PI, sync::mpsc, time::Duration};

fn trim(v: f32) -> f32 {
    if v > 1.0 {
        return 1.0;
    }
    if v < -1.0 {
        return -1.0;
    }
    v
}

pub fn convert_joistick(x: f32, y: f32)->(f32, f32){
    let power = trim((x * x + y * y).sqrt());
        let angle = f32::atan2(x, y);
        let principale = (power * trim(3.0 - angle.abs() / PI * 4.0)) as f32;
        let secondario = (power * trim(1.0 - angle.abs() / PI * 4.0)) as f32;
        //let secondario_u8 = ((secondario * 7.0).round() as i8) as u8;
        if angle > 0. {
            (principale, secondario)
        } else {
            (secondario, principale)
        }
}
pub fn convert_car(x:f32, y:f32)->(u8, u8){
    let principale = trim(x);
        let principale_u8 = ((principale*7.0).round() as i8) as u8;
        let secondario = trim(y);
        let secondario_u8 = ((secondario * 7.0).round() as i8) as u8;
        (principale_u8, secondario_u8)
}




pub async fn get_input_from_joistick()-> mpsc::Receiver<(f32, f32)> {
    let (sender, reciver) = mpsc::channel::<(f32, f32)>();
    spawn(async move{
        let mut gilrs = Gilrs::new().unwrap();
        let mut x=0.0;
        let mut y=0.0;
        loop {
            sleep(Duration::from_millis(50)).await;
            
            // Examine new events
            while let Some(Event {
                event,
                ..
            }) = gilrs.next_event()
            {
                if let gilrs::EventType::AxisChanged(axis, value, _) = event {
                    match axis {
                        gilrs::Axis::LeftStickX => {
                            x = value as f32;
                        }
                        gilrs::Axis::LeftStickY => {
                            y = value as f32;
                        }
                        _ => {}
                    }
                    let _ = sender.send((x, y));
    
                }
            }
        }
    });

    reciver
}