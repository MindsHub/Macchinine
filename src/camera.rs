

use std::sync::mpsc::{Sender, Receiver, channel};

use once_cell::sync::OnceCell;

/*
#[no_mangle]
pub extern "system" fn Java_com_example_cameraapp_openCamera(
    env: JNIEnv,
    _: JClass,
    activity: JObject,
) {
    // Initialize logger
    android_logger::init_once(Config::default());

    info!("Opening Camera from Rust");

    // Call Java methods via JNI here (e.g., use Camera API in Java)
    let class_name = "com/example/cameraapp/CameraHelper";
    let class = env.find_class(class_name).expect("Find CameraHelper class");
    let method_id = env
        .get_method_id(class, "openCamera", "(Landroid/app/Activity;)V")
        .expect("Get method id");

    // Call the openCamera method in Java
    env.call_method(
        activity,
        method_id,
        jni::signature::Signature::Void,
        &[],
    )
    .expect("Call Java method");
}*/


static FRAME_SENDER: OnceCell<Sender<Vec<u8>>> = OnceCell::new();

#[unsafe(no_mangle)]
#[cfg(target_os = "android")]
pub extern "system" fn Java_com_mindshub_macchinine_CameraHelper_processFrame<'local>(
    env: jni::JNIEnv<'local>,
    _class: jni::objects::JClass<'local>,
    frame_data: jni::sys::jbyteArray,
) {
    use jni::objects::{JObject, JClass};
    use jni::JNIEnv;
    use jni::sys::jstring;
    use log::{info, debug};
    use android_logger::Config;
    let bytes: Vec<u8> = env
        .convert_byte_array(frame_data)
        .expect("Error converting byte array");
    println!("Camera frame data received: {}", bytes.len());
    let _ =FRAME_SENDER.get().unwrap().send(bytes);
}


pub fn init_frame_sender()->Receiver<Vec<u8>>{
    let (sender, receiver) = channel::<Vec<u8>>();
    
    FRAME_SENDER.set(sender).unwrap();
    

    
    receiver
}