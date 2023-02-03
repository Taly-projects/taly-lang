////////////////////////////////////////////////////////////////////////////////////////////////////
//                                               File                                             //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

#[derive(Clone)]
pub struct File {
    pub name: String,
    pub header: String,
    pub src: String
}

impl File {

    pub fn new(name: String) -> File {
        Self {
            name,
            header: String::new(),
            src: String::new()
        }
    }

}



////////////////////////////////////////////////////////////////////////////////////////////////////
//                                             Project                                            //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

pub struct Project {
    pub files: Vec<File>
}

impl Project {

    pub fn new() -> Self {
        Self {
            files: Vec::new()
        }
    }

    pub fn get_file(&mut self, name: String) -> &mut File {
        let mut contains = false;
        for file in self.files.iter_mut() {
            if file.name == name {
                contains = true;
            }
        }

        if contains {
            for file in self.files.iter_mut() {
                if file.name == name {
                    return file;
                }
            }
            unreachable!()
        } else {
            let file = File::new(name);
            self.files.push(file);
            self.files.last_mut().unwrap()
        }
    }

}