use super::VulkanDebugName;

use ash::vk;

/// Implement the debug name api for any type+handle tuple.
impl<T> VulkanDebugName<T> for (vk::ObjectType, T)
where
    T: vk::Handle + Copy,
{
    fn type_and_handle(&self) -> (vk::ObjectType, T) {
        (self.0, self.1)
    }
}
