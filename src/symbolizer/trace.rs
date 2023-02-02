#[derive(Clone, Debug)]
pub struct Trace {
    pub full: bool,
    pub index: usize,
    pub parent: Option<Box<Trace>>
}

impl Default for Trace {
    
    fn default() -> Self {
        Self { 
            full: false,
            index: 0, 
            parent: None
        }
    }

}

impl Trace {

    pub fn full() -> Self {
        Self {
            full: true,
            index: 0,
            parent: None
        }
    }

    pub fn new(index: usize, parent: Trace) -> Self {
        Self {
            full: false,
            index,
            parent: Some(Box::new(parent))
        }
    }

    fn as_vec(&self) -> Vec<usize> {
        let mut vec = Vec::new();
        if let Some(parent) = &self.parent {
            vec.append(&mut parent.as_vec());
        }
        vec.push(self.index);
        vec
    }

    pub fn follows_path(&self, other: &Trace) -> bool {
        let self_vec = self.as_vec();
        let other_vec = other.as_vec();

        if self_vec.len() < other_vec.len() {
            return false;
        }

        for i in 0..(other_vec.len() - 1) {
            if self_vec[i] != other_vec[i] {
                return false;
            }
        }
        
        true

        // match (&self.parent, &other.parent) {
        //     (None, None) => true,
        //     (None, Some(_)) => false,
        //     (Some(_), None) => self.index == other.index,
        //     (Some(lhs), Some(rhs)) => {
        //         if !lhs.follows_path(rhs) {
        //             return false;
        //         };
        //         self.index == other.index
        //     },
        // }
    }

}