use std::{sync::mpsc, time::Duration};

#[cfg(feature = "local_master")]
use btleplug::{api::Characteristic, platform::Peripheral};
use eframe::NativeOptions;
use main::{
    ble::connect, egui_wrapper::{start_gui, GuiCommand, GuiEvent}, get_remote, joistick::{self, get_input_from_joistick}, recv_messages, send_message, setup::create_runtime, Message, RUNTIME_STORAGE
};
use pollster::FutureExt;
use tokio::time::sleep;

#[cfg(all(feature = "local_master", feature = "remote_master"))]
compile_error!("Feature 1 and 2 are mutually exclusive and cannot be enabled together");

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Warn) // Default Log Level
        .parse_default_env()
        .init();
    let run_time = create_runtime().unwrap();
    let (sender_event_gui, receiver_event_gui) = mpsc::channel::<GuiEvent>();
    let (sender_command_gui, receiver_command_gui) = mpsc::channel::<GuiCommand>();

    let (sender_command_remote, reciver_sender_command_remote) = mpsc::channel::<Message>();
    let (sender_image_gui, receiver_image_gui) = mpsc::channel::<Vec<u8>>();
    let (mut remote_writer, mut remote_reader) = run_time.spawn(get_remote()).block_on().unwrap();
    #[cfg(feature = "remote_master")]
    run_time.spawn(async move {
        loop {
            let mut recv = None;
            while let Some(Message::Img(c)) = recv_messages(&mut remote_reader).await {
                recv = Some(c);
            }
            if let Some(c) = recv {
                sender_image_gui.send(c).unwrap();
            }else{
                sleep(Duration::from_millis(10)).await;
            }
        }
    });

    run_time.spawn(async move {
        /*let mut recv = None;
        while let Ok(x) = receiver_command_gui.try_recv() {
            recv = Some(x);
        }
        match recv {
            Some(GuiCommand::Connect(addr, service, charac)) => {
                let _ = send_message(
                    &mut remote,
                    Message::Connect {
                        addr: addr.to_string_no_delim(),
                        service: service.as_u128(),
                        char: charac.as_u128(),
                    },
                )
                .await;
            }
            _=>{}
        }*/

        loop {
            let mut recv = None;
            while let Ok(x) = reciver_sender_command_remote.try_recv() {
                recv = Some(x);
            }
            if let Some(Message::Motors(x, y)) = recv {
                log::error!("Sending motors command to remote: {:?}", (x, y));
                let _ = send_message(&mut remote_writer, Message::Motors(x, y)).await;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        
    });
    run_time.spawn(async move {
        #[cfg(feature = "local_master")]
        let mut ble: Option<(Peripheral, Characteristic)> = None;
        let joistick_reciver = get_input_from_joistick().await;
        sender_event_gui.send(GuiEvent::Motors(0.0, 0.0)).unwrap();
        loop {
            #[cfg(feature = "local_master")]
            if let Ok(ev) = receiver_command_gui.try_recv() {
                match ev {
                    GuiCommand::Connect{addr, serv, char} => {
                        if let Ok(b) = connect(addr, serv, char).await {
                            ble = Some(b);
                            sender_event_gui.send(GuiEvent::Connected).unwrap();
                        }
                    }
                }
            }
            let mut recv = None;
            while let Ok(ev) = joistick_reciver.try_recv() {
                recv = Some(ev);
            }
            if let Some((x, y)) = recv {
                log::error!("x: {}, d: {}", x, y);
                let (sx, dx) = joistick::convert_joistick(x, y);
                log::error!("sx: {}, dx: {}", sx, dx);
                sender_event_gui.send(GuiEvent::Motors(sx, dx)).unwrap();

                #[cfg(feature = "local_master")]
                {
                    use btleplug::api::Peripheral;
                    use main::convert_to_ble;
                    if let Some(ble) = ble.as_mut() {
                        let to_send = convert_to_ble(sx, dx);
                        let _ = ble
                            .0
                            .write(&ble.1, &[to_send], btleplug::api::WriteType::WithResponse)
                            .await;
                    }
                }
                #[cfg(feature = "remote_master")]
                {
                    //log::error!("Sending motors command to remote: {:?}", (sx, dx));
                    sender_command_remote.send(Message::Motors(sx, dx)).unwrap();
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    });

    RUNTIME_STORAGE
        .set(run_time)
        .unwrap_or_else(|_| panic!("Runtime already set"));

    //let frame_recv = camera::init_frame_sender();

    start_gui(
        NativeOptions::default(),
        receiver_event_gui,
        Some(receiver_image_gui),
        sender_command_gui,
    )
    .unwrap();
}
