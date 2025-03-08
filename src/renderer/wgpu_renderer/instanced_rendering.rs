use std::num::{NonZero, NonZeroU64};

use crate::renderer::ModelHandle;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
    normal: [[f32; 3]; 3],
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        // TODO: Should not be necessary.
        let model =
            cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from(self.rotation);
        InstanceRaw {
            model: model.into(),
            normal: cgmath::Matrix3::from(self.rotation).into(),
        }
    }
}

impl InstanceRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        // TODO: This sucks.
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in the shader.
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials, we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5, not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}


pub struct InstanceManager {
    // ModelHandle -> InstanceGroup
    pub instance_groups: Vec<InstanceGroup>, // Each element corresponds with a ModelID
}

pub struct InstanceGroup {
    model: u16, // TODO: Remove
    buffer: wgpu::Buffer,
    instance_indices: Box<[u16]>,      // Sparse set
    instance_data: Box<[InstanceRaw]>, // Sparse set
    free_list: Vec<u16>,               // Sparse set
    max_instances: u16,
    num_instances: u16,
}

impl InstanceGroup {
    // fn make_from_slice(device: &mut wgpu::Device, instance_data: &[Instance], max_instances: u16) -> InstanceGroup {
    //     let instance_data = instance_data.iter().map(Instance::to_raw).collect::<Vec<_>>();
    //     let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    //         label: Some("Instance Buffer"),
    //         contents: bytemuck::cast_slice(&instance_data),
    //         usage: wgpu::BufferUsages::VERTEX,
    //     });

    //     let num_instances: u16 = instance_data.len() as u16;

    //     Self {
    //         buffer,
    //         instance_indices: (0..instance_data.len() as u16).collect(),
    //         instance_data,
    //         num_instances,
    //         max_instances,
    //         free_list: vec![],
    //     }
    // }

    pub fn len(&self) -> u64 {
        self.num_instances as u64
    }
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    // model is a parameter for debug purposes
    pub fn new_empty(model: u16, device: &mut wgpu::Device, max_instances: u16) -> InstanceGroup {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instance buffer"), // TODO: Add model name to label
            size: (size_of::<InstanceRaw>() * max_instances as usize) as u64, // TODO: Add model name to label
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false, // TODO: Consider this
        });
        let instance_indices = vec![u16::MAX; max_instances as usize];
        let mut instance_data: Vec<InstanceRaw> = Vec::with_capacity(max_instances as usize);
        unsafe {
            instance_data.set_len(max_instances as usize);
        }
        let instance_indices = instance_indices.into_boxed_slice();
        debug_assert_eq!(instance_indices.len(), max_instances as usize);
        let instance_data = instance_data.into_boxed_slice();
        debug_assert_eq!(instance_data.len(), max_instances as usize);

        Self {
            model,
            buffer,
            instance_indices,
            instance_data,
            max_instances,
            num_instances: 0,
            free_list: vec![],
        }
    }
}

impl InstanceManager {
    // Handles for instances created this way are like (model, 0..N).
    pub fn new() -> InstanceManager {
        Self {
            instance_groups: vec![],
        }
    }

    pub fn add_instance_group(&mut self, device: &mut wgpu::Device, model: u16, max_instances: u16) {
        self.instance_groups.push(InstanceGroup::new_empty(model, device, max_instances));
    }

    pub fn set_from_slice(
        &mut self,
        model: ModelHandle,
        instances: &[Instance],
        queue: &mut wgpu::Queue,
    ) {
        let instance_group = &mut self.instance_groups[model.0 as usize];
        debug_assert_eq!(model.0, instance_group.model);
        assert!(instances.len() <= instance_group.max_instances as usize);
        instance_group.free_list.clear();
        instance_group.num_instances = instances.len() as u16;

        for (i, item) in instances.iter().map(Instance::to_raw).enumerate() {
            instance_group.instance_data[i] = item;
        }
        for i in 0..instances.len() as u16 {
            instance_group.instance_indices[i as usize] = i;
        }

        let mut buffer_view = queue
            .write_buffer_with(
                &instance_group.buffer,
                0,
                NonZero::new(instances.len() as u64 * Self::INSTANCE_SIZE).unwrap(),
            )
            .expect("Could not access instance buffer.");

        buffer_view.copy_from_slice(bytemuck::cast_slice(&instance_group.instance_data));
    }

    pub fn clear_instances(&mut self, model: ModelHandle) {
        todo!()
    }

    pub fn add_from_slice(
        &mut self,
        model: ModelHandle,
        instances: &[Instance],
        queue: &mut wgpu::Queue,
    ) {
        // TODO We need to fill all the slots in the free list first and then fill the rest normally.
        todo!()
    }

    const INSTANCE_SIZE: u64 = size_of::<InstanceRaw>() as u64;
    const INSTANCE_SIZE_NZ: NonZeroU64 = unsafe { NonZeroU64::new_unchecked(Self::INSTANCE_SIZE) };

    pub fn add_instance(
        &mut self,
        queue: &wgpu::Queue,
        model: ModelHandle,
        instance: &Instance,
    ) -> InstanceHandle {
        const MAX_INSTANCES: u16 = 100;

        // let mut instance_group = self.instance_groups.entry(model.0).or_insert(InstanceGroup::make_empty(device, MAX_INSTANCES));
        let instance_group = &mut self.instance_groups[model.0 as usize];
        debug_assert_eq!(model.0, instance_group.model);

        // assert_ne!(instance_group.max_instances, instance_group.num_instances);
        if instance_group.max_instances == instance_group.num_instances {
            panic!("Instance buffer is full.");
        }

        let instance_index = instance_group.num_instances as u16;

        {
            let instance_data = instance.to_raw();
            let mut buffer_view = queue
                .write_buffer_with(
                    &instance_group.buffer,
                    instance_index as u64 * Self::INSTANCE_SIZE,
                    Self::INSTANCE_SIZE_NZ,
                )
                .expect("Could not access instance buffer.");
            buffer_view.copy_from_slice(bytemuck::bytes_of(&instance_data));
            instance_group.instance_data[instance_index as usize] = instance_data;
        }

        let instance_id = if instance_group.free_list.is_empty() {
            instance_group.num_instances
        } else {
            instance_group.free_list.pop().unwrap()
        };
        instance_group.num_instances += 1;
        instance_group.instance_indices[instance_id as usize] = instance_index;

        InstanceHandle(model, instance_id)
    }

    // Returns an instance id.
    pub fn update_instance(
        &mut self,
        queue: &wgpu::Queue,
        instance_handle: InstanceHandle,
        new_instance: &Instance,
    ) {
        let instance_group = &mut self.instance_groups[instance_handle.0 .0 as usize];
        debug_assert_eq!(instance_handle.0.0, instance_group.model);
        let instance_index = instance_group.instance_indices[instance_handle.1 as usize];

        {
            let instance_data = new_instance.to_raw();
            let mut buffer_view = queue
                .write_buffer_with(
                    &instance_group.buffer,
                    instance_index as u64 * Self::INSTANCE_SIZE,
                    Self::INSTANCE_SIZE_NZ,
                )
                .expect("Could not access instance buffer.");
            buffer_view.copy_from_slice(bytemuck::bytes_of(&instance_data));
            instance_group.instance_data[instance_index as usize] = instance_data;
        }
    }

    pub fn delete_instance(&mut self, queue: &wgpu::Queue, instance_handle: InstanceHandle) {
        let instance_group = &mut self.instance_groups[instance_handle.0 .0 as usize];
        debug_assert_eq!(instance_handle.0.0, instance_group.model);
        debug_assert!(instance_group.num_instances != 0); // TODO: Unreachable
        let instance_index = instance_group.instance_indices[instance_handle.1 as usize];

        if instance_group.num_instances - 1 != instance_index {
            let mut buffer_view = queue
                .write_buffer_with(
                    &instance_group.buffer,
                    instance_index as u64 * Self::INSTANCE_SIZE,
                    Self::INSTANCE_SIZE_NZ,
                )
                .expect("Could not access instance buffer.");

            let last_instance_data =
                &instance_group.instance_data[(instance_group.num_instances - 1) as usize];
            buffer_view.copy_from_slice(bytemuck::bytes_of(last_instance_data));
        }

        instance_group.num_instances -= 1;
        instance_group.instance_indices[instance_group.num_instances as usize] = instance_index;
        instance_group.instance_indices[instance_index as usize] = u16::MAX;
        instance_group.free_list.push(instance_index);
    }
}
