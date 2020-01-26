//! Unity Interfaces
#![allow(non_upper_case_globals, dead_code)]

use libc::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnityInterfaceGUID
{
    pub guid_high: c_ulonglong,
    pub guid_low: c_ulonglong
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IUnityInterface {}

#[repr(C)]
pub struct IUnityInterfaces
{
    pub get_interface: extern "system" fn(guid: UnityInterfaceGUID) -> *mut IUnityInterface,
    pub register_interface: extern "system" fn(guid: UnityInterfaceGUID, ptr: *mut IUnityInterface),
    pub get_interface_split: extern "system" fn(guid_high: c_longlong, guid_low: c_longlong) -> *mut IUnityInterface,
    pub register_interface_split: extern "system" fn(guid_high: c_longlong, guid_low: c_longlong, ptr: *mut IUnityInterface)
}

pub type UnityGfxRenderer = c_int;
pub const kUnityGfxRendererVulkan: UnityGfxRenderer = 21;

pub type UnityGfxDeviceEventType = c_int;
pub const kUnityGfxDeviceEventInitialize: UnityGfxDeviceEventType = 0;
pub const kUnityGfxDeviceEventShutdown: UnityGfxDeviceEventType = 1;
// pub const kUnityGfxDeviceEventBeforeReset: UnityGfxDeviceEventType = 2;
// pub const kUnityGfxDeviceEventAfterReset: UnityGfxDeviceEventType = 3;

pub type IUnityGraphicsDeviceEventCallback = extern "system" fn(eventType: UnityGfxDeviceEventType);

#[repr(C)]
pub struct IUnityGraphics
{
    pub get_renderer: extern "system" fn() -> UnityGfxRenderer,
    pub register_device_event_callback: extern "system" fn(callback: IUnityGraphicsDeviceEventCallback),
    pub unregister_device_event_callback: extern "system" fn(callback: IUnityGraphicsDeviceEventCallback),
    pub reserve_event_id_range: extern "system" fn(count: c_int) -> c_int
}
impl IUnityGraphics
{
    pub const GUID: UnityInterfaceGUID = UnityInterfaceGUID
    {
        guid_high: 0x7CBA0A9CA4DDB544u64, guid_low: 0x8C5AD4926EB17B11u64
    };
}

pub type UnityRenderingEvent = extern "system" fn(eventId: c_int);
pub type UnityRenderingEventAndData = extern "system" fn(eventId: c_int, data: *mut c_void);

// vk //

use bedrock::vk::*;

#[repr(C)]
pub struct UnityVulkanInstance
{
    pub pipeline_cache: VkPipelineCache,
    pub instance: VkInstance,
    pub physical_device: VkPhysicalDevice,
    pub device: VkDevice,
    pub graphics_queue: VkQueue,
    pub get_instance_proc_addr: PFN_vkGetInstanceProcAddr,
    pub queue_family_index: c_uint,
    pub _resv: [*mut c_void; 8]
}
#[repr(C)]
pub struct UnityVulkanMemory
{
    pub memory: VkDeviceMemory,
    pub offset: VkDeviceSize,
    pub size: VkDeviceSize,
    pub mapped: *mut c_void,
    pub flags: VkMemoryPropertyFlags,
    pub memory_type_index: c_uint,
    pub _resv: [*mut c_void; 4]
}

pub type UnityVulkanResourceAccessMode = c_uint;
pub const kUnityVulkanResourceAccess_ObserveOnly: UnityVulkanResourceAccessMode = 0;
pub const kUnityVulkanResourceAccess_PipelineBarrier: UnityVulkanResourceAccessMode = 1;
pub const kUnityVulkanResourceAccess_Recreates: UnityVulkanResourceAccessMode = 2;

#[repr(C)]
pub struct UnityVulkanImage
{
    pub memory: UnityVulkanMemory,
    pub image: VkImage,
    pub layout: VkImageLayout,
    pub aspect: VkImageAspectFlags,
    pub usage: VkImageUsageFlags,
    pub format: VkFormat,
    pub extent: VkExtent3D,
    pub tiling: VkImageTiling,
    pub type_: VkImageType,
    pub samples: VkSampleCountFlags,
    pub layers: c_int,
    pub mip_count: c_int,
    pub _resv: [*mut c_void; 4]
}
#[repr(C)]
pub struct UnityVulkanBuffer
{
    pub memory: UnityVulkanMemory,
    pub buffer: VkBuffer,
    pub size_in_bytes: isize,
    pub usage: VkBufferUsageFlags,
    pub _resv: [*mut c_void; 4]
}
#[repr(C)]
pub struct UnityVulkanRecordingState
{
    pub command_buffer: VkCommandBuffer,
    pub command_buffer_level: VkCommandBufferLevel,
    pub render_pass: VkRenderPass,
    pub framebuffer: VkFramebuffer,
    pub sub_pass_index: c_int,
    pub current_frame_number: c_ulonglong,
    pub safe_frame_number: c_ulonglong,
    pub _resv: [*mut c_void; 4]
}

pub type UnityVulkanEventRenderPassPreCondition = c_int;
pub const kUnityVulkanRenderPass_DontCare: UnityVulkanEventRenderPassPreCondition = 0;
pub const kUnityVulkanRenderPass_EnsureInside: UnityVulkanEventRenderPassPreCondition = 1;
pub const kUnityVulkanRenderPass_EnsureOutside: UnityVulkanEventRenderPassPreCondition = 2;

pub type UnityVulkanGraphicsQueueAccess = c_uint;
pub const kUnityVulkanGraphicsQueueAccess_DontCare: UnityVulkanGraphicsQueueAccess = 0;
pub const kUnityVulkanGraphicsQueueAccess_Allow: UnityVulkanGraphicsQueueAccess = 1;

pub type UnityVulkanEventConfigFlagBits = c_uint;
pub const kUnityVulkanEventConfigFlag_EnsurePreviousFrameSubmission: UnityVulkanEventConfigFlagBits = 1 << 0;
pub const kUnityVulkanEventConfigFlag_FlushCommandBuffers: UnityVulkanEventConfigFlagBits = 1 << 1;
pub const kUnityVulkanEventConfigFlag_SyncWorkerThreads: UnityVulkanEventConfigFlagBits = 1 << 2;
pub const kUnityVulkanEventConfigFlag_ModifiersCommandBuffersState: UnityVulkanEventConfigFlagBits = 1 << 3;

#[repr(C)]
#[derive(Clone)]
pub struct UnityVulkanPluginEventConfig
{
    pub render_pass_precondition: UnityVulkanEventRenderPassPreCondition,
    pub graphics_queue_access: UnityVulkanGraphicsQueueAccess,
    pub flags: u32
}

pub const UnityVulkanWholeImage: *const VkImageSubresource = std::ptr::null();
pub type UnityVulkanInitCallback = extern "system" fn(get_instance_proc_addr: PFN_vkGetInstanceProcAddr, userdata: *mut c_void) -> PFN_vkGetInstanceProcAddr;

pub type UnityVulkanSwapchainMode = c_uint;
pub const kUnityVulkanSwapchainMode_Default: UnityVulkanSwapchainMode = 0;
pub const kUnityVulkanSwapchainMode_Offscreen: UnityVulkanSwapchainMode = 1;

#[repr(C)]
#[derive(Clone)]
pub struct UnityVulkanSwapchainConfiguration
{
    pub mode: UnityVulkanSwapchainMode
}

pub enum RenderSurfaceBase {}
pub type UnityRenderBuffer = *mut RenderSurfaceBase;

#[repr(C)]
pub struct IUnityGraphicsVulkan
{
    pub intercept_initialization: extern "system" fn(func: UnityVulkanInitCallback, userdata: *mut c_void) -> bool,
    pub intercept_vulkan_api: extern "system" fn(name: *const c_char, func: PFN_vkVoidFunction) -> PFN_vkVoidFunction,
    pub configure_event: extern "system" fn(event_id: c_int, plugin_event_config: *const UnityVulkanPluginEventConfig),
    pub instance: extern "system" fn() -> UnityVulkanInstance,
    pub command_recording_state: extern "system" fn(out_command_recording_state: *mut UnityVulkanRecordingState, queue_access: UnityVulkanGraphicsQueueAccess) -> bool,
    pub access_texture: extern "system" fn(native_texture: *mut c_void, sub_resource: *const VkImageSubresource, layout: VkImageLayout,
        pipeline_stage_flags: VkPipelineStageFlags, access_flags: VkAccessFlags, access_mode: UnityVulkanResourceAccessMode, out_image: *mut UnityVulkanImage) -> bool,
    pub access_render_buffer_texture: extern "system" fn(native_render_buffer: UnityRenderBuffer, sub_resource: *const VkImageSubresource, layout: VkImageLayout,
        pipeline_stage_flags: VkPipelineStageFlags, access_flags: VkAccessFlags, access_mode: UnityVulkanResourceAccessMode, out_image: *mut UnityVulkanImage) -> bool,
    pub access_render_buffer_resolve_texture: extern "system" fn(native_render_buffer: UnityRenderBuffer, sub_resource: *const VkImageSubresource, layout: VkImageLayout,
        pipeline_stage_flags: VkPipelineStageFlags, access_flags: VkAccessFlags, access_mode: UnityVulkanResourceAccessMode, out_image: *mut UnityVulkanImage) -> bool,
    pub access_buffer: extern "system" fn(native_buffer: *mut c_void ,pipeline_stage_flags: VkPipelineStageFlags, access_flags: VkAccessFlags,
        access_mode: UnityVulkanResourceAccessMode, out_buffer: *mut UnityVulkanBuffer) -> bool,
    pub ensure_outside_render_pass: extern "system" fn(),
    pub ensure_inside_render_pass: extern "system" fn(),
    pub access_queue: extern "system" fn(_: UnityRenderingEventAndData, event_id: c_int, user_data: *mut c_void, flush: bool),
    pub configure_swapchain: extern "system" fn(swap_chain_config: *const UnityVulkanSwapchainConfiguration) -> bool
}
impl IUnityGraphicsVulkan
{
    pub const GUID: UnityInterfaceGUID = UnityInterfaceGUID
    {
        guid_high: 0x95355348d4ef4e11u64, guid_low: 0x9789313dfcffcc87u64
    };
}

use std::ptr::NonNull;

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct UnityGraphicsVulkanRef(NonNull<IUnityGraphicsVulkan>);
impl UnityGraphicsVulkanRef
{
    pub fn from_ptr(p: *mut IUnityGraphicsVulkan) -> Option<Self>
    {
        NonNull::new(p).map(UnityGraphicsVulkanRef)
    }
    pub fn from_interfaces(ifs: *mut IUnityInterfaces) -> Option<Self>
    {
        Self::from_ptr(unsafe { ((*ifs).get_interface)(IUnityGraphicsVulkan::GUID) as *mut IUnityGraphicsVulkan })
    }

    pub fn configure_event(&self, event_id: c_int, plugin_event_config: &UnityVulkanPluginEventConfig)
    {
        unsafe { (self.0.as_ref().configure_event)(event_id, plugin_event_config as _); }
    }
    
    pub fn instance(&self) -> UnityVulkanInstance
    {
        unsafe { (self.0.as_ref().instance)() }
    }
    pub fn command_recording_state(&self, out_cmd_recording_state: &mut UnityVulkanRecordingState, queue_access: UnityVulkanGraphicsQueueAccess) -> bool
    {
        unsafe { (self.0.as_ref().command_recording_state)(out_cmd_recording_state as _, queue_access) }
    }
    pub fn access_render_buffer_texture(&self,
        native_render_buffer: UnityRenderBuffer,
        sub_resource: Option<&VkImageSubresource>,
        layout: VkImageLayout,
        pipeline_stage_flags: VkPipelineStageFlags,
        access_flags: VkAccessFlags,
        access_mode: UnityVulkanResourceAccessMode) -> Option<UnityVulkanImage>
    {
        let mut oi = std::mem::MaybeUninit::uninit();
        let result = unsafe
        {
            (self.0.as_ref().access_render_buffer_texture)(native_render_buffer,
                sub_resource.map(|p| p as _).unwrap_or(std::ptr::null()), layout,
                pipeline_stage_flags, access_flags, access_mode, oi.as_mut_ptr())
        };
        if result { Some(unsafe { oi.assume_init() }) } else { None }
    }

    // RenderPass Controls //

    pub fn ensure_outside_render_pass(&self)
    {
        unsafe { (self.0.as_ref().ensure_outside_render_pass)(); }
    }
    pub fn ensure_inside_render_pass(&self)
    {
        unsafe { (self.0.as_ref().ensure_inside_render_pass)(); }
    }
}
