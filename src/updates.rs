//! Bridge to the stable Android OTA shell. Loaded native code is never overwritten.
#[cfg(target_os="android")]
static ANDROID_APP: std::sync::OnceLock<winit::platform::android::activity::AndroidApp> = std::sync::OnceLock::new();

#[cfg(target_os="android")]
pub fn configure(app:winit::platform::android::activity::AndroidApp) { let _=ANDROID_APP.set(app); }

pub fn open() -> Result<(),String> {
    #[cfg(target_os="android")] { shell_call("checkForUpdates") }
    #[cfg(not(target_os="android"))] {
        Err("Встроенные OTA-обновления доступны в Android APK.".into())
    }
}
pub fn mark_ready() {
    #[cfg(target_os="android")]
    if let Err(error)=shell_call("markUpdateHealthy") {log::warn!("OTA health marker: {error}");}
}
#[cfg(target_os="android")]
fn shell_call(method:&str)->Result<(),String> {
    let app=ANDROID_APP.get().ok_or("Android activity unavailable")?;
    let vm=unsafe{jni::JavaVM::from_raw(app.vm_as_ptr() as *mut jni::sys::JavaVM)}.map_err(|e|e.to_string())?;
    let mut env=vm.attach_current_thread().map_err(|e|e.to_string())?;
    let activity=unsafe{jni::objects::JObject::from_raw(app.activity_as_ptr() as jni::sys::jobject)};
    let result=env.call_method(&activity,method,"()V",&[]).map(|_|()).map_err(|e|e.to_string());
    std::mem::forget(activity);
    if result.is_err(){let _=env.exception_clear();}
    result
}
