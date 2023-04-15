//! Connects to the Bluetooth GATT echo service and tests it.
//! 
mod bluetooth;
 use bluer::Result;
 use bluer::Uuid;
 use colored::Colorize;
use bluetooth::Bluetooth;
use rand::Rng;
use std::time::Duration;
use tokio::{
    io::AsyncWriteExt,
    time::sleep,
};
use gilrs::{Gilrs, Event};
/*
const SERVICE_UUID: bluer::Uuid = Uuid::from_u128(0x00000000000000000099aabbccddeeff);
const CHARACTERISTIC_UUID: bluer::Uuid = Uuid::from_u128(0x0000110c00001000800000805f9b34fb);
async fn find_our_characteristic(device: &Device) -> Result<Option<Characteristic>> {
    let addr = device.address();
    let uuids = device.uuids().await?.unwrap_or_default();
    //let k = uuids.get(0).unwrap().as_bytes();
    println!("Discovered device {} with service UUIDs: {:?}", addr, uuids);
    
    let md = device.manufacturer_data().await?;
    println!("    Manufacturer data: {:x?}", &md);

    if uuids.contains(&SERVICE_UUID) {
        println!("    Device provides our service!");
        if !device.is_connected().await? {
            println!("    Connecting...");
            let mut retries = 2;
            loop {
                match device.connect().await {
                    Ok(()) => break,
                    Err(err) if retries > 0 => {
                        println!("    Connect error: {}", &err);
                        retries -= 1;
                    }
                    Err(err) => return Err(err),
                }
            }
            println!("    Connected");
        } else {
            println!("    Already connected");
        }

        println!("    Enumerating services...");
        for service in device.services().await? {
            let uuid = service.uuid().await?;
            println!("    Service UUID: {}", &uuid);
            if uuid == SERVICE_UUID {
                println!("    Found our service!");
                for char in service.characteristics().await? {
                    let uuid = char.uuid().await?;
                    println!("    Characteristic UUID: {}", &uuid);
                    if uuid == CHARACTERISTIC_UUID {
                        println!("    Found our characteristic!");
                        return Ok(Some(char));
                    }
                }
            }
        }

        println!("    Not found!");
    }

    Ok(None)
}

async fn exercise_characteristic(char: &Characteristic) -> Result<()> {
    let mut write_io = char.write_io().await?;
    println!("    Obtained write IO with MTU {} bytes", write_io.mtu());
    let mut gilrs = Gilrs::new().unwrap();
    //let mut notify_io = char.notify_io().await?;
    //println!("    Obtained notification IO with MTU {} bytes", notify_io.mtu());

    // Flush notify buffer.
    //let mut buf = [0; 1024];
    //while let Ok(Ok(_)) = timeout(Duration::from_secs(1), notify_io.read(&mut buf)).await {}
    let mut steering= 0.0f64;
    let mut forward= 0.0f64;
    let mut rng = rand::thread_rng();
    loop {
        sleep(Duration::from_millis(30)).await;
        // Examine new events
        while let Some(Event {event, id: _, time: _}) = gilrs.next_event() {
            match event{
                gilrs::EventType::AxisChanged(axis, value, _) => {
                    match axis{
                        gilrs::Axis::LeftStickX => {
                                steering = value as f64;
                        },
                        gilrs::Axis::LeftStickY => {
                            forward = value as f64;
                    },
                        _ => {}
                    }
                },
                _ => {},
            }
        }
        let diag = forward*forward+steering*steering;
        let diag = diag.sqrt();
        println!("{steering:.3}, {forward:.3}, {diag:.3}");
        if diag < 0.2{
            let data: Vec<u8> = "s".as_bytes().into_iter().map(|x| x.clone()).collect();
            write_io.write_all(&data).await.expect("write failed");
            continue;
        }
        if forward>0. && rng.gen_bool(forward/diag as f64){
            let data: Vec<u8> = "f".as_bytes().into_iter().map(|x| x.clone()).collect();
            write_io.write_all(&data).await.expect("write failed");
            continue;
            //send = send.add("r");
        }
        if forward<0. && rng.gen_bool(-forward/diag as f64){
            let data: Vec<u8> = "b".as_bytes().into_iter().map(|x| x.clone()).collect();
            write_io.write_all(&data).await.expect("write failed");
            continue;
        }
        if steering>0. && rng.gen_bool(steering/diag as f64){
            let data: Vec<u8> = "r".as_bytes().into_iter().map(|x| x.clone()).collect();
            write_io.write_all(&data).await.expect("write failed");
            continue;
            //send = send.add("r");
        }
        if steering<0. && rng.gen_bool(-steering/diag as f64){
            let data: Vec<u8> = "l".as_bytes().into_iter().map(|x| x.clone()).collect();
            write_io.write_all(&data).await.expect("write failed");
            continue;
        }
        let data: Vec<u8> = "s".as_bytes().into_iter().map(|x| x.clone()).collect();
        write_io.write_all(&data).await.expect("write failed");
        
        //println!("mandato f {steering}");

    }
    Ok(())
}


#[tokio::main]
async fn main() -> bluer::Result<()> {
    /*let mut bl = Bluetooth::new();
    bl.add_device(
        Uuid::from_u128(0x0000ffe000001000800000805f9b34fb),
        Uuid::from_u128(0x0000ffe100001000800000805f9b34fb),
        [0xA8, 0x10, 0x87, 0x67, 0x73, 0x2A].into(), Box::new(t));
    bl.add_device(
        Uuid::from_u128(0x0000ffe000001000800000805f9b34fb),
        Uuid::from_u128(0x0000ffe200001000800000805f9b34fb),
        [0xA8, 0x10, 0x87, 0x67, 0x73, 0x2A].into(), Box::new(y));
        
    bl.add_device(
        Uuid::from_u128(0x0000110c00001000800000805f9b34fb),
        Uuid::from_u128(0x0000110c00001000800000805f9b34fb),
        [0xA8, 0x10, 0x87, 0x67, 0x73, 0x2A].into(), Box::new(y));    
    bl.scan().await.unwrap();*/
    println!("{}", SERVICE_UUID);
    println!("{}", CHARACTERISTIC_UUID);
    env_logger::init();
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    {
        println!(
            "Discovering on Bluetooth adapter {} with address {}\n",
            adapter.name(),
            adapter.address().await?
        );
        let discover = adapter.discover_devices().await?;
        pin_mut!(discover);
        let mut done = false;
        while let Some(evt) = discover.next().await {
            match evt {
                AdapterEvent::DeviceAdded(addr) => {
                    let device = adapter.device(addr)?;
                    match find_our_characteristic(&device).await {
                        Ok(Some(char)) => match exercise_characteristic(&char).await {
                            Ok(()) => {
                                println!("    Characteristic exercise completed");
                                done = true;
                            }
                            Err(err) => {
                                println!("    Characteristic exercise failed: {}", &err);
                            }
                        },
                        Ok(None) => (),
                        Err(err) => {
                            println!("    Device failed: {}", &err);
                            let _ = adapter.remove_device(device.address()).await;
                        }
                    }
                    match device.disconnect().await {
                        Ok(()) => println!("    Device disconnected"),
                        Err(err) => println!("    Device disconnection failed: {}", &err),
                    }
                    println!();
                }
                AdapterEvent::DeviceRemoved(addr) => {
                    println!("Device removed {addr}");
                }
                _ => (),
            }
            if done {
                break;
            }
        }
        println!("Stopping discovery");
    }

    sleep(Duration::from_secs(1)).await;
    Ok(())
}*/
async fn old_car_control(char: bluer::gatt::remote::Characteristic)->Result<()>{
    let mut write_io = char.write_io().await?;
    println!("    Obtained write IO with MTU {} bytes", write_io.mtu());
    let mut gilrs = Gilrs::new().unwrap();
    let mut steering= 0.0f64;
    let mut forward= 0.0f64;
    let mut rng = rand::thread_rng();
    loop {
        sleep(Duration::from_millis(30)).await;
        // Examine new events
        while let Some(Event {event, id: _, time: _}) = gilrs.next_event() {
            
            if let gilrs::EventType::AxisChanged(axis, value, _) = event{
                match axis{
                    gilrs::Axis::LeftStickX => {
                            steering = value as f64;
                    },
                    gilrs::Axis::LeftStickY => {
                        forward = value as f64;
                    },
                    _ => {}
                }
            }
        }
        let diag = forward*forward+steering*steering;
        let diag = diag.sqrt();
        println!("{steering:.3}, {forward:.3}, {diag:.3}");
        if diag < 0.2{
            let data: Vec<u8> = "s".as_bytes().to_vec();
            write_io.write_all(&data).await.expect("write failed");
            continue;
        }
        if forward>0. && rng.gen_bool(forward/diag){
            let data: Vec<u8> = "f".as_bytes().to_vec();
            write_io.write_all(&data).await.expect("write failed");
            continue;
            //send = send.add("r");
        }
        if forward<0. && rng.gen_bool(-forward/diag){
            let data: Vec<u8> = "b".as_bytes().to_vec();
            write_io.write_all(&data).await.expect("write failed");
            continue;
        }
        if steering>0. && rng.gen_bool(steering/diag){
            let data: Vec<u8> = "r".as_bytes().to_vec();
            write_io.write_all(&data).await.expect("write failed");
            continue;
            //send = send.add("r");
        }
        if steering<0. && rng.gen_bool(-steering/diag){
            let data: Vec<u8> = "l".as_bytes().to_vec();
            write_io.write_all(&data).await.expect("write failed");
            continue;
        }
        let data: Vec<u8> = "s".as_bytes().to_vec();
        write_io.write_all(&data).await.expect("write failed");
        
        //println!("mandato f {steering}");

    }
    //Ok(())
}

async fn car_control(char: bluer::gatt::remote::Characteristic)->Result<()>{
    /*#include "JoystickDriver.c"
#pragma DebuggerWindows("joystickSimple")
float trim(float x){
	if(x>1.0)
		return 1.0;
	if(x<-1.0)
		return -1.0;
	return x;
}
task main()
{
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
 */

    let mut write_io = char.write_io().await?;
    println!("    Obtained write IO with MTU {} bytes", write_io.mtu());
    let mut gilrs = Gilrs::new().unwrap();
    let mut steering= 0.0f64;
    let mut forward= 0.0f64;
    let mut rng = rand::thread_rng();
    loop {
        sleep(Duration::from_millis(30)).await;
        // Examine new events
        while let Some(Event {event, id: _, time: _}) = gilrs.next_event() {
            
            if let gilrs::EventType::AxisChanged(axis, value, _) = event{
                match axis{
                    gilrs::Axis::LeftStickX => {
                            steering = value as f64;
                    },
                    gilrs::Axis::LeftStickY => {
                        forward = value as f64;
                    },
                    _ => {}
                }
            }
        }
        let diag = forward*forward+steering*steering;
        let diag = diag.sqrt();
        println!("{steering:.3}, {forward:.3}, {diag:.3}");
        if diag < 0.2{
            let data: Vec<u8> = "s".as_bytes().to_vec();
            write_io.write_all(&data).await.expect("write failed");
            continue;
        }
        if forward>0. && rng.gen_bool(forward/diag){
            let data: Vec<u8> = "f".as_bytes().to_vec();
            write_io.write_all(&data).await.expect("write failed");
            continue;
            //send = send.add("r");
        }
        if forward<0. && rng.gen_bool(-forward/diag){
            let data: Vec<u8> = "b".as_bytes().to_vec();
            write_io.write_all(&data).await.expect("write failed");
            continue;
        }
        if steering>0. && rng.gen_bool(steering/diag){
            let data: Vec<u8> = "r".as_bytes().to_vec();
            write_io.write_all(&data).await.expect("write failed");
            continue;
            //send = send.add("r");
        }
        if steering<0. && rng.gen_bool(-steering/diag){
            let data: Vec<u8> = "l".as_bytes().to_vec();
            write_io.write_all(&data).await.expect("write failed");
            continue;
        }
        let data: Vec<u8> = "s".as_bytes().to_vec();
        write_io.write_all(&data).await.expect("write failed");
        
        //println!("mandato f {steering}");

    }
    //Ok(())
}

async fn car1(char: bluer::gatt::remote::Characteristic){
    println!("controlling car 1");
    old_car_control(char).await.unwrap();
}
async fn car2(char: bluer::gatt::remote::Characteristic){
    println!("{}", "controlling car 2".yellow());
    old_car_control(char).await.unwrap();
}



#[tokio::main(flavor = "current_thread")]
async fn main() -> bluer::Result<()> {
    let mut bl = Bluetooth::new();
    bl.add_device(
        Uuid::from_u128(0x0000ffe000001000800000805f9b34fb),
        Uuid::from_u128(0x0000ffe100001000800000805f9b34fb),
        [0xA8, 0x10, 0x87, 0x67, 0x73, 0x2A].into(), Box::new(car1));
    bl.add_device(
        Uuid::from_u128(0x0000ffe000001000800000805f9b34fb),
        Uuid::from_u128(0x0000ffe200001000800000805f9b34fb),
        [0x48, 0x87, 0x2D, 0x11, 0xA6, 0xF1].into(), Box::new(car2));
        
    /*bl.add_device(
        Uuid::from_u128(0x0000110c00001000800000805f9b34fb),
        Uuid::from_u128(0x0000110c00001000800000805f9b34fb),
        [0xA8, 0x10, 0x87, 0x67, 0x73, 0x2A].into(), Box::new(y));    */
    bl.scan().await.unwrap();
    Ok(())
}
