//! Connects to the Bluetooth GATT echo service and tests it.
//!
mod bluetooth;
use bluer::Result;
use bluer::Uuid;
use bluetooth::Bluetooth;
use colored::Colorize;
use gilrs::{Event, Gilrs};
use std::f64::consts::PI;
use std::time::Duration;
use tokio::{io::AsyncWriteExt, time::sleep};
fn trim(v: f64) -> f64 {
    if v > 1.0 {
        return 1.0;
    }
    if v < -1.0 {
        return -1.0;
    }
    return v;
}

async fn car_control(char: bluer::gatt::remote::Characteristic) -> Result<()> {
    let mut write_io = char.write_io().await?;
    println!("    Obtained write IO with MTU {} bytes", write_io.mtu());
    let mut notify_io = char.notify_io().await?;
    let mut gilrs = Gilrs::new().unwrap();
    let mut x = 0.0f64;
    let mut y = 0.0f64;

    loop {
        sleep(Duration::from_millis(30)).await;
        // Examine new events
        while let Some(Event {
            event,
            id: _,
            time: _,
        }) = gilrs.next_event()
        {
            if let gilrs::EventType::AxisChanged(axis, value, _) = event {
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
        /*/*
          while(1 == 1)
          {
            getJoystickSettings(joystick);
            int y = joystick.joy1_y1;
            int x = joystick.joy1_x1;//motor[motorB]
            int power=sqrt(y*y+x*x)*100/255;
            if(power>100)
                power=100;
            float angle=atan2(x,y);
            if(angle>0){
                motor[motorB]=power*trim(1.0-angle/PI*4.0);
                motor[motorC]=power*trim(3.0-angle/PI*4.0);
            }else{
                motor[motorC]=power*trim(1.0+angle/PI*4.0);
                motor[motorB]=power*trim(3.0+angle/PI*4.0);
              }
          }
        }
         */ */
        let power = trim((x * x + y * y).sqrt());
        let angle = f64::atan2(x, y);
        let principale = ((power * trim(3.0 - angle.abs() / PI * 4.0) * 7.0).round() as i8) as u8;
        let secondario = ((power * trim(1.0 - angle.abs() / PI * 4.0) * 7.0).round() as i8) as u8;
        //println!("{principale} {secondario}");
        //println!("{:#010b} {:#010b}",(principale&(0x0f as u8)), secondario&(0x0f as u8));
        if let Ok(read) = notify_io.try_recv(){
            if let Some(ultimo) = read.last() {
                println!("{}-{}", read.len(), ultimo);
            }
        }
        

        let to_send = if angle > 0. {
            (principale & (0x0f as u8)) * 16 | (secondario & (0x0f as u8))
        } else {
            (secondario & (0x0f as u8)) * 16 | (principale & (0x0f as u8))
        };
        if let Err(e) = write_io.write_all(&[to_send as u8]).await {
            println!("Errore {e:?}");
        }

        //println!("{:#010b}", to_send);
        //notify_io.read_exact(&mut echo_buf).await
    }
}

async fn car1(char: bluer::gatt::remote::Characteristic) {
    println!("controlling car 1");
    car_control(char).await.unwrap();
}
async fn car2(char: bluer::gatt::remote::Characteristic) {
    println!("{}", "controlling car 2".yellow());
    car_control(char).await.unwrap();
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> bluer::Result<()> {
    let mut bl = Bluetooth::new();
    bl.add_device(
        Uuid::from_u128(0x0000ffe000001000800000805f9b34fb),
        Uuid::from_u128(0x0000ffe100001000800000805f9b34fb),
        [0xA8, 0x10, 0x87, 0x67, 0x73, 0x2A].into(),
        Box::new(car1),
    );
    bl.add_device(
        Uuid::from_u128(0x0000ffe000001000800000805f9b34fb),
        Uuid::from_u128(0x0000ffe200001000800000805f9b34fb),
        [0x48, 0x87, 0x2D, 0x11, 0xA6, 0xF1].into(),
        Box::new(car2),
    );
    bl.scan().await.unwrap();
    Ok(())
}
