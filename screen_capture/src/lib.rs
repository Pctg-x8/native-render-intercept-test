
use libc::*;
use lazy_static::*;
use log::*;
use bedrock::vk::*;
use std::cell::Cell;
use std::sync::RwLock;
use std::ptr::null_mut;
use winapi::um::winuser::*;
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::shared::windef::{RECT, HWND};
use winapi::shared::minwindef::{LRESULT, WPARAM, LPARAM, UINT};

mod unity;
use unity::*;

pub struct ExtRenderTarget
{
    handle: HWND,
    surface: VkSurfaceKHR
}
impl ExtRenderTarget
{
    pub fn new(instance: &UnityVulkanInstance) -> Self
    {
        trace!("Interceptor: ExtRenderTarget::new");

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

        let ws = WS_OVERLAPPED | WS_CAPTION | WS_BORDER | WS_SYSMENU | WS_MINIMIZEBOX | WS_VISIBLE;
        let mut rect = RECT
        {
            left: 0, top: 0, right: 640, bottom: 480
        };
        unsafe { AdjustWindowRectEx(&mut rect, ws, false as _, 0); }

        let handle = unsafe
        {
            CreateWindowExA(0, c.lpszClassName, b"RenderingInterceptorTest\0".as_ptr() as _, ws,
                CW_USEDEFAULT, CW_USEDEFAULT, rect.right - rect.left, rect.bottom - rect.top,
                null_mut(), null_mut(), c.hInstance, null_mut()
            )
        };

        let fp_create_surface_khr: PFN_vkCreateWin32SurfaceKHR = unsafe
        {
            std::mem::transmute((instance.get_instance_proc_addr)(instance.instance, b"vkCreateWin32SurfaceKHR\0".as_ptr() as *const _).unwrap())
        };
        let sinfo = VkWin32SurfaceCreateInfoKHR
        {
            hinstance: c.hInstance, hwnd: handle,
            .. Default::default()
        };
        let mut sptr = std::mem::MaybeUninit::uninit();
        let r = fp_create_surface_khr(instance.instance, &sinfo, std::ptr::null(), sptr.as_mut_ptr());
        if r != VK_SUCCESS { panic!("vkCreateWin32SurfaceKHR failed"); }

        ExtRenderTarget
        {
            handle,
            surface: unsafe { sptr.assume_init() }
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

const SCREEN_CAPTURE_EVENT_ID: c_int = 1;

pub struct VkRenderingInterceptor
{
    uinstance: UnityGraphicsVulkanRef,
    instance: UnityVulkanInstance,
    ert: ExtRenderTarget,
    current_rb: UnityRenderBuffer
}
impl VkRenderingInterceptor
{
    pub fn new(ifs: *mut IUnityInterfaces) -> Self
    {
        let uinstance = UnityGraphicsVulkanRef::from_interfaces(ifs).expect("no IUnityGraphicsVulkan");
        let instance = uinstance.instance();
        let ert = ExtRenderTarget::new(&instance);
        
        // Copyコマンド+Present命令を出すのでoutside renderpass、かつGraphics Queueアクセス可能である必要がある
        uinstance.configure_event(SCREEN_CAPTURE_EVENT_ID, &UnityVulkanPluginEventConfig
        {
            flags: 0,
            render_pass_precondition: kUnityVulkanRenderPass_EnsureOutside,
            graphics_queue_access: kUnityVulkanGraphicsQueueAccess_Allow
        });

        trace!("Interceptor::VkRenderingInterceptor Initialized");
        VkRenderingInterceptor
        {
            uinstance, instance, ert, current_rb: std::ptr::null_mut()
        }
    }
    pub fn set_render_buffer(&mut self, rb: UnityRenderBuffer)
    {
        self.current_rb = rb;
    }
    
    pub fn handle_event(&self)
    {
        let rb_image = self.uinstance.access_render_buffer_texture(
            self.current_rb,
            Some(&VkImageSubresource { aspectMask: VK_IMAGE_ASPECT_COLOR_BIT, arrayLayer: 0, mipLevel: 0 }),
            VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL,
            VK_PIPELINE_STAGE_TRANSFER_BIT,
            VK_ACCESS_TRANSFER_READ_BIT,
            kUnityVulkanResourceAccess_PipelineBarrier
        ).expect("Unable to get render buffer texture");
    }
}
unsafe impl Sync for VkRenderingInterceptor {}
unsafe impl Send for VkRenderingInterceptor {}

#[no_mangle]
pub extern "system" fn rendering_event_ptr() -> UnityRenderingEvent { rendering_event }
extern "system" fn rendering_event(event_id: c_int)
{
    if event_id == SCREEN_CAPTURE_EVENT_ID
    {
        let rh = GRAPHICS_DEVICE.read().unwrap();
        if let Some(ref gd) = *rh { gd.handle_event(); }
    }
}

#[no_mangle]
pub extern "system" fn set_render_buffer(rb: UnityRenderBuffer)
{
    let mut wh = GRAPHICS_DEVICE.write().unwrap();
    if let Some(ref mut gd) = *wh { gd.set_render_buffer(rb); }
}

type DebugFn = extern "system" fn(ostr: *const c_char);

lazy_static!{
    static ref GRAPHICS_DEVICE: RwLock<Option<VkRenderingInterceptor>> = RwLock::new(None);
    static ref DEBUG_FN: RwLock<Option<DebugFn>> = RwLock::new(None);
}
extern "system" fn gfx_event_handler(event_type: UnityGfxDeviceEventType)
{
    trace!("Interceptor Event: {}", event_type);

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
    // flexi_logger::Logger::with_str("trace").log_to_file().start().expect("Logger initialization failed");
    info!("Initializing Plugin...");

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
    info!("Uninitializing Plugin...");
    GFX_IF.with(|o| unsafe { ((*o.get()).unregister_device_event_callback)(gfx_event_handler) });
}
