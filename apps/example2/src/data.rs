#[repr(C)]
#[derive(Copy,Clone,Debug,bytemuck::Pod,bytemuck::Zeroable)]
pub(crate) struct ControlPoints{
  pub   points:[[f32;2];4],
}

