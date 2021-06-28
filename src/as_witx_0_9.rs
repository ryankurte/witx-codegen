
use std::rc::Rc;

use crate::astype::*;

use witx0_9::{TypeRef, Layout as _};

impl From<witx0_9::IntRepr> for ASType {
    fn from(witx0_9: witx0_9::IntRepr) -> Self {
        match witx0_9 {
            witx0_9::IntRepr::U8 => ASType::U8,
            witx0_9::IntRepr::U16 => ASType::U16,
            witx0_9::IntRepr::U32 => ASType::U32,
            witx0_9::IntRepr::U64 => ASType::U64,
        }
    }
}

impl From<&witx0_9::BuiltinType> for ASType {
    fn from(witx0_9_builtin: &witx0_9::BuiltinType) -> Self {
        match witx0_9_builtin {
            witx0_9::BuiltinType::Char => ASType::Char32,
            witx0_9::BuiltinType::F32 => ASType::F32,
            witx0_9::BuiltinType::F64 => ASType::F64,
            witx0_9::BuiltinType::S8 => ASType::S8,
            witx0_9::BuiltinType::S16 => ASType::S16,
            witx0_9::BuiltinType::S32 => ASType::S32,
            witx0_9::BuiltinType::S64 => ASType::S64,

            witx0_9::BuiltinType::U8 { lang_c_char: false } => ASType::U8,
            witx0_9::BuiltinType::U8 { lang_c_char: true } => ASType::Char8,
            witx0_9::BuiltinType::U16 => ASType::U16,
            witx0_9::BuiltinType::U32 {
                lang_ptr_size: false,
            } => ASType::U32,
            witx0_9::BuiltinType::U32 {
                lang_ptr_size: true,
            } => ASType::USize,
            witx0_9::BuiltinType::U64 => ASType::U64,
        }
    }
}

/// Variant::as_option polyfill from [witnext@0.10.0-beta3](https://docs.rs/witnext/0.10.0-beta3/src/witnext/ast.rs.html#542)
pub fn variant_as_option(v: &witx0_9::Variant) -> Option<&TypeRef> {
    if v.cases.len() != 2 {
        return None;
    }
    if v.cases[0].name != "none" || v.cases[0].tref.is_some() {
        return None;
    }
    if v.cases[1].name != "some" {
        return None;
    }
    v.cases[1].tref.as_ref()
}

impl From<&witx0_9::Type> for ASType {
    fn from(type_witx0_9: &witx0_9::Type) -> Self {
        match type_witx0_9 {
            witx0_9::Type::Builtin(witx0_9_builtin) => ASType::from(witx0_9_builtin),
            witx0_9::Type::ConstPointer(constptr_tref) => {
                let pointee = ASType::from(constptr_tref);
                ASType::ConstPtr(Rc::new(pointee))
            }
            witx0_9::Type::Pointer(constptr_tref) => {
                let pointee = ASType::from(constptr_tref);
                ASType::MutPtr(Rc::new(pointee))
            }
            witx0_9::Type::Handle(_handle_data_type) => {
                // data type doesn't seem to be used for anything
                // TODO: witx::HandleDataType doesn't have this as an option
                // https://docs.rs/witx/0.9.1/witx/struct.HandleDatatype.html
                //let resource_name = handle_data_type.resource_id.name.as_str().to_string();
                //ASType::Handle(resource_name)
                unimplemented!()
            }
            witx0_9::Type::Record(record) if record.is_tuple() =>
            // Tuple
            {
                let mut tuple_members = vec![];
                let layout_witx0_9 = &record.member_layout();
                for member_witx0_9 in layout_witx0_9 {
                    let member_tref = &member_witx0_9.member.tref;
                    let member_offset = member_witx0_9.offset;
                    let member = ASTupleMember {
                        offset: member_offset,
                        type_: Rc::new(ASType::from(member_tref)),
                        padding: 0,
                    };
                    tuple_members.push(member);
                }
                // Perform a second pass to compute padding between members
                let n = if layout_witx0_9.is_empty() {
                    0
                } else {
                    layout_witx0_9.len() - 1
                };
                for (i, member_witx0_9) in layout_witx0_9.iter().enumerate().take(n) {
                    let member_tref = &member_witx0_9.member.tref;
                    let member_size = member_tref.mem_size();
                    let member_padding =
                        layout_witx0_9[i + 1].offset - member_witx0_9.offset - member_size;
                    tuple_members[i].padding = member_padding;
                }
                ASType::Tuple(tuple_members)
            }
            witx0_9::Type::Record(record) if record.bitflags_repr().is_none() =>
            // Struct
            {
                let mut struct_members = vec![];
                let layout_witx0_9 = &record.member_layout();
                for member_witx0_9 in layout_witx0_9 {
                    let member_name = member_witx0_9.member.name.as_str().to_string();
                    let member_tref = &member_witx0_9.member.tref;
                    let member_offset = member_witx0_9.offset;
                    let member = ASStructMember {
                        name: member_name,
                        offset: member_offset,
                        type_: Rc::new(ASType::from(member_tref)),
                        padding: 0,
                    };
                    struct_members.push(member);
                }
                // Perform a second pass to compute padding between members
                let n = if layout_witx0_9.is_empty() {
                    0
                } else {
                    layout_witx0_9.len() - 1
                };
                for (i, member_witx0_9) in layout_witx0_9.iter().enumerate().take(n) {
                    let member_tref = &member_witx0_9.member.tref;
                    let member_size = member_tref.mem_size();
                    let member_padding =
                        layout_witx0_9[i + 1].offset - member_witx0_9.offset - member_size;
                    struct_members[i].padding = member_padding;
                }
                ASType::Struct(struct_members)
            }
            witx0_9::Type::Record(record) if record.bitflags_repr().is_some() =>
            // Constants
            {
                let mut constants = vec![];
                let constants_repr = ASType::from(record.bitflags_repr().unwrap());
                for (idx, contants_witx0_9) in record.member_layout().iter().enumerate() {
                    let constant_name = contants_witx0_9.member.name.as_str().to_string();
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
            witx0_9::Type::Record(record) => {
                dbg!(record);
                dbg!(record.bitflags_repr());
                unreachable!()
            }
            witx0_9::Type::Variant(variant)
                if (variant.is_enum() || variant.is_bool())
                    && variant.as_expected().is_none()
                    && variant_as_option(variant).is_none() =>
            // Enum
            {
                let enum_repr = ASType::from(variant.tag_repr);
                let mut choices = vec![];
                for (idx, choice_witx0_9) in variant.cases.iter().enumerate() {
                    let choice_name = choice_witx0_9.name.as_str().to_string();
                    let choice = ASEnumChoice {
                        name: choice_name,
                        value: idx,
                    };
                    choices.push(choice);
                }
                // witx0_9 exposes booleans as enums
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
            witx0_9::Type::Variant(variant)
                if variant.as_expected().is_none() && variant_as_option(variant).is_some() =>
            // Option
            {
                let tag_repr = ASType::from(variant.tag_repr);
                let option_offset = variant.payload_offset();
                assert_eq!(variant.cases.len(), 1);
                let option_tref = &variant.cases[0].tref;
                let option_type = match &option_tref {
                    None => ASType::Void,
                    Some(type_witx0_9) => ASType::from(type_witx0_9),
                };
                ASType::Option(ASOption {
                    tag_repr: Rc::new(tag_repr),
                    offset: option_offset,
                    type_: Rc::new(option_type),
                })
            }
            witx0_9::Type::Variant(variant)
                if variant.as_expected().is_some() && variant_as_option(variant).is_none() =>
            // Result
            {
                let tag_repr = ASType::from(variant.tag_repr);
                let result_offset = variant.payload_offset();
                assert_eq!(variant.cases.len(), 2);
                assert_eq!(variant.cases[0].name, "ok");
                assert_eq!(variant.cases[1].name, "err");
                let ok_tref = &variant.cases[0].tref;
                let ok_type = match &ok_tref {
                    None => ASType::Void,
                    Some(type_witx0_9) => ASType::from(type_witx0_9),
                };
                let error_tref = &variant.cases[1].tref;
                let error_type = match &error_tref {
                    None => ASType::Void,
                    Some(type_witx0_9) => ASType::from(type_witx0_9),
                };
                let full_size = variant.mem_size();
                let tag_size = variant.tag_repr.mem_size();
                let padding_after_tag = full_size - tag_size;
                ASType::Result(ASResult {
                    tag_repr: Rc::new(tag_repr),
                    result_offset,
                    padding_after_tag,
                    error_type: Rc::new(error_type),
                    ok_type: Rc::new(ok_type),
                })
            }
            witx0_9::Type::Variant(variant) =>
            // Tagged Union
            {
                let tag_repr = ASType::from(variant.tag_repr);
                let member_offset = variant.payload_offset();
                let mut members = vec![];
                for member_witx0_9 in &variant.cases {
                    let member_name = member_witx0_9.name.as_str().to_string();
                    let member_type = match member_witx0_9.tref.as_ref() {
                        None => ASType::Void,
                        Some(type_witx0_9) => ASType::from(type_witx0_9),
                    };
                    let member = ASUnionMember {
                        name: member_name,
                        type_: Rc::new(member_type),
                    };
                    members.push(member);
                }
                let full_size = variant.mem_size();
                let tag_size = variant.tag_repr.mem_size();
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
            witx0_9::Type::List(items_tref) => {
                let elements_type = ASType::from(items_tref);
                match elements_type {
                    // The "string" keyword in witx0_9 returns a Char32, even if the actual encoding is expected to be UTF-8
                    ASType::Char32 | ASType::Char8 => ASType::String(Rc::new(ASType::Char8)),
                    _ => ASType::Slice(Rc::new(elements_type)),
                }
            }
        }
    }
}

impl From<&witx0_9::TypeRef> for ASType {
    fn from(witx0_9_tref: &witx0_9::TypeRef) -> Self {
        match witx0_9_tref {
            witx0_9::TypeRef::Value(type_witx0_9) => ASType::from(type_witx0_9.as_ref()),
            witx0_9::TypeRef::Name(alias_witx0_9) => {
                let alias_witx0_9 = alias_witx0_9.as_ref();
                let alias_name = alias_witx0_9.name.as_str().to_string();
                let alias_target = ASType::from(&alias_witx0_9.tref);
                ASType::Alias(ASAlias {
                    name: alias_name,
                    type_: Rc::new(alias_target),
                })
            }
        }
    }
}