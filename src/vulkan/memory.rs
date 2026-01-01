use crate::vulkan::uniform_buffer_object::UniformBuffer;

//pub unsafe fn map_uniform_buffer<U>(ubo: U, device: &Device, size: usize)
//where
//    U: UniformBuffer,
//{
//    let memory = unsafe {
//        self.device.map_memory(
//            object.uniform_buffers_memory[image_index],
//            0,
//            size_of::<PbrUniform>() as u64,
//            vk::MemoryMapFlags::empty(),
//        )
//    }?;
//
//    memcpy(&ubo, memory.cast(), 1);
//
//    self.device
//        .unmap_memory(object.uniform_buffers_memory[image_index]);
//}
