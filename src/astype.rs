use std::rc::Rc;

/// Top level decoded module
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ASModule {
    pub name: String,
    // TODO: fill this in
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ASAlias {
    pub name: String,
    pub type_: Rc<ASType>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ASStructMember {
    pub name: String,
    pub offset: usize,
    pub type_: Rc<ASType>,
    pub padding: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ASEnumChoice {
    pub name: String,
    pub value: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ASEnum {
    pub repr: Rc<ASType>,
    pub choices: Vec<ASEnumChoice>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ASUnionMember {
    pub name: String,
    pub type_: Rc<ASType>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ASUnion {
    pub tag_repr: Rc<ASType>,
    pub members: Vec<ASUnionMember>,
    pub member_offset: usize,
    pub padding_after_tag: usize,
    pub max_member_size: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ASTupleMember {
    pub type_: Rc<ASType>,
    pub offset: usize,
    pub padding: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ASOption {
    pub tag_repr: Rc<ASType>,
    pub type_: Rc<ASType>,
    pub offset: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ASResult {
    pub tag_repr: Rc<ASType>,
    pub error_type: Rc<ASType>,
    pub ok_type: Rc<ASType>,
    pub result_offset: usize,
    pub padding_after_tag: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ASConstant {
    pub name: String,
    pub value: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ASConstants {
    pub repr: Rc<ASType>,
    pub constants: Vec<ASConstant>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ASType {
    Void,
    Alias(ASAlias),
    Bool,
    Char8,
    Char32,
    USize,
    F32,
    F64,
    S8,
    S16,
    S32,
    S64,
    U8,
    U16,
    U32,
    U64,
    Constants(ASConstants),
    Result(ASResult),
    Option(ASOption),
    Handle(String),
    Enum(ASEnum),
    Tuple(Vec<ASTupleMember>),
    ConstPtr(Rc<ASType>),
    MutPtr(Rc<ASType>),
    Union(ASUnion),
    Struct(Vec<ASStructMember>),
    Slice(Rc<ASType>),
    String(Rc<ASType>),
    ReadBuffer(Rc<ASType>),
    WriteBuffer(Rc<ASType>),
}


pub struct ASTypeDecomposed {
    pub name: String,
    pub type_: Rc<ASType>,
}

impl ASType {
    pub fn leaf(&self) -> &ASType {
        if let ASType::Alias(alias) = self {
            alias.type_.as_ref()
        } else {
            self
        }
    }

    pub fn decompose(&self, name: &str, as_mut_pointers: bool) -> Vec<ASTypeDecomposed> {
        let leaf = self.leaf();

        if as_mut_pointers {
            return match leaf {
                ASType::Void => vec![],
                _ => vec![ASTypeDecomposed {
                    name: name.to_string(),
                    type_: Rc::new(ASType::MutPtr(Rc::new(self.clone()))),
                }],
            };
        }

        match leaf {
            ASType::Void => vec![],
            ASType::ReadBuffer(elements_type)
            | ASType::WriteBuffer(elements_type)
            | ASType::Slice(elements_type)
            | ASType::String(elements_type) => {
                let ptr_name = format!("{}_ptr", name);
                let len_name = format!("{}_len", name);
                let ptr_type = if let ASType::WriteBuffer(_) = leaf {
                    ASType::MutPtr(elements_type.clone())
                } else {
                    ASType::ConstPtr(elements_type.clone())
                };
                let ptr_element = ASTypeDecomposed {
                    name: ptr_name,
                    type_: Rc::new(ptr_type),
                };
                let len_element = ASTypeDecomposed {
                    name: len_name,
                    type_: Rc::new(ASType::USize),
                };
                vec![ptr_element, len_element]
            }
            _ => {
                vec![ASTypeDecomposed {
                    name: name.to_string(),
                    type_: Rc::new(self.clone()),
                }]
            }
        }
    }
}
