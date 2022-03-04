//! UNTESTED code.
//!
//! TODO: docs

use std::future::Future;
use std::pin::Pin;

#[derive(Clone, Copy, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct Position(pub usize); // TODO: pub?

pub trait ActiveResource: Clone {
    fn position(&self) -> &Position;
}

pub struct Circular<Active: ActiveResource, Inactive: Clone> {
    vec: Vec<Inactive>,
    position: Option<Position>,
    allocator: Box<dyn Fn(Inactive, Position) -> Pin<Box<dyn Future<Output = Active> + Send + Sync>> + Send + Sync>,
}

/// We have a vector of "resources", "allocated" positions therein, and "next" resource to be allocated.
/// There is the operation to replace a position in the vector of positions
/// by the available position.
///
/// Example: Several threads use a pool of network nodes to download from.
/// From the pool we "view" a range of currently used nodes, one by thread.
/// If a note is invalidated, it is removed from the list.
/// Nodes later than it in the range decrease their positions.
///
/// TODO: Test it.
impl<Active: ActiveResource, Inactive: Clone> Circular<Active, Inactive> {
    pub fn new(allocator: Box<dyn Fn(Inactive, Position) -> Pin<Box<dyn Future<Output = Active> + Send + Sync>> + Send + Sync>) -> Self {
        Self {
            vec: Vec::new(),
            position: None,
            allocator,
        }
    }
    pub fn push(&mut self, value: Inactive) {
        self.vec.push(value)
    }
    pub fn append(&mut self, other: &mut Vec<Inactive>) {
        self.vec.append(other)
    }
    pub fn remove(&mut self) -> Option<Inactive> {
        if let Some(mut pos) = self.position {
            let result = self.vec.remove(pos.0);
            if pos.0 == self.vec.len() {
                pos.0 = 0;
            }
            Some(result)
        } else {
            None
        }
    }

    pub fn inactive_is_empty(&self) -> bool {
        self.vec.is_empty()
    }
    pub fn inactive_len(&self) -> usize {
        self.vec.len()
    }

    pub fn inactive_iter(&self) -> std::slice::Iter<Inactive> {
        self.vec.iter()
    }
    pub fn inactive_iter_mut(&mut self) -> std::slice::IterMut<Inactive> {
        self.vec.iter_mut()
    }
    pub fn get_position(&self) -> Option<Position> {
        self.position
    }
    pub fn set_position(&mut self, pos: Option<Position>) {
        self.position = pos;
    }
    pub fn get_current_inactive(&self) -> Option<&Inactive> {
        self.position.map(|pos| &self.vec[pos.0])
    }
    pub fn get_current_inactive_mut(&mut self) -> Option<&mut Inactive> {
        self.position.map(|pos| &mut self.vec[pos.0])
    }
    pub fn clear(&mut self) {
        self.vec.clear();
        self.position = None;
    }

    pub async fn next(&mut self) -> Option<Active> {
        let new = self.allocate_base().await;
        let new2 = new.clone();
        self.position = new.map(|v| *v.position());
        new2
    }
    async fn allocate_base(&mut self) -> Option<Active> {
        let len = self.inactive_len();
        if len == 0 {
            None
        } else {
            let new_pos = if let Some(old_pos) = self.position {
                Position(if old_pos.0 + 1 != len {
                    old_pos.0 + 1
                } else {
                    0
                })
            } else {
                Position(0)
            };
            let active = (self.allocator)(self.vec[new_pos.0].clone(), new_pos).await;
            Some(active)
        }
    }
}

/// Tests do not pass.
#[cfg(test)]
mod tests {
    use crate::{Position, Circular, VecWithPositions};

    #[test]
    fn one_position_before() {
        let mut v = Circular::new();
        let mut input = (0..10).collect::<Vec<i32>>();
        v.append(&mut input);
        v.set_position(Some(Position(3)));
        v.remove(Position(5));
        assert_eq!(v.inactive_iter().map(|n| *n).collect::<Vec<i32>>(), vec![0, 1, 2, 3, 4, 6, 7, 8, 9]);
        assert_eq!(v.get_position(), Some(Position(3)));
    }

    #[test]
    fn one_position_middle() {
        let mut v = Circular::new();
        let mut input = (0..10).collect::<Vec<i32>>();
        v.append(&mut input);
        v.set_position(Some(Position(5)));
        v.remove(Position(5));
        assert_eq!(v.inactive_iter().map(|n| *n).collect::<Vec<i32>>(), vec![0, 1, 2, 3, 4, 6, 7, 8, 9]);
        assert_eq!(v.get_position(), Some(Position(5)));
    }

    #[test]
    fn one_position_after() {
        let mut v = Circular::new();
        let mut input = (0..10).collect::<Vec<i32>>();
        v.append(&mut input);
        v.set_position(Some(Position(7)));
        v.remove(Position(5));
        assert_eq!(v.inactive_iter().map(|n| *n).collect::<Vec<i32>>(), vec![0, 1, 2, 3, 4, 6, 7, 8, 9]);
        assert_eq!(v.get_position(), Some(Position(6)));
    }
}
