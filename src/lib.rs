
use libc::*;

mod unity;

#[no_mangle]
pub extern "system" fn UnityPluginLoad(ifs: *mut unity::IUnityInterfaces)
{
    println!("Interceptor Loaded!");
}
