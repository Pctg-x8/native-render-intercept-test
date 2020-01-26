
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

pub struct RenderControl
{
    fp_acquire_next_image: PFN_vkAcquireNextImageKHR,
    fp_submit_commands: PFN_vkQueueSubmit,
    fp_present: PFN_vkQueuePresentKHR,
    fp_wait_fences: PFN_vkWaitForFences,
    fp_reset_fences: PFN_vkResetFences,
    image_ready_order: VkSemaphore,
    present_order: VkSemaphore,
    last_render: VkFence,
    has_last_render_issued: bool
}
impl RenderControl
{
    pub fn new(instance: &UnityVulkanInstance) -> Self
    {
        let fp_create_semaphore: PFN_vkCreateSemaphore = unsafe
        {
            std::mem::transmute((instance.get_instance_proc_addr)(instance.instance, b"vkCreateSemaphore\0".as_ptr() as *const _).unwrap())
        };
        let fp_create_fence: PFN_vkCreateFence = unsafe
        {
            std::mem::transmute((instance.get_instance_proc_addr)(instance.instance, b"vkCreateFence\0".as_ptr() as *const _).unwrap())
        };

        let mut image_ready_order = std::mem::MaybeUninit::uninit();
        let mut present_order = std::mem::MaybeUninit::uninit();
        let mut last_render = std::mem::MaybeUninit::uninit();
        fp_create_semaphore(instance.device, &Default::default(), std::ptr::null(), image_ready_order.as_mut_ptr());
        fp_create_semaphore(instance.device, &Default::default(), std::ptr::null(), present_order.as_mut_ptr());
        fp_create_fence(instance.device, &Default::default(), std::ptr::null(), last_render.as_mut_ptr());
        
        RenderControl
        {
            fp_acquire_next_image: unsafe
            {
                std::mem::transmute((instance.get_instance_proc_addr)(instance.instance, b"vkAcquireNextImageKHR\0".as_ptr() as *const _).unwrap())
            },
            fp_submit_commands: unsafe
            {
                std::mem::transmute((instance.get_instance_proc_addr)(instance.instance, b"vkQueueSubmit\0".as_ptr() as *const _).unwrap())
            },
            fp_present: unsafe
            {
                std::mem::transmute((instance.get_instance_proc_addr)(instance.instance, b"vkQueuePresentKHR\0".as_ptr() as *const _).unwrap())
            },
            fp_wait_fences: unsafe
            {
                std::mem::transmute((instance.get_instance_proc_addr)(instance.instance, b"vkWaitForFences\0".as_ptr() as *const _).unwrap())
            },
            fp_reset_fences: unsafe
            {
                std::mem::transmute((instance.get_instance_proc_addr)(instance.instance, b"vkResetFences\0".as_ptr() as *const _).unwrap())
            },
            image_ready_order: unsafe { image_ready_order.assume_init() },
            present_order: unsafe { present_order.assume_init() },
            last_render: unsafe { last_render.assume_init() },
            has_last_render_issued: false
        }
    }

    pub fn acquire_next_frame(&mut self, sw: VkSwapchainKHR, dev: VkDevice) -> u32
    {
        let mut next = 0;
        (self.fp_acquire_next_image)(dev, sw, std::u64::MAX, self.image_ready_order, std::ptr::null_mut(), &mut next);

        next
    }
    pub fn submit_command(&mut self, command: VkCommandBuffer, sw: VkSwapchainKHR, bb_index: u32, dev: VkDevice, gq: VkQueue)
    {
        if self.has_last_render_issued
        {
            (self.fp_wait_fences)(dev, 1, &self.last_render, true as _, std::u64::MAX);
            (self.fp_reset_fences)(dev, 1, &self.last_render);
            self.has_last_render_issued = false;
        }

        let subinfo = VkSubmitInfo
        {
            waitSemaphoreCount: 1,
            pWaitSemaphores: &self.image_ready_order,
            pWaitDstStageMask: &VK_PIPELINE_STAGE_TRANSFER_BIT,
            commandBufferCount: 1,
            pCommandBuffers: &command,
            signalSemaphoreCount: 1,
            pSignalSemaphores: &self.present_order,
            .. Default::default()
        };
        (self.fp_submit_commands)(gq, 1, &subinfo, self.last_render);
        self.has_last_render_issued = true;

        let pinfo = VkPresentInfoKHR
        {
            waitSemaphoreCount: 1,
            pWaitSemaphores: &self.present_order,
            swapchainCount: 1,
            pSwapchains: &sw,
            pImageIndices: &bb_index,
            .. Default::default()
        };
        (self.fp_present)(gq, &pinfo);
    }
}

pub struct ExtRenderTarget
{
    handle: HWND,
    instance: VkInstance,
    device: VkDevice,
    graphics_queue: VkQueue,
    get_instance_proc_addr: PFN_vkGetInstanceProcAddr,
    surface: VkSurfaceKHR,
    swapchain: VkSwapchainKHR,
    extent: VkExtent2D,
    bb_images: Vec<VkImage>,
    rc: RenderControl
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
        let w = 1280;
        let mut rect = RECT
        {
            left: 0, top: 0, right: w, bottom: w * 9 / 16
        };
        unsafe { AdjustWindowRectEx(&mut rect, ws, false as _, 0); }

        let handle = unsafe
        {
            CreateWindowExA(0, c.lpszClassName, b"RenderingInterceptorTest\0".as_ptr() as _, ws,
                CW_USEDEFAULT, CW_USEDEFAULT, rect.right - rect.left, rect.bottom - rect.top,
                null_mut(), null_mut(), c.hInstance, null_mut()
            )
        };

        let fp_get_physical_device_presentation_support: PFN_vkGetPhysicalDeviceWin32PresentationSupportKHR = unsafe
        {
            std::mem::transmute((instance.get_instance_proc_addr)(instance.instance, b"vkGetPhysicalDeviceWin32PresentationSupportKHR\0".as_ptr() as *const _).unwrap())
        };
        let fp_get_physical_device_surface_support: PFN_vkGetPhysicalDeviceSurfaceSupportKHR = unsafe
        {
            std::mem::transmute((instance.get_instance_proc_addr)(instance.instance, b"vkGetPhysicalDeviceSurfaceSupportKHR\0".as_ptr() as *const _).unwrap())
        };
        let pres_support = fp_get_physical_device_presentation_support(instance.physical_device, instance.queue_family_index);
        if pres_support == 0 { panic!("WindowSubsystem does not support Vulkan rendering?"); }

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
        let surface = unsafe { sptr.assume_init() };
        let mut surface_supported = 0;
        fp_get_physical_device_surface_support(instance.physical_device, instance.queue_family_index, surface, &mut surface_supported);
        if surface_supported == 0
        {
            panic!("Vulkan does not support this surface to render");
        }

        let fp_get_physical_device_surface_formats: PFN_vkGetPhysicalDeviceSurfaceFormatsKHR = unsafe
        {
            std::mem::transmute((instance.get_instance_proc_addr)(instance.instance, b"vkGetPhysicalDeviceSurfaceFormatsKHR\0".as_ptr() as *const _).unwrap())
        };
        let fp_get_physical_device_surface_present_modes: PFN_vkGetPhysicalDeviceSurfacePresentModesKHR = unsafe
        {
            std::mem::transmute((instance.get_instance_proc_addr)(instance.instance, b"vkGetPhysicalDeviceSurfacePresentModesKHR\0".as_ptr() as *const _).unwrap())
        };
        let fp_get_physical_device_surface_capabilities: PFN_vkGetPhysicalDeviceSurfaceCapabilitiesKHR = unsafe
        {
            std::mem::transmute((instance.get_instance_proc_addr)(instance.instance, b"vkGetPhysicalDeviceSurfaceCapabilitiesKHR\0".as_ptr() as *const _).unwrap())
        };
        let format = unsafe
        {
            let mut format_cnt = 0;
            fp_get_physical_device_surface_formats(instance.physical_device, surface, &mut format_cnt, std::ptr::null_mut());
            let mut fmts = Vec::with_capacity(format_cnt as _);
            fmts.set_len(format_cnt as _);
            fp_get_physical_device_surface_formats(instance.physical_device, surface, &mut format_cnt, fmts.as_mut_ptr());
            
            fmts.into_iter().find(|f| f.format == VK_FORMAT_R8G8B8A8_SRGB || f.format == VK_FORMAT_B8G8R8A8_SRGB).unwrap()
        };
        let pres_mode = unsafe
        {
            let mut pm_cnt = 0;
            fp_get_physical_device_surface_present_modes(instance.physical_device, surface, &mut pm_cnt, std::ptr::null_mut());
            let mut pres_modes = Vec::with_capacity(pm_cnt as _);
            pres_modes.set_len(pm_cnt as _);
            fp_get_physical_device_surface_present_modes(instance.physical_device, surface, &mut pm_cnt, pres_modes.as_mut_ptr());

            let pm0 = pres_modes[0];
            pres_modes.into_iter().find(|&m| m == VK_PRESENT_MODE_FIFO_KHR || m == VK_PRESENT_MODE_MAILBOX_KHR).unwrap_or(pm0)
        };
        let mut caps = std::mem::MaybeUninit::uninit();
        fp_get_physical_device_surface_capabilities(instance.physical_device, surface, caps.as_mut_ptr());
        let caps = unsafe { caps.assume_init() };
        let available_composite_alpha = if (caps.supportedCompositeAlpha & VK_COMPOSITE_ALPHA_INHERIT_BIT_KHR) != 0
        {
            VK_COMPOSITE_ALPHA_INHERIT_BIT_KHR
        }
        else { VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR };

        let fp_create_swapchain: PFN_vkCreateSwapchainKHR = unsafe
        {
            std::mem::transmute((instance.get_instance_proc_addr)(instance.instance, b"vkCreateSwapchainKHR\0".as_ptr() as *const _).unwrap())
        };
        let mut wrect = std::mem::MaybeUninit::uninit();
        unsafe { GetClientRect(handle, wrect.as_mut_ptr()); }
        let wrect = unsafe { wrect.assume_init() };
        let ew =
            if caps.currentExtent.width == 0xffff_ffff { (wrect.right - wrect.left) as _ }
            else { caps.currentExtent.width };
        let eh =
            if caps.currentExtent.height == 0xffff_ffff { (wrect.bottom - wrect.top) as _ }
            else { caps.currentExtent.height };
        let buffer_count = 2.max(caps.minImageCount).min(caps.maxImageCount);
        let scinfo = VkSwapchainCreateInfoKHR
        {
            surface,
            minImageCount: buffer_count,
            imageFormat: format.format,
            imageColorSpace: format.colorSpace,
            imageExtent: VkExtent2D { width: ew, height: eh },
            imageArrayLayers: 1,
            imageUsage: VK_IMAGE_USAGE_TRANSFER_DST_BIT,
            imageSharingMode: VK_SHARING_MODE_EXCLUSIVE,
            preTransform: VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR,
            compositeAlpha: available_composite_alpha,
            presentMode: pres_mode,
            .. Default::default()
        };
        let mut swapchain = std::mem::MaybeUninit::uninit();
        fp_create_swapchain(instance.device, &scinfo, std::ptr::null(), swapchain.as_mut_ptr());
        let swapchain = unsafe { swapchain.assume_init() };

        let fp_get_swapchain_images: PFN_vkGetSwapchainImagesKHR = unsafe
        {
            std::mem::transmute((instance.get_instance_proc_addr)(instance.instance, b"vkGetSwapchainImagesKHR\0".as_ptr() as *const _).unwrap())
        };
        let mut bb_image_count = 0;
        fp_get_swapchain_images(instance.device, swapchain, &mut bb_image_count, std::ptr::null_mut());
        let mut bb_images = Vec::with_capacity(bb_image_count as _);
        unsafe { bb_images.set_len(bb_image_count as _); }
        fp_get_swapchain_images(instance.device, swapchain, &mut bb_image_count, bb_images.as_mut_ptr());

        ExtRenderTarget
        {
            handle,
            instance: instance.instance,
            device: instance.device,
            graphics_queue: instance.graphics_queue,
            get_instance_proc_addr: instance.get_instance_proc_addr,
            surface,
            swapchain,
            extent: scinfo.imageExtent,
            bb_images,
            rc: RenderControl::new(instance)
        }
    }

    pub fn wait_next_frame(&mut self) -> u32
    {
        self.rc.acquire_next_frame(self.swapchain, self.device)
    }
    pub fn submit_command(&mut self, command: VkCommandBuffer, bb_index: u32)
    {
        self.rc.submit_command(command, self.swapchain, bb_index, self.device, self.graphics_queue);
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
impl Drop for ExtRenderTarget
{
    fn drop(&mut self)
    {
        let fp_destroy_swapchain: PFN_vkDestroySwapchainKHR = unsafe
        {
            std::mem::transmute((self.get_instance_proc_addr)(self.instance, b"vkDestroySwapchainKHR\0".as_ptr() as *const _).unwrap())
        };
        let fp_destroy_surface: PFN_vkDestroySurfaceKHR = unsafe
        {
            std::mem::transmute((self.get_instance_proc_addr)(self.instance, b"vkDestroySurfaceKHR\0".as_ptr() as *const _).unwrap())
        };

        fp_destroy_swapchain(self.device, self.swapchain, std::ptr::null_mut());
        fp_destroy_surface(self.instance, self.surface, std::ptr::null_mut());
    }
}

const SCREEN_CAPTURE_EVENT_ID: c_int = 1;

pub struct VkRenderingInterceptor
{
    uinstance: UnityGraphicsVulkanRef,
    instance: UnityVulkanInstance,
    ert: ExtRenderTarget,
    current_rb: UnityRenderBuffer,
    cmd_pool: VkCommandPool,
    cbuf: VkCommandBuffer,
    fp_reset_command_pool: PFN_vkResetCommandPool,
    fp_begin_command_record: PFN_vkBeginCommandBuffer,
    fp_end_command_record: PFN_vkEndCommandBuffer,
    fp_cmd_blit_image: PFN_vkCmdBlitImage,
    fp_cmd_pipeline_barrier: PFN_vkCmdPipelineBarrier
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

        let fp_create_command_pool: PFN_vkCreateCommandPool = unsafe
        {
            std::mem::transmute((instance.get_instance_proc_addr)(instance.instance, b"vkCreateCommandPool\0".as_ptr() as *const _).unwrap())
        };
        let fp_alloc_command_buffer: PFN_vkAllocateCommandBuffers = unsafe
        {
            std::mem::transmute((instance.get_instance_proc_addr)(instance.instance, b"vkAllocateCommandBuffers\0".as_ptr() as *const _).unwrap())
        };
        let cpinfo = VkCommandPoolCreateInfo
        {
            queueFamilyIndex: instance.queue_family_index,
            .. Default::default()
        };
        let mut cmd_pool = std::mem::MaybeUninit::uninit();
        fp_create_command_pool(instance.device, &cpinfo, std::ptr::null(), cmd_pool.as_mut_ptr());
        let cmd_pool = unsafe { cmd_pool.assume_init() };
        let ainfo = VkCommandBufferAllocateInfo
        {
            commandPool: cmd_pool,
            commandBufferCount: 1,
            level: VK_COMMAND_BUFFER_LEVEL_PRIMARY,
            .. Default::default()
        };
        let mut cbuf = std::mem::MaybeUninit::uninit();
        fp_alloc_command_buffer(instance.device, &ainfo, cbuf.as_mut_ptr());

        trace!("Interceptor::VkRenderingInterceptor Initialized");
        VkRenderingInterceptor
        {
            fp_reset_command_pool: unsafe
            {
                std::mem::transmute((instance.get_instance_proc_addr)(instance.instance, b"vkResetCommandPool\0".as_ptr() as *const _).unwrap())
            },
            fp_begin_command_record: unsafe
            {
                std::mem::transmute((instance.get_instance_proc_addr)(instance.instance, b"vkBeginCommandBuffer\0".as_ptr() as *const _).unwrap())
            },
            fp_end_command_record: unsafe
            {
                std::mem::transmute((instance.get_instance_proc_addr)(instance.instance, b"vkEndCommandBuffer\0".as_ptr() as *const _).unwrap())
            },
            fp_cmd_blit_image: unsafe
            {
                std::mem::transmute((instance.get_instance_proc_addr)(instance.instance, b"vkCmdBlitImage\0".as_ptr() as *const _).unwrap())
            },
            fp_cmd_pipeline_barrier: unsafe
            {
                std::mem::transmute((instance.get_instance_proc_addr)(instance.instance, b"vkCmdPipelineBarrier\0".as_ptr() as *const _).unwrap())
            },
            uinstance, instance, ert, current_rb: std::ptr::null_mut(),
            cmd_pool, cbuf: unsafe { cbuf.assume_init() }
        }
    }
    pub fn set_render_buffer(&mut self, rb: UnityRenderBuffer)
    {
        self.current_rb = rb;
    }
    
    pub fn handle_event(&mut self)
    {
        let rb_image = self.uinstance.access_render_buffer_texture(
            self.current_rb,
            Some(&VkImageSubresource { aspectMask: VK_IMAGE_ASPECT_COLOR_BIT, arrayLayer: 0, mipLevel: 0 }),
            VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL,
            VK_PIPELINE_STAGE_TRANSFER_BIT,
            VK_ACCESS_TRANSFER_READ_BIT,
            kUnityVulkanResourceAccess_PipelineBarrier
        ).expect("Unable to get render buffer texture");

        let bb_index = self.ert.wait_next_frame();
        (self.fp_reset_command_pool)(self.ert.device, self.cmd_pool, VK_COMMAND_POOL_RESET_RELEASE_RESOURCES_BIT);
        (self.fp_begin_command_record)(self.cbuf, &Default::default());

        let dst_image = self.ert.bb_images[bb_index as usize];
        let in_barrier_transfer_ready = VkImageMemoryBarrier
        {
            image: dst_image, subresourceRange: VkImageSubresourceRange
            {
                aspectMask: VK_IMAGE_ASPECT_COLOR_BIT,
                baseArrayLayer: 0, layerCount: 1,
                baseMipLevel: 0, levelCount: 1
            },
            oldLayout: VK_IMAGE_LAYOUT_PRESENT_SRC_KHR, newLayout: VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
            srcAccessMask: VK_ACCESS_MEMORY_READ_BIT, dstAccessMask: VK_ACCESS_TRANSFER_WRITE_BIT,
            srcQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED, dstQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED,
            .. Default::default()
        };
        let out_barrier_present_ready = VkImageMemoryBarrier
        {
            image: dst_image, subresourceRange: VkImageSubresourceRange
            {
                aspectMask: VK_IMAGE_ASPECT_COLOR_BIT,
                baseArrayLayer: 0, layerCount: 1,
                baseMipLevel: 0, levelCount: 1
            },
            oldLayout: VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL, newLayout: VK_IMAGE_LAYOUT_PRESENT_SRC_KHR,
            srcAccessMask: VK_ACCESS_TRANSFER_WRITE_BIT, dstAccessMask: VK_ACCESS_MEMORY_READ_BIT,
            srcQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED, dstQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED,
            .. Default::default()
        };
        let region = VkImageBlit
        {
            srcSubresource: VkImageSubresourceLayers
            {
                aspectMask: VK_IMAGE_ASPECT_COLOR_BIT,
                baseArrayLayer: 0,
                layerCount: 1,
                mipLevel: 0
            },
            dstSubresource: VkImageSubresourceLayers
            {
                aspectMask: VK_IMAGE_ASPECT_COLOR_BIT,
                baseArrayLayer: 0,
                layerCount: 1,
                mipLevel: 0
            },
            srcOffsets: [
                VkOffset3D { x: 0, y: 0, z: 0 },
                VkOffset3D { x: rb_image.extent.width as _, y: rb_image.extent.height as _, z: 1 }
            ],
            dstOffsets: [
                VkOffset3D { x: 0, y: 0, z: 0 },
                VkOffset3D { x: self.ert.extent.width as _, y: self.ert.extent.height as _, z: 1 }
            ]
        };

        (self.fp_cmd_pipeline_barrier)(self.cbuf, VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT, VK_PIPELINE_STAGE_TRANSFER_BIT, 0,
            0, std::ptr::null(), 0, std::ptr::null(), 1, &in_barrier_transfer_ready);
        (self.fp_cmd_blit_image)(self.cbuf, rb_image.image, rb_image.layout, dst_image, VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
            1, &region, VK_FILTER_LINEAR);
        (self.fp_cmd_pipeline_barrier)(self.cbuf, VK_PIPELINE_STAGE_TRANSFER_BIT, VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, 0,
            0, std::ptr::null(), 0, std::ptr::null(), 1, &out_barrier_present_ready);

        (self.fp_end_command_record)(self.cbuf);
        self.ert.submit_command(self.cbuf, bb_index);
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
        let mut rh = GRAPHICS_DEVICE.write().unwrap();
        if let Some(ref mut gd) = *rh { gd.handle_event(); }
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
