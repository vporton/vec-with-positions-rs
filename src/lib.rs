use std::cell::Cell;
use std::rc::Rc;

/// A `Vec` inside together with positions that move together with the elements if the `Vec`
/// has deletions or insertions.
///
/// Implemented partially.

struct VecWithPositions<T> {
    vec: Vec<T>,
    positions: Vec<Rc<Position<T>>>,
}

impl<T> VecWithPositions<T> {
    pub fn new() -> Self {
        Self {
            vec: Vec::new(),
            positions: Vec::new(),
        }
    }
    pub fn push(&mut self, value: T) {
        self.vec.push(value)
    }
    pub fn append(&mut self, other: &mut Vec<T>) {
        self.vec.append(other)
    }
    pub fn remove(&mut self, index: usize) -> T {
        let result = self.vec.remove(index);
        for pos in &mut self.positions {
            if pos.pos >= index && pos.pos != 0 {
                let position = Rc::get_mut(&mut *pos).unwrap();
                position.pos -= 1;
            }
        }
        result
    }
    pub fn clear(&mut self) {
        self.vec.clear();
    }
    pub fn get(&self, index: usize) -> Option<&T> {
        self.vec.get(index)
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.vec.get_mut(index)
    }
}

struct Position<T> {
    of: Rc<Cell<VecWithPositions<T>>>,
    pos: usize,
    pos_in_positions: usize,
}

impl<T> Position<T> {
    pub fn new(container: Rc<Cell<VecWithPositions<T>>>, pos: usize)
        -> Rc<Self>
    {
        let mut positions = (*container).get_mut().get_mut().positions;
        let result = Rc::new(Self {
            of: container.clone(),
            pos,
            pos_in_positions: positions.len(),
        });
        positions.push(result.clone()); // FIXME: See docs of Rc::get_mut
        result
    }
    pub fn get(&self) -> Option<&T> {
        (*self.of).get().get(self.pos)
    }
    pub fn get_mut(&self) -> Option<&mut T> {
        (*self.of).get_mut().get_mut(self.pos)
    }
}

impl<T> Drop for Position<T> {
    fn drop(&mut self) {
        (*self.of).get_mut().positions.remove(self.pos_in_positions);
        for i in self.pos_in_positions .. (*self.of).get_mut().positions.len() {
            (*self.of).get_mut().positions[i].pos_in_positions -= 1;
        }
    }
}