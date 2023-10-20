//! Connects to the Bluetooth GATT echo service and tests it.
//!
mod bluetooth;
use bluer::Result;
use bluer::Uuid;
use bluetooth::Bluetooth;
use colored::Colorize;
use egui::RobotEvent;
use egui::start_gui;
use gilrs::{Event, Gilrs};
use std::f64::consts::PI;
use std::io;
use std::io::Write;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread::spawn;
use std::time::Duration;
use tokio::{io::AsyncWriteExt, time::sleep};
mod egui;
fn trim(v: f64) -> f64 {
    if v > 1.0 {
        return 1.0;
    }
    if v < -1.0 {
        return -1.0;
    }
    v
}

async fn car_control(char: bluer::gatt::remote::Characteristic, sender: Sender<RobotEvent>, do_notify: bool) -> Result<()> {
    let mut write_io = char.write_io().await?;
    println!("    Obtained write IO with MTU {} bytes", write_io.mtu());
    let notify_io = if do_notify { Some(char.notify_io().await?) } else { None };
    let mut gilrs = Gilrs::new().unwrap();
    let mut x = 0.0f64;
    let mut y = 0.0f64;

    loop {
        sleep(Duration::from_millis(50)).await;
        
        // Examine new events
        while let Some(Event {
            event,
            id: _,
            time: _,
        }) = gilrs.next_event()
        {
            if let gilrs::EventType::AxisChanged(axis, value, code) = event {
                //println!("{code:?}");
                match axis {
                    gilrs::Axis::LeftStickX => {
                        x = value as f64;
                    }
                    gilrs::Axis::LeftStickY => {
                        y = value as f64;
                    }
                    _ => {}
                }

            }
        }
        /*let y = (y + 1f64) / 2f64;
        let x = y * x;*/
        let power = trim((x * x + y * y).sqrt());
        let angle = f64::atan2(x, y);
        let principale = (power * trim(3.0 - angle.abs() / PI * 4.0)) as f32;
        let principale_u8 = ((principale*7.0).round() as i8) as u8;
        let secondario = (power * trim(1.0 - angle.abs() / PI * 4.0)) as f32;
        let secondario_u8 = ((secondario * 7.0).round() as i8) as u8;

        // se c'è qualcosa da leggere lo visualizziamo
        if let Some(notify_io) = &notify_io {
            if let Ok(read) = notify_io.try_recv() {
                if let Some(ultimo) = read.last() {
                    println!("{}-{}", read.len(), ultimo);
                }
            }
        }

        let to_send = if angle > 0. {
            sender.send(RobotEvent::Motors(principale, secondario)).unwrap();
            ((principale_u8 & (0x0f_u8)) * 16) | (secondario_u8 & (0x0f_u8))
        } else {
            sender.send(RobotEvent::Motors(secondario, principale)).unwrap();
            ((secondario_u8 & (0x0f_u8)) * 16) | (principale_u8 & (0x0f_u8))

        };
        write_io.write_all(&[to_send]).await?;
        write_io.flush().await?;
        //println!("{:#010b}", to_send);
    }
}

async fn car_hc08(char: bluer::gatt::remote::Characteristic, sender: Sender<RobotEvent>) -> Result<()> {
    println!("controlling car with HC-08");
    return car_control(char, sender, true).await;
}
async fn car_altra(char: bluer::gatt::remote::Characteristic, sender: Sender<RobotEvent>) -> Result<()> {
    println!("{}", "controlling car altra".yellow());
    return car_control(char, sender, false).await;
}

#[tokio::main(flavor = "current_thread")]
async fn bluetooth(sender: Sender<RobotEvent>, use_hc08: bool) -> bluer::Result<()> {
    let mut bl = Bluetooth::new(sender);

    if use_hc08 {
        bl.add_device(
            Uuid::from_u128(0x0000ffe000001000800000805f9b34fb),
            Uuid::from_u128(0x0000ffe100001000800000805f9b34fb),
            [0xA8, 0x10, 0x87, 0x67, 0x73, 0x2A].into(),
            Box::new(car_hc08),
        );
    } else {
        bl.add_device(
            Uuid::from_u128(0x0000ffe000001000800000805f9b34fb),
            Uuid::from_u128(0x0000ffe200001000800000805f9b34fb),
            [0x48, 0x87, 0x2D, 0x11, 0xA6, 0xF1].into(),
            Box::new(car_altra),
        );
    }

    bl.scan().await
}

fn main() -> bluer::Result<()> {
    print!("Scegli una macchina, inserisci a (HC-08) o b (altra): ");
    io::stdout().flush().unwrap();
    let use_hc08 = io::stdin().lines().next().unwrap_or(Ok(String::from("a"))).unwrap_or(String::from("a")) != "b";
    println!("{}", if use_hc08 { "Uso HC-08" } else { "Uso l'altra" });

    let (sender, receiver) = mpsc::channel::<RobotEvent>();
    let join = spawn(move || {
        while let Err(e) = bluetooth(sender.clone(), use_hc08) {
            println!("Error: {e}");
            std::thread::sleep(Duration::from_secs(3));
            println!("Retrying");
        }
    });
    start_gui(receiver).unwrap();

    join.join().unwrap();
    Ok(())
}
