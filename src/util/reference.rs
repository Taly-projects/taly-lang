#[derive(Debug)]
pub struct MutRef<T> {
    ptr: *mut T
}

impl<T> MutRef<T> {

    pub fn new(ptr: &mut T) -> Self {
        Self {
            ptr
        }
    }

    pub fn get(&self) -> &mut T {
        unsafe {
            &mut (*self.ptr)
        }
    }

}

impl<T> Clone for MutRef<T> {
    
    fn clone(&self) -> Self {
        Self { 
            ptr: self.ptr.clone() 
        }
    }

}