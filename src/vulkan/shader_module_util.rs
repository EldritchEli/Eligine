#![allow(unsafe_op_in_unsafe_fn)]
use vulkanalia::bytecode::Bytecode;
use vulkanalia::vk::{DeviceV1_0, HasBuilder};
use vulkanalia::{Device, vk};

#[allow(unsafe_op_in_unsafe_fn)]

pub unsafe fn create_shader_module(
    device: &Device,
    bytecode: &[u8],
) -> anyhow::Result<vk::ShaderModule> {
    let bytecode = Bytecode::new(bytecode)?;
    let info = vk::ShaderModuleCreateInfo::builder()
        .code_size(bytecode.code_size())
        .code(bytecode.code());

    Ok(device.create_shader_module(&info, None)?)
}
