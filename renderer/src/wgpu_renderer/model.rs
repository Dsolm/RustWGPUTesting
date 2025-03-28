use super::texture;

pub struct WgpuModel {
    // pub name: String,
    pub meshes: Vec<WgpuMesh>,
    pub materials: Vec<WgpuMaterial>,
}

#[allow(dead_code)]
pub struct WgpuMaterial { // TODO: Do we really need to keep al these fields here?
    pub name: String,
    pub diffuse_texture: texture::Texture,
    pub normal_texture: texture::Texture,
    pub bind_group: wgpu::BindGroup,
}

impl WgpuMaterial {
    pub fn new(
        device: &wgpu::Device,
        name: &str,
        diffuse_texture: texture::Texture,
        normal_texture: texture::Texture, // NEW!
        layout: &wgpu::BindGroupLayout,
    ) -> Self { 
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
                // NEW!
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
                },
            ],
            label: Some(name),
        });

        Self {
            name: String::from(name),
            diffuse_texture,
            normal_texture, // NEW!
            bind_group,
        }
    }
}

#[allow(dead_code)]
pub struct WgpuMesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],

    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
}

impl ModelVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

// model.rs
// pub trait DrawModel {
//     fn draw_model(&mut self, model: &WgpuModel, camera_bind_group: &wgpu::BindGroup, light_bind_group: &wgpu::BindGroup);
//     fn draw_model_instanced(
//         &mut self,
//         model: &WgpuModel,
//         instances: Range<u32>,
//         camera_bind_group: &wgpu::BindGroup,
//         light_bind_group: &wgpu::BindGroup
//     );

//     fn draw_mesh(&mut self, mesh: &WgpuMesh, material: &WgpuMaterial,camera_bind_group: &wgpu::BindGroup, light_bind_group: &wgpu::BindGroup);
//     fn draw_mesh_instanced(
//         &mut self,
//         mesh: &WgpuMesh,
//         material: &WgpuMaterial,
//         instances: Range<u32>,
//         camera_bind_group: &wgpu::BindGroup,
//         light_bind_group: &wgpu::BindGroup
//     );
// }
// impl<'a> DrawModel for wgpu::RenderPass<'a>
// {
//     fn draw_mesh(
//         &mut self, mesh: &WgpuMesh,
//         material: &WgpuMaterial,
//         camera_bind_group: &wgpu::BindGroup,
//         light_bind_group: &wgpu::BindGroup
//     ) {
//         self.draw_mesh_instanced(mesh, material, 0..1, camera_bind_group, light_bind_group);
//     }

//     fn draw_mesh_instanced(
//         &mut self,
//         mesh: &WgpuMesh,
//         material: &WgpuMaterial,
//         instances: Range<u32>,
//         camera_bind_group: &wgpu::BindGroup,
//         light_bind_group: &wgpu::BindGroup
//     ) {
//         self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
//         self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
//         self.set_bind_group(0, &material.bind_group, &[]);
//         self.set_bind_group(1, camera_bind_group, &[]);
//         self.set_bind_group(2, light_bind_group, &[]);
//         self.draw_indexed(0..mesh.num_elements, 0, instances); // 
//     }

//     fn draw_model(&mut self, model: &WgpuModel, camera_bind_group: &wgpu::BindGroup, light_bind_group: &wgpu::BindGroup) {
//         self.draw_model_instanced(model, 0..1, camera_bind_group, light_bind_group);
//     }

//     fn draw_model_instanced(
//         &mut self,
//         model: &WgpuModel,
//         instances: Range<u32>,
//         camera_bind_group: &wgpu::BindGroup,
//         light_bind_group: &wgpu::BindGroup
//     ) {
//         for mesh in &model.meshes {
//             let material = &model.materials[mesh.material];
//             self.draw_mesh_instanced(mesh, material, instances.clone(), camera_bind_group, light_bind_group); // TODO: Remove clone
//         }
//     }
// }