//! Unity Interfaces

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
struct IUnityInterface {}

#[repr(C)]
struct IUnityInterfaces
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
pub const kUnityGfxDeviceEventBeforeReset: UnityGfxDeviceEventType = 2;
pub const kUnityGfxDeviceEventAfterReset: UnityGfxDeviceEventType = 3;

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

pub type UnityRenderBuffer = c_void;

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
