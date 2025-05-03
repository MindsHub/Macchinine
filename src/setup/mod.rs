#[cfg(target_os = "android")]
pub mod android;

#[cfg(not(target_os = "android"))]
pub mod pc;

#[cfg(target_os = "android")]
pub use android::*;
#[cfg(not(target_os = "android"))]
pub use pc::*;

use thiserror::Error;


#[derive(Debug, Error)]
pub enum Error {
    #[error("Not found")]
    NotFound,
  #[error("Btleplug error: {0}")]
  Btleplug(#[from] btleplug::Error),

  #[cfg(target_os = "android")]
  #[error("JNI {0}")]
  Jni(#[from] jni::errors::Error),

  #[cfg(target_os = "android")]
  #[error("Cannot initialize CLASS_LOADER")]
  ClassLoader,

  //#[error("Cannot initialize RUNTIME")]
  //Runtime,
  #[cfg(target_os = "android")]
  #[error("Java vm not initialized")]
  JavaVM,
}