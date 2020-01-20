
use libc::*;
use lazy_static::*;
use std::cell::Cell;
use std::sync::RwLock;
use std::ptr::null_mut;
use winapi::um::winuser::*;
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::shared::windef::HWND;
use winapi::shared::minwindef::{LRESULT, WPARAM, LPARAM, UINT};

mod unity;
use unity::*;

pub struct ExtRenderTarget
{
    handle: HWND
}
impl ExtRenderTarget
{
    pub fn new() -> Self
    {
        let c = WNDCLASSEXA
        {
            cbSize: std::mem::size_of::<WNDCLASSEXA>() as _,
            style: CS_OWNDC,
            lpfnWndProc: Some(Self::wev_callback),
            cbClsExtra: 0, cbWndExtra: 0,
            hInstance: unsafe { GetModuleHandleA(null_mut()) },
            hCursor: unsafe { LoadCursorA(null_mut(), b"IDC_ARROW\0".as_ptr() as _) },
            lpszClassName: b"com.cterm2.unity.render_interceptor.MainWindow\0".as_ptr() as _,
            .. unsafe { std::mem::zeroed() }
        };
        if unsafe { RegisterClassExA(&c) == 0 }
        {
            panic!("RegisterClass failed");
        }

        let handle = unsafe
        {
            CreateWindowExA(0, c.lpszClassName, b"RenderingInterceptorTest\0".as_ptr() as _,
                WS_OVERLAPPED | WS_CAPTION | WS_BORDER | WS_SYSMENU | WS_MINIMIZEBOX | WS_VISIBLE,
                CW_USEDEFAULT, CW_USEDEFAULT, 640, 480,
                null_mut(), null_mut(), null_mut(), null_mut()
            )
        };

        ExtRenderTarget
        {
            handle
        }
    }

    extern "system" fn wev_callback(wnd: HWND, msg: UINT, wp: WPARAM, lp: LPARAM) -> LRESULT
    {
        if msg == WM_QUIT
        {
            return 0;
        }

        unsafe { DefWindowProcA(wnd, msg, wp, lp) }
    }
}

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
