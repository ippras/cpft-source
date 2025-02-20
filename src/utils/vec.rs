/// Extension methods for [`Vec`]
pub trait VecExt<T> {
    fn remove_by_value(&mut self, value: &T) -> Option<T>;
}

impl<T: PartialEq> VecExt<T> for Vec<T> {
    fn remove_by_value(&mut self, value: &T) -> Option<T> {
        let position = self.iter().position(|source| source == value)?;
        Some(self.remove(position))
    }
}
