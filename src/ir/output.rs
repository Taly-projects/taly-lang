use crate::{util::position::Positioned, parser::node::Node};

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                             Include                                            //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

#[derive(Clone, Debug)]
pub enum IncludeType {
    External,
    StdExternal,
    Internal
}

#[derive(Clone, Debug)]
pub struct Include {
    pub include_type: IncludeType,
    pub path: Positioned<String>
}

impl Include {

    pub fn full_path(&self) -> String {
        match self.include_type {
            IncludeType::External => format!("c-{}", self.path.data),
            IncludeType::StdExternal => format!("std-{}", self.path.data),
            IncludeType::Internal => self.path.data.clone(),
        }
    }

}



////////////////////////////////////////////////////////////////////////////////////////////////////
//                                            IR Output                                           //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

#[derive(Clone, Debug)]
pub struct IROutput {
    pub includes: Vec<Include>,
    pub ast: Vec<Positioned<Node>>   
}