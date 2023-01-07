////////////////////////////////////////////////////////////////////////////////////////////////////
//                                           Source File                                          //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

#[derive(Clone)]
pub struct SourceFile {
    pub path: String,
    pub src: String
}

impl SourceFile {

    pub fn new(path: String, src: String) -> Self {
        Self {
            path,
            src
        }
    }

    pub fn name_ext(&self) -> String {
        let index = self.path.rfind('/').map(|x| x + 1).unwrap_or(0);
        self.path[index..(self.path.len() - 1)].to_string()
    }

    pub fn name(&self) -> String {
        let name_ext = self.name_ext();

        let index = name_ext.rfind('.').unwrap_or(name_ext.len() - 1);
        name_ext[0..index].to_string()
    }

    pub fn ext(&self) -> String {
        let name_ext = self.name_ext();

        let index = name_ext.rfind('.').map(|x| x + 1).unwrap_or(name_ext.len() - 1);
        if index == name_ext.len() - 1 {
            return "".to_string();
        }
        name_ext[index..(name_ext.len() - 1)].to_string()
    }

}