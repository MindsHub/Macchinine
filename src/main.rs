//! Discover Bluetooth devices and list them.
/*pub mod discover_services;
#[tokio::main(flavor = "current_thread")]
async fn main() -> bluer::Result<()> {
    discover_services::main_disc().await
}*/

/* use bluer::{Adapter, AdapterEvent, Address, DeviceEvent, AddressType, rfcomm::{SocketAddr, Stream},};
use futures::{pin_mut, stream::SelectAll, StreamExt};
use tokio::io::AsyncWriteExt;
use std::{collections::HashSet, env, any::Any};

async fn query_device(adapter: &Adapter, addr: Address) -> bluer::Result<()> {
    let device = adapter.device(addr)?;
    println!("    Address type:       {}", device.address_type().await?);
    println!("    Name:               {:?}", device.name().await?);
    println!("    Icon:               {:?}", device.icon().await?);
    println!("    Class:              {:?}", device.class().await?);
    println!("    UUIDs:              {:?}", device.uuids().await?.unwrap_or_default());
    println!("    Paired:             {:?}", device.is_paired().await?);
    println!("    Connected:          {:?}", device.is_connected().await?);
    println!("    Trusted:            {:?}", device.is_trusted().await?);
    println!("    Modalias:           {:?}", device.modalias().await?);
    println!("    RSSI:               {:?}", device.rssi().await?);
    println!("    TX power:           {:?}", device.tx_power().await?);
    println!("    Manufacturer data:  {:?}", device.manufacturer_data().await?);
    println!("    Service data:       {:?}", device.service_data().await?);
    Ok(())
}

async fn query_all_device_properties(adapter: &Adapter, addr: Address) -> bluer::Result<()> {
    let device = adapter.device(addr)?;
    let props = device.all_properties().await?;
    for prop in props {
        println!("    {:?}", &prop);
    }
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> bluer::Result<()> {
    //const PSM: u16 = PSM_LE_DYN_START;// + 5;
    /*let with_changes = env::args().any(|arg| arg == "--changes");
    let all_properties = env::args().any(|arg| arg == "--all-properties");
    let filter_addr: HashSet<_> = env::args().filter_map(|arg| arg.parse::<Address>().ok()).collect();*/
    let address = Address::new([0xA8, 0x10, 0x87, 0x67, 0x73, 0x2A]);
    //env_logger::init();
    //let session = bluer::Session::new().await?;
    //let adapter = session.default_adapter().await?;
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    
    let y = adapter.connect_device(address, AddressType::LePublic).await.unwrap();
    let y: SocketAddr = y.into();
    println!("{:?} {:?}", y.services().await, y);

    
    
    //let target_addr: Address = args[1].parse().expect("invalid address");
    let target_sa = SocketAddr::new(address, 1);

    println!("Connecting to {:?}", &target_sa);
    let mut stream = Stream::connect(target_sa).await.expect("connection failed");
    loop{
        stream.write("r".as_bytes()).await.unwrap();
    }
    todo!()
*/

//! Connects to the Bluetooth GATT echo service and tests it.
//! 
 use bluer::{gatt::remote::Characteristic, AdapterEvent, Device, Result};
 use bluer::Uuid;
const SERVICE_UUID: Uuid = Uuid::from_u128(0x0000ffe000001000800000805f9b34fb);

/// Characteristic UUID for GATT example.
const CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0x0000ffe200001000800000805f9b34fb);

use futures::{pin_mut, StreamExt};
use rand::Rng;
use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    time::{sleep, timeout},
};
use gilrs::{Gilrs, Event, GamepadId};
fn connect_joistick(gir: &Gilrs)-> Option<GamepadId>{
    //let mut gilrs = Gilrs::new().unwrap();

    // Iterate over all connected gamepads
    for (_id, gamepad) in gir.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }

    Some(gir.gamepads().next().unwrap().0)
}

async fn find_our_characteristic(device: &Device) -> Result<Option<Characteristic>> {
    let addr = device.address();
    let uuids = device.uuids().await?.unwrap_or_default();
    //let k = uuids.get(0).unwrap().as_bytes();
    println!("Discovered device {} with service UUIDs: {:?}", addr, uuids);
    for i in uuids.iter(){
        println!("{:?}", i.as_bytes());
    }
    
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
}