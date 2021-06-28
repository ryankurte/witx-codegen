
use std::rc::Rc;

use crate::astype::*;

use witnext::Layout as _;

impl From<witnext::IntRepr> for ASType {
    fn from(witx: witnext::IntRepr) -> Self {
        match witx {
            witnext::IntRepr::U8 => ASType::U8,
            witnext::IntRepr::U16 => ASType::U16,
            witnext::IntRepr::U32 => ASType::U32,
            witnext::IntRepr::U64 => ASType::U64,
        }
    }
}

impl From<&witnext::BuiltinType> for ASType {
    fn from(witx_builtin: &witnext::BuiltinType) -> Self {
        match witx_builtin {
            witnext::BuiltinType::Char => ASType::Char32,
            witnext::BuiltinType::F32 => ASType::F32,
            witnext::BuiltinType::F64 => ASType::F64,
            witnext::BuiltinType::S8 => ASType::S8,
            witnext::BuiltinType::S16 => ASType::S16,
            witnext::BuiltinType::S32 => ASType::S32,
            witnext::BuiltinType::S64 => ASType::S64,

            witnext::BuiltinType::U8 { lang_c_char: false } => ASType::U8,
            witnext::BuiltinType::U8 { lang_c_char: true } => ASType::Char8,
            witnext::BuiltinType::U16 => ASType::U16,
            witnext::BuiltinType::U32 {
                lang_ptr_size: false,
            } => ASType::U32,
            witnext::BuiltinType::U32 {
                lang_ptr_size: true,
            } => ASType::USize,
            witnext::BuiltinType::U64 => ASType::U64,
        }
    }
}

impl From<&witnext::Type> for ASType {
    fn from(type_witx: &witnext::Type) -> Self {
        match type_witx {
            witnext::Type::Builtin(witx_builtin) => ASType::from(witx_builtin),
            witnext::Type::ConstPointer(constptr_tref) => {
                let pointee = ASType::from(constptr_tref);
                ASType::ConstPtr(Rc::new(pointee))
            }
            witnext::Type::Pointer(constptr_tref) => {
                let pointee = ASType::from(constptr_tref);
                ASType::MutPtr(Rc::new(pointee))
            }
            witnext::Type::Handle(handle_data_type) => {
                // data type doesn't seem to be used for anything
                let resource_name = handle_data_type.resource_id.name.as_str().to_string();
                ASType::Handle(resource_name)
            }
            witnext::Type::Record(record) if record.is_tuple() =>
            // Tuple
            {
                let mut tuple_members = vec![];
                let layout_witx = &record.member_layout(true);
                for member_witx in layout_witx {
                    let member_tref = &member_witx.member.tref;
                    let member_offset = member_witx.offset;
                    let member = ASTupleMember {
                        offset: member_offset,
                        type_: Rc::new(ASType::from(member_tref)),
                        padding: 0,
                    };
                    tuple_members.push(member);
                }
                // Perform a second pass to compute padding between members
                let n = if layout_witx.is_empty() {
                    0
                } else {
                    layout_witx.len() - 1
                };
                for (i, member_witx) in layout_witx.iter().enumerate().take(n) {
                    let member_tref = &member_witx.member.tref;
                    let member_size = member_tref.mem_size(true);
                    let member_padding =
                        layout_witx[i + 1].offset - member_witx.offset - member_size;
                    tuple_members[i].padding = member_padding;
                }
                ASType::Tuple(tuple_members)
            }
            witnext::Type::Record(record) if record.bitflags_repr().is_none() =>
            // Struct
            {
                let mut struct_members = vec![];
                let layout_witx = &record.member_layout(true);
                for member_witx in layout_witx {
                    let member_name = member_witx.member.name.as_str().to_string();
                    let member_tref = &member_witx.member.tref;
                    let member_offset = member_witx.offset;
                    let member = ASStructMember {
                        name: member_name,
                        offset: member_offset,
                        type_: Rc::new(ASType::from(member_tref)),
                        padding: 0,
                    };
                    struct_members.push(member);
                }
                // Perform a second pass to compute padding between members
                let n = if layout_witx.is_empty() {
                    0
                } else {
                    layout_witx.len() - 1
                };
                for (i, member_witx) in layout_witx.iter().enumerate().take(n) {
                    let member_tref = &member_witx.member.tref;
                    let member_size = member_tref.mem_size(true);
                    let member_padding =
                        layout_witx[i + 1].offset - member_witx.offset - member_size;
                    struct_members[i].padding = member_padding;
                }
                ASType::Struct(struct_members)
            }
            witnext::Type::Record(record) if record.bitflags_repr().is_some() =>
            // Constants
            {
                let mut constants = vec![];
                let constants_repr = ASType::from(record.bitflags_repr().unwrap());
                for (idx, contants_witx) in record.member_layout(true).iter().enumerate() {
                    let constant_name = contants_witx.member.name.as_str().to_string();
                    let constant = ASConstant {
                        name: constant_name,
                        value: 1u64 << idx,
                    };
                    constants.push(constant);
                }
                ASType::Constants(ASConstants {
                    repr: Rc::new(constants_repr),
                    constants,
                })
            }
            witnext::Type::Record(record) => {
                dbg!(record);
                dbg!(record.bitflags_repr());
                unreachable!()
            }
            witnext::Type::Variant(variant)
                if (variant.is_enum() || variant.is_bool())
                    && variant.as_expected().is_none()
                    && variant.as_option().is_none() =>
            // Enum
            {
                let enum_repr = ASType::from(variant.tag_repr);
                let mut choices = vec![];
                for (idx, choice_witx) in variant.cases.iter().enumerate() {
                    let choice_name = choice_witx.name.as_str().to_string();
                    let choice = ASEnumChoice {
                        name: choice_name,
                        value: idx,
                    };
                    choices.push(choice);
                }
                // WITX exposes booleans as enums
                if choices.len() == 2
                    && choices[0].name == "false"
                    && choices[0].value == 0
                    && choices[1].name == "true"
                    && choices[1].value == 1
                {
                    ASType::Bool
                } else {
                    ASType::Enum(ASEnum {
                        repr: Rc::new(enum_repr),
                        choices,
                    })
                }
            }
            witnext::Type::Variant(variant)
                if variant.as_expected().is_none() && variant.as_option().is_some() =>
            // Option
            {
                let tag_repr = ASType::from(variant.tag_repr);
                let option_offset = variant.payload_offset(true);
                assert_eq!(variant.cases.len(), 1);
                let option_tref = &variant.cases[0].tref;
                let option_type = match &option_tref {
                    None => ASType::Void,
                    Some(type_witx) => ASType::from(type_witx),
                };
                ASType::Option(ASOption {
                    tag_repr: Rc::new(tag_repr),
                    offset: option_offset,
                    type_: Rc::new(option_type),
                })
            }
            witnext::Type::Variant(variant)
                if variant.as_expected().is_some() && variant.as_option().is_none() =>
            // Result
            {
                let tag_repr = ASType::from(variant.tag_repr);
                let result_offset = variant.payload_offset(true);
                assert_eq!(variant.cases.len(), 2);
                assert_eq!(variant.cases[0].name, "ok");
                assert_eq!(variant.cases[1].name, "err");
                let ok_tref = &variant.cases[0].tref;
                let ok_type = match &ok_tref {
                    None => ASType::Void,
                    Some(type_witx) => ASType::from(type_witx),
                };
                let error_tref = &variant.cases[1].tref;
                let error_type = match &error_tref {
                    None => ASType::Void,
                    Some(type_witx) => ASType::from(type_witx),
                };
                let full_size = variant.mem_size(true);
                let tag_size = variant.tag_repr.mem_size(true);
                let padding_after_tag = full_size - tag_size;
                ASType::Result(ASResult {
                    tag_repr: Rc::new(tag_repr),
                    result_offset,
                    padding_after_tag,
                    error_type: Rc::new(error_type),
                    ok_type: Rc::new(ok_type),
                })
            }
            witnext::Type::Variant(variant) =>
            // Tagged Union
            {
                let tag_repr = ASType::from(variant.tag_repr);
                let member_offset = variant.payload_offset(true);
                let mut members = vec![];
                for member_witx in &variant.cases {
                    let member_name = member_witx.name.as_str().to_string();
                    let member_type = match member_witx.tref.as_ref() {
                        None => ASType::Void,
                        Some(type_witx) => ASType::from(type_witx),
                    };
                    let member = ASUnionMember {
                        name: member_name,
                        type_: Rc::new(member_type),
                    };
                    members.push(member);
                }
                let full_size = variant.mem_size(true);
                let tag_size = variant.tag_repr.mem_size(true);
                let padding_after_tag = full_size - tag_size;
                let max_member_size = full_size - member_offset;
                ASType::Union(ASUnion {
                    tag_repr: Rc::new(tag_repr),
                    members,
                    member_offset,
                    padding_after_tag,
                    max_member_size,
                })
            }
            witnext::Type::List(items_tref) => {
                let elements_type = ASType::from(items_tref);
                match elements_type {
                    // The "string" keyword in WITX returns a Char32, even if the actual encoding is expected to be UTF-8
                    ASType::Char32 | ASType::Char8 => ASType::String(Rc::new(ASType::Char8)),
                    _ => ASType::Slice(Rc::new(elements_type)),
                }
            }
            witnext::Type::Buffer(buffer) if buffer.out => {
                let elements_type = ASType::from(&buffer.tref);
                ASType::WriteBuffer(Rc::new(elements_type))
            }
            witnext::Type::Buffer(buffer) => {
                let elements_typ = ASType::from(&buffer.tref);
                ASType::ReadBuffer(Rc::new(elements_typ))
            }
        }
    }
}

impl From<&witnext::TypeRef> for ASType {
    fn from(witx_tref: &witnext::TypeRef) -> Self {
        match witx_tref {
            witnext::TypeRef::Value(type_witx) => ASType::from(type_witx.as_ref()),
            witnext::TypeRef::Name(alias_witx) => {
                let alias_witx = alias_witx.as_ref();
                let alias_name = alias_witx.name.as_str().to_string();
                let alias_target = ASType::from(&alias_witx.tref);
                ASType::Alias(ASAlias {
                    name: alias_name,
                    type_: Rc::new(alias_target),
                })
            }
        }
    }
}