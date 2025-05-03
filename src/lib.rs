use std::{sync::mpsc, time::Duration};

use ble::connect;
use btleplug::api::{BDAddr, Peripheral};
use eframe::NativeOptions;
pub mod ble;
pub mod egui_wrapper;

pub mod camera;
#[cfg(not(target_os = "android"))]
pub mod joistick;
//pub mod relay;
pub mod relay2;
pub mod setup;
use egui_wrapper::{GuiCommand, GuiEvent, start_gui};
use once_cell::sync::OnceCell;
use pollster::FutureExt;
pub use relay2::*;

use setup::create_runtime;
use tokio::time::Instant;
use uuid::Uuid;
#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

pub static RUNTIME_STORAGE: OnceCell<tokio::runtime::Runtime> = OnceCell::new();

fn _main(options: NativeOptions) -> eframe::Result<()> {
    let run_time = create_runtime().unwrap();
    let (send_ble, recv_send_ble) = mpsc::channel::<u8>();
    let (sender_gui_event, receiver_gui_event) = mpsc::channel::<GuiEvent>();
    run_time.spawn(async move {
        let (p, c) = connect(
            BDAddr::from_str_no_delim("01234567AA19").unwrap(),
            Uuid::from_u128(0x0000ffe000001000800000805f9b34fb),
            Uuid::from_u128(0x0000ffe100001000800000805f9b34fb),
        )
        .await
        .unwrap();
        log::error!("Connected");
        loop {
            let mut recv = None;
            while let Ok(x) = recv_send_ble.try_recv() {
                recv = Some(x);
            }
            if let Some(x) = recv {
                log::error!("Sending command to device: {:?}", x);
                p.write(&c, &[x], btleplug::api::WriteType::WithResponse)
                    .await
                    .unwrap();
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    });

    //init camera
    let frame_recv = camera::init_frame_sender();
    let (mut remote_writer, mut remote_reader) = run_time.spawn(get_remote()).block_on().unwrap();
    //send to remote
    run_time.spawn(async move {
        
        log::error!("Connected starting listening to commands");
        let mut time_last_image = Instant::now();
        loop {
            // handle image, and send to remote
            let mut recv = None;
            while let Ok(c) = frame_recv.try_recv() {
                recv = Some(c);
            }
            if let Some(c) = recv {
                if time_last_image.elapsed().as_millis() > 100 {
                    time_last_image = Instant::now();
                    log::error!(
                        "Recved image from camera, sending to send_loop: {:?}",
                        c.len()
                    );
                    send_message(&mut remote_writer, Message::Img(c.into())).await;
                }
            }
        }
    });
    run_time.spawn(async move {
        loop {
            // handle command from gui
            let mut recv = None;
            while let Some(x) = recv_messages(&mut remote_reader).await {
                log::error!("Recved command from remote: {:?}", x);
                recv = Some(x);
            }
            match recv {
                Some(Message::Motors(x, y)) => {
                    log::error!("Recved motors command from remote: {:?}", (x, y));
                    send_ble.send(convert_to_ble(x, y)).unwrap();
                    sender_gui_event.send(GuiEvent::Motors(x, y)).unwrap();
                }
                _ => {}
            }
        }
    });

    RUNTIME_STORAGE
        .set(run_time)
        .unwrap_or_else(|_| panic!("Runtime already set"));

    let (cmd_sender, cmd_recv) = mpsc::channel::<GuiCommand>();
    start_gui(options, receiver_gui_event, None, cmd_sender)
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub fn android_main(app: AndroidApp) {
    use ble::connect;
    use winit::platform::android::EventLoopBuilderExtAndroid;

    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Debug),
    );

    let options = NativeOptions {
        event_loop_builder: Some(Box::new(move |builder| {
            builder.with_android_app(app);
        })),

        ..Default::default()
    };

    _main(options).unwrap_or_else(|err| {
        log::error!("Failure while running EFrame application: {err:?}");
    });
}

#[cfg(not(target_os = "android"))]
pub fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Warn) // Default Log Level
        .parse_default_env()
        .init();

    _main(NativeOptions::default()).unwrap();
}

pub fn convert_to_ble(x: f32, y: f32) -> u8 {
    let principale_u8 = ((x * 7.0).round() as i8) as u8;
    let secondario_u8 = ((y * 7.0).round() as i8) as u8;
    ((principale_u8 & (0x0f_u8)) * 16) | (secondario_u8 & (0x0f_u8))
}
