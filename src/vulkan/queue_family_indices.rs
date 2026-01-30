#![allow(unsafe_op_in_unsafe_fn)]
use crate::vulkan::device_util::SuitabilityError;
use crate::vulkan::render_app::AppData;
use anyhow::anyhow;
use vulkanalia::vk::{InstanceV1_0, KhrSurfaceExtension};
use vulkanalia::{Instance, vk};

#[derive(Copy, Clone, Debug)]
pub struct QueueFamilyIndices {
    pub graphics: u32,
    pub present: u32,
}

//Note! To Increase performance choose one queuefamily instead of separate '
// such that  drawing and presentation don't use different families.
impl QueueFamilyIndices {
    pub unsafe fn get(
        instance: &Instance,
        data: &AppData,
        physical_device: vk::PhysicalDevice,
    ) -> anyhow::Result<Self> {
        println!("before swap");
        let properties = instance.get_physical_device_queue_family_properties(physical_device);
        println!("after first");
        let graphics = properties
            .iter()
            .position(|p| p.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .map(|i| i as u32);
        println!("after properties");
        let mut present = None;
        for (index, _properties) in properties.iter().enumerate() {
            if instance.get_physical_device_surface_support_khr(
                physical_device,
                index as u32,
                data.surface,
            )? {
                present = Some(index as u32);
                break;
            }
        }

        if let (Some(graphics), Some(present)) = (graphics, present) {
            Ok(Self { graphics, present })
        } else {
            Err(anyhow!(SuitabilityError(
                "Missing required queue families."
            )))
        }
    }
}
