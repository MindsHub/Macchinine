/*  

    let mut gilrs = Gilrs::new().unwrap();

    // Iterate over all connected gamepads
    for (_id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }

    let mut active_gamepad = None;

    loop {
        // Examine new events
        while let Some(Event { id, event, time }) = gilrs.next_event() {
            println!("{:?} New event from {}: {:?}", time, id, event);
            active_gamepad = Some(id);
        }

        // You can also use cached gamepad state
        if let Some(gamepad) = active_gamepad.map(|id| gilrs.gamepad(id)) {
            if gamepad.is_pressed(Button::South) {
                println!("Button South is pressed (XBox - A, PS - X)");
            }
        }
    }

}*/
use gilrs::{Gilrs, Event, GamepadId};
//libbluetooth-dev
use bluetooth_serial_port::{BtProtocol, BtSocket};
use rand::Rng;
use std::io::{Write};
use std::ops::Add;
use std::time;

fn bluetooth_connection()-> BtSocket{
    println!("bt-scanning");
    // scan for devices
    let devices = bluetooth_serial_port::scan_devices(time::Duration::from_secs(20)).unwrap();
    if devices.len() == 0 {
        panic!("No devices found");
    }

    println!("Found bluetooth devices {:?}", devices);

    // "device.addr" is the MAC address of the device
    let device = &devices[0];
    println!(
        "Connecting to `{}` ({})",
        device.name,
        device.addr.to_string()
    );

    // create and connect the RFCOMM socket
    let mut socket = BtSocket::new(BtProtocol::RFCOMM).unwrap();
    socket.connect(device.addr).unwrap();
    println!("connected");
    socket
}

fn connect_joistick(gir: &Gilrs)-> Option<GamepadId>{
    //let mut gilrs = Gilrs::new().unwrap();

    // Iterate over all connected gamepads
    for (_id, gamepad) in gir.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }

    Some(gir.gamepads().next().unwrap().0)
}

fn main() {
    
    //let mut gilrs = Gilrs::new().unwrap();
    let mut bt = bluetooth_connection();

    /*// BtSocket implements the `Read` and `Write` traits (they're blocking)
    let mut buffer = [0; 10];
    //let num_bytes_read = socket.read(&mut buffer[..]).unwrap();
    let num_bytes_written = socket.write(&buffer[0..10]).unwrap();
    println!(
        "Read bytes, wrote `{}` bytes",
         num_bytes_written
    );*/
    let mut rng = rand::thread_rng();
    //connect_joistick(&gilrs);
    let mut steering = 0.0f64;
    loop {
        // Examine new events
        /*while let Some(Event {event, id: _, time: _}) = gilrs.next_event() {
            match event{
                gilrs::EventType::AxisChanged(axis, value, _) => {
                    match axis{
                        gilrs::Axis::LeftStickX => {
                                steering = value as f64;
                        },
                        _ => {}
                    }
                },
                _ => {},
            }
        }*/

        let mut send  = "".to_string();
        /*if steering>0. && rng.gen_bool(steering as f64){
            send = send.add("r");
        }
        if steering<0. && rng.gen_bool(-steering as f64){
            send = send.add("l");
        }*/
        send = send.add("r");
        bt.write(send.as_bytes()).unwrap();
        println!("{send}");
        
    }

    // BtSocket also implements `mio::Evented` for async IO
    //let poll = Poll::new().unwrap();
    //poll.registry().register(&mut socket, Token(0), Interest::READABLE | Interest::WRITABLE).unwrap();
    // loop { ... poll events and wait for socket to be readable/writable ... }
}