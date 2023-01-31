#[derive(Clone)]
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

}