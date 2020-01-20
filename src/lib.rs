
use libc::*;
use lazy_static::*;
use std::cell::Cell;
use std::sync::RwLock;
use std::ptr::null_mut;

mod unity;
use unity::*;

pub struct VkRenderingInterceptor
{
    instance: UnityVulkanInstance
}
impl VkRenderingInterceptor
{
    pub fn new(ifs: *mut IUnityInterfaces) -> Self
    {
        let vkif = unsafe { ((*ifs).get_interface)(IUnityGraphicsVulkan::GUID) as *mut IUnityGraphicsVulkan };
        let instance = unsafe { ((*vkif).instance)() };

        VkRenderingInterceptor
        {
            instance
        }
    }
}
unsafe impl Sync for VkRenderingInterceptor {}
unsafe impl Send for VkRenderingInterceptor {}

#[no_mangle]
pub extern "system" fn rendering_event_ptr() -> UnityRenderingEvent { rendering_event }
extern "system" fn rendering_event(event_id: c_int)
{
    // Rendering Event Capture here
}

lazy_static!{
    static ref GRAPHICS_DEVICE: RwLock<Option<VkRenderingInterceptor>> = RwLock::new(None);
}
extern "system" fn gfx_event_handler(event_type: UnityGfxDeviceEventType)
{
    if event_type == kUnityGfxDeviceEventInitialize
    {
        // init here
        let rt = GFX_IF.with(|o| unsafe { ((*o.get()).get_renderer)() });
        if rt != kUnityGfxRendererVulkan
        {
            // Renderer Type is not supported! ignoring
            return;
        }

        *GRAPHICS_DEVICE.write().unwrap() = Some(INTERFACES.with(|v| VkRenderingInterceptor::new(v.get())));
    }
    else if event_type == kUnityGfxDeviceEventShutdown
    {
        // fini here
        *GRAPHICS_DEVICE.write().unwrap() = None;
    }
}

thread_local!{
    static INTERFACES: Cell<*mut IUnityInterfaces> = Cell::new(null_mut());
    static GFX_IF: Cell<*mut IUnityGraphics> = Cell::new(null_mut());
}
#[no_mangle]
pub extern "system" fn UnityPluginLoad(ifs: *mut IUnityInterfaces)
{
    INTERFACES.with(|v| v.set(ifs));
    let gfx_if = unsafe { ((*ifs).get_interface)(IUnityGraphics::GUID) as *mut IUnityGraphics };
    GFX_IF.with(|v| v.set(gfx_if));
    unsafe { ((*gfx_if).register_device_event_callback)(gfx_event_handler); }
    
    // Manual Initialization
    // ref: https://docs.unity3d.com/Manual/NativePluginInterface.html
    gfx_event_handler(kUnityGfxDeviceEventInitialize);
}
#[no_mangle]
pub extern "system" fn UnityPluginUnload()
{
    GFX_IF.with(|o| unsafe { ((*o.get()).unregister_device_event_callback)(gfx_event_handler) });
}
