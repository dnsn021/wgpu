#![cfg(feature = "wgsl-in")]

use naga::valid::{Capabilities, ValidationFlags, Validator};

const ENABLE: &str = "enable atomic_vec2u_min_max;";
const BUFFER: &str = "@group(0) @binding(0) var<storage, read_write> a: atomic<vec2u>;";
const NESTED_TYPES: &str = "
    struct AtomicLeaf {
        ordinary: u32,
        value: atomic<vec2u>,
    }
    struct AtomicNested {
        leaves: array<AtomicLeaf, 2>,
    }
";

fn validate(source: &str, capabilities: Capabilities) -> bool {
    let Ok(module) = naga::front::wgsl::parse_str(source) else {
        return false;
    };
    Validator::new(ValidationFlags::all(), capabilities)
        .validate(&module)
        .is_ok()
}

fn nested_source() -> String {
    format!(
        "{ENABLE}
        enable wgpu_binding_array;
        {NESTED_TYPES}
        @group(0) @binding(0)
        var<storage, read_write> data: array<AtomicNested, 2>;
        @group(0) @binding(1)
        var<storage, read_write> bindings: binding_array<AtomicNested, 2>;

        @compute @workgroup_size(1)
        fn main() {{
            let ordinary = data[0].leaves[0].ordinary;
            data[0].leaves[1].ordinary = ordinary;
            atomicStoreMin(&data[0].leaves[0].value, vec2u(1));
            atomicStoreMax(&bindings[0].leaves[1].value, vec2u(2));
        }}"
    )
}

#[test]
fn atomic_vec2u_feature_is_independent() {
    let source = format!("{ENABLE} {BUFFER} @compute @workgroup_size(1) fn main() {{ atomicStoreMin(&a, vec2(1, 2)); atomicStoreMax(&a, vec2u(3, 4)); }}");
    assert!(validate(&source, Capabilities::ATOMIC_VEC2U_MIN_MAX));
    assert!(!validate(&source, Capabilities::empty()));
    assert!(!validate(
        &source,
        Capabilities::SHADER_INT64 | Capabilities::SHADER_INT64_ATOMIC_MIN_MAX
    ));
    assert!(!validate(&source.replace(ENABLE, ""), Capabilities::all()));
    assert!(!validate(
        &format!("{ENABLE} const x = 1u64;"),
        Capabilities::ATOMIC_VEC2U_MIN_MAX
    ));

    let mut frontend = naga::front::wgsl::Frontend::new_with_options(naga::front::wgsl::Options {
        capabilities: Capabilities::empty(),
        ..naga::front::wgsl::Options::new()
    });
    assert!(frontend.parse(ENABLE).is_err());
    assert!(naga::front::wgsl::parse_str("requires atomic_vec2u_min_max;").is_ok());
}

#[test]
fn atomic_vec2u_rejects_other_operations() {
    for operation in [
        "atomicLoad(&a);",
        "atomicStore(&a, vec2u(1));",
        "atomicMin(&a, vec2u(1));",
        "atomicMax(&a, vec2u(1));",
        "atomicAdd(&a, vec2u(1));",
        "atomicSub(&a, vec2u(1));",
        "atomicAnd(&a, vec2u(1));",
        "atomicOr(&a, vec2u(1));",
        "atomicXor(&a, vec2u(1));",
        "atomicExchange(&a, vec2u(1));",
        "atomicCompareExchangeWeak(&a, vec2u(0), vec2u(1));",
        "let x = a;",
        "a = vec2u(1);",
        "let x = atomicStoreMin(&a, vec2u(1));",
        "atomicStoreMax(&a, 1u);",
        "atomicStoreMin(&a, vec3u(1));",
        "atomicStoreMin(&a, vec2i(1));",
        "atomicStoreMax(&a, vec2f(1));",
    ] {
        let source =
            format!("{ENABLE} {BUFFER} @compute @workgroup_size(1) fn main() {{ {operation} }}");
        assert!(
            !validate(&source, Capabilities::all()),
            "accepted {operation}"
        );
    }
    for ty in ["u32", "i32", "vec2u", "atomic<u32>", "atomic<i32>"] {
        let source = format!("{ENABLE} @group(0) @binding(0) var<storage, read_write> a: {ty}; @compute @workgroup_size(1) fn main() {{ atomicStoreMin(&a, vec2u(1)); }}");
        assert!(!validate(&source, Capabilities::all()), "accepted {ty}");
    }
    for ty in ["vec2i", "vec2f", "vec3u", "vec4u"] {
        assert!(!validate(
            &format!("{ENABLE} alias A = atomic<{ty}>;"),
            Capabilities::all()
        ));
    }
}

#[test]
fn atomic_vec2u_restricts_address_space_and_stage() {
    for (space, access) in [
        ("workgroup", None),
        ("private", None),
        ("uniform", None),
        ("storage", Some("read")),
        ("storage", Some("write")),
    ] {
        let variable_space = match access {
            Some(access) => format!("{space}, {access}"),
            None => space.to_string(),
        };
        let binding = if matches!(space, "uniform" | "storage") {
            "@group(0) @binding(0)"
        } else {
            ""
        };
        for ty in [
            "atomic<vec2u>",
            "array<atomic<vec2u>, 2>",
            "AtomicLeaf",
            "AtomicNested",
            "array<AtomicNested, 2>",
        ] {
            let source =
                format!("{ENABLE} {NESTED_TYPES} {binding} var<{variable_space}> a: {ty};");
            assert!(
                !validate(&source, Capabilities::all()),
                "accepted var<{variable_space}> {ty}"
            );

            let pointer = match access {
                Some(access) => format!("ptr<{space}, {ty}, {access}>"),
                None => format!("ptr<{space}, {ty}>"),
            };
            let source = format!("{ENABLE} {NESTED_TYPES} alias P = {pointer};");
            assert!(
                !validate(&source, Capabilities::all()),
                "accepted {pointer}"
            );
        }
    }
    let source = format!("{ENABLE} {BUFFER} fn update() {{ atomicStoreMax(&a, vec2u(1)); }} @vertex fn main() -> @builtin(position) vec4f {{ update(); return vec4f(0); }}");
    assert!(!validate(&source, Capabilities::all()));
    assert!(validate(
        &format!("{ENABLE} {BUFFER} @fragment fn main() {{ atomicStoreMin(&a, vec2u(1)); }}"),
        Capabilities::ATOMIC_VEC2U_MIN_MAX
    ));
}

#[test]
fn atomic_vec2u_nested_types_preserve_ordinary_member_access() {
    assert!(validate(
        &nested_source(),
        Capabilities::ATOMIC_VEC2U_MIN_MAX | Capabilities::STORAGE_BUFFER_BINDING_ARRAY
    ));
}

#[test]
fn atomic_vec2u_ir_restrictions_propagate_through_nested_types() {
    let module = naga::front::wgsl::parse_str(&nested_source()).unwrap();
    let nested_ty = module
        .types
        .iter()
        .find_map(|(handle, ty)| (ty.name.as_deref() == Some("AtomicNested")).then_some(handle))
        .unwrap();
    let data = module
        .global_variables
        .iter()
        .find_map(|(handle, var)| (var.name.as_deref() == Some("data")).then_some(handle))
        .unwrap();
    let bindings = module
        .global_variables
        .iter()
        .find_map(|(handle, var)| (var.name.as_deref() == Some("bindings")).then_some(handle))
        .unwrap();

    let mut valid = module.clone();
    valid.types.insert(
        naga::Type {
            name: None,
            inner: naga::TypeInner::Pointer {
                base: nested_ty,
                space: naga::AddressSpace::Storage {
                    access: naga::StorageAccess::LOAD | naga::StorageAccess::STORE,
                },
            },
        },
        naga::Span::default(),
    );
    Validator::new(ValidationFlags::all(), Capabilities::all())
        .validate(&valid)
        .unwrap();

    for space in [
        naga::AddressSpace::WorkGroup,
        naga::AddressSpace::Storage {
            access: naga::StorageAccess::LOAD,
        },
        naga::AddressSpace::Storage {
            access: naga::StorageAccess::LOAD
                | naga::StorageAccess::STORE
                | naga::StorageAccess::ATOMIC,
        },
    ] {
        let mut invalid = module.clone();
        invalid.types.insert(
            naga::Type {
                name: None,
                inner: naga::TypeInner::Pointer {
                    base: nested_ty,
                    space,
                },
            },
            naga::Span::default(),
        );
        let error = Validator::new(ValidationFlags::all(), Capabilities::all())
            .validate(&invalid)
            .unwrap_err();
        assert!(matches!(
            error.as_inner(),
            naga::valid::ValidationError::Type {
                source: naga::valid::TypeError::InvalidAtomicVec2UAddressSpace(actual),
                ..
            } if *actual == space
        ));
    }

    for (global, space) in [
        (data, naga::AddressSpace::WorkGroup),
        (
            data,
            naga::AddressSpace::Storage {
                access: naga::StorageAccess::LOAD
                    | naga::StorageAccess::STORE
                    | naga::StorageAccess::ATOMIC,
            },
        ),
        (
            bindings,
            naga::AddressSpace::Storage {
                access: naga::StorageAccess::LOAD,
            },
        ),
    ] {
        let mut invalid = module.clone();
        invalid.global_variables[global].space = space;
        let error = Validator::new(ValidationFlags::all(), Capabilities::all())
            .validate(&invalid)
            .unwrap_err();
        assert!(matches!(
            error.as_inner(),
            naga::valid::ValidationError::GlobalVariable {
                source: naga::valid::GlobalVariableError::InvalidType(actual),
                ..
            } if *actual == space
        ));
    }
}

#[test]
fn atomic_vec2u_ir_validation() {
    let source = format!("{ENABLE} {BUFFER} @compute @workgroup_size(1) fn main() {{ atomicStoreMin(&a, vec2u(1)); }}");
    let module = naga::front::wgsl::parse_str(&source).unwrap();
    for fun in [
        naga::AtomicFunction::Add,
        naga::AtomicFunction::Subtract,
        naga::AtomicFunction::And,
        naga::AtomicFunction::ExclusiveOr,
        naga::AtomicFunction::InclusiveOr,
        naga::AtomicFunction::Exchange { compare: None },
    ] {
        let mut invalid = module.clone();
        let statement = invalid.entry_points[0]
            .function
            .body
            .iter_mut()
            .find(|s| matches!(s, naga::Statement::Atomic { .. }))
            .unwrap();
        let naga::Statement::Atomic { fun: operation, .. } = statement else {
            unreachable!();
        };
        *operation = fun;
        assert!(Validator::new(ValidationFlags::all(), Capabilities::all())
            .validate(&invalid)
            .is_err());
    }

    let mut invalid = module.clone();
    let statement = invalid.entry_points[0]
        .function
        .body
        .iter_mut()
        .find(|s| matches!(s, naga::Statement::Atomic { .. }))
        .unwrap();
    let naga::Statement::Atomic {
        fun, value, result, ..
    } = statement
    else {
        unreachable!();
    };
    *fun = naga::AtomicFunction::Exchange {
        compare: Some(*value),
    };
    *result = None;
    assert!(Validator::new(ValidationFlags::all(), Capabilities::all())
        .validate(&invalid)
        .is_err());

    let mut invalid = module.clone();
    let result_ty = invalid.types.insert(
        naga::Type {
            name: None,
            inner: naga::TypeInner::Vector {
                size: naga::VectorSize::Bi,
                scalar: naga::Scalar::U32,
            },
        },
        naga::Span::default(),
    );
    let result = invalid.entry_points[0].function.expressions.append(
        naga::Expression::AtomicResult {
            ty: result_ty,
            comparison: false,
        },
        naga::Span::default(),
    );
    let statement = invalid.entry_points[0]
        .function
        .body
        .iter_mut()
        .find(|s| matches!(s, naga::Statement::Atomic { .. }))
        .unwrap();
    let naga::Statement::Atomic {
        result: atomic_result,
        ..
    } = statement
    else {
        unreachable!();
    };
    *atomic_result = Some(result);
    let error = Validator::new(ValidationFlags::all(), Capabilities::all())
        .validate(&invalid)
        .unwrap_err();
    assert!(matches!(
        error.as_inner(),
        naga::valid::ValidationError::EntryPoint {
            source: naga::valid::EntryPointError::Function(
                naga::valid::FunctionError::InvalidAtomic(_)
            ),
            ..
        }
    ));

    let mut invalid = module.clone();
    let statement = invalid.entry_points[0]
        .function
        .body
        .iter_mut()
        .find(|s| matches!(s, naga::Statement::Atomic { .. }))
        .unwrap();
    let naga::Statement::Atomic { pointer, value, .. } = *statement else {
        unreachable!();
    };
    *statement = naga::Statement::Store { pointer, value };

    assert!(Validator::new(ValidationFlags::all(), Capabilities::all())
        .validate(&invalid)
        .is_err());

    let mut invalid = module.clone();
    let function = &mut invalid.entry_points[0].function;
    let statement = function
        .body
        .iter_mut()
        .find(|s| matches!(s, naga::Statement::Atomic { .. }))
        .unwrap();
    let naga::Statement::Atomic { pointer, value, .. } = *statement else {
        unreachable!();
    };
    function.expressions[value] = naga::Expression::Load { pointer };
    *statement = naga::Statement::Return { value: None };
    let error = Validator::new(ValidationFlags::all(), Capabilities::all())
        .validate(&invalid)
        .unwrap_err();
    assert!(matches!(
        error.as_inner(),
        naga::valid::ValidationError::EntryPoint {
            source: naga::valid::EntryPointError::Function(
                naga::valid::FunctionError::Expression {
                    source: naga::valid::ExpressionError::InvalidPointerType(_),
                    ..
                }
            ),
            ..
        }
    ));
}

#[test]
fn atomic_vec2u_ir_rejects_composite_load_and_store() {
    let source = format!(
        "{ENABLE} {NESTED_TYPES}
        @group(0) @binding(0)
        var<storage, read_write> data: array<AtomicNested, 2>;
        @compute @workgroup_size(1)
        fn main() {{
            atomicStoreMin(&data[0].leaves[0].value, vec2u(1));
        }}"
    );
    let module = naga::front::wgsl::parse_str(&source).unwrap();
    let data = module
        .global_variables
        .iter()
        .find_map(|(handle, var)| (var.name.as_deref() == Some("data")).then_some(handle))
        .unwrap();
    let data_pointer = module.entry_points[0]
        .function
        .expressions
        .iter()
        .find_map(|(handle, expression)| {
            matches!(*expression, naga::Expression::GlobalVariable(global) if global == data)
                .then_some(handle)
        })
        .unwrap();

    let mut invalid = module.clone();
    let statement = invalid.entry_points[0]
        .function
        .body
        .iter()
        .find(|statement| matches!(statement, naga::Statement::Atomic { .. }))
        .unwrap();
    let naga::Statement::Atomic { value, .. } = *statement else {
        unreachable!();
    };
    invalid.entry_points[0].function.expressions[value] = naga::Expression::Load {
        pointer: data_pointer,
    };
    let statement = invalid.entry_points[0]
        .function
        .body
        .iter_mut()
        .find(|statement| matches!(statement, naga::Statement::Atomic { .. }))
        .unwrap();
    *statement = naga::Statement::Return { value: None };
    let error = Validator::new(ValidationFlags::all(), Capabilities::all())
        .validate(&invalid)
        .unwrap_err();
    assert!(matches!(
        error.as_inner(),
        naga::valid::ValidationError::EntryPoint {
            source: naga::valid::EntryPointError::Function(
                naga::valid::FunctionError::Expression {
                    source: naga::valid::ExpressionError::InvalidPointerType(_),
                    ..
                }
            ),
            ..
        }
    ));

    let mut invalid = module.clone();
    let statement = invalid.entry_points[0]
        .function
        .body
        .iter_mut()
        .find(|statement| matches!(statement, naga::Statement::Atomic { .. }))
        .unwrap();
    let naga::Statement::Atomic { value, .. } = *statement else {
        unreachable!();
    };
    *statement = naga::Statement::Store {
        pointer: data_pointer,
        value,
    };
    let error = Validator::new(ValidationFlags::all(), Capabilities::all())
        .validate(&invalid)
        .unwrap_err();
    assert!(matches!(
        error.as_inner(),
        naga::valid::ValidationError::EntryPoint {
            source: naga::valid::EntryPointError::Function(
                naga::valid::FunctionError::InvalidStoreTypes { .. }
            ),
            ..
        }
    ));
}

#[test]
fn atomic_vec2u_layout() {
    let source = format!("{ENABLE} alias AtomicArray = array<atomic<vec2u>, 2>; struct S {{ pad: u32, a: atomic<vec2u>, tail: u32 }} @group(0) @binding(0) var<storage, read_write> a: array<S>;");
    let module = naga::front::wgsl::parse_str(&source).unwrap();
    let mut layouter = naga::proc::Layouter::default();
    layouter.update(module.to_ctx()).unwrap();
    let atomic = module
        .types
        .iter()
        .find_map(|(handle, ty)| (ty.inner == naga::TypeInner::AtomicVec2U32).then_some(handle))
        .unwrap();
    assert_eq!(layouter[atomic].size, 8);
    assert_eq!(
        layouter[atomic].alignment,
        naga::proc::Alignment::from_width(8)
    );

    let atomic_array = module
        .types
        .iter()
        .find_map(|(handle, ty)| {
            matches!(ty.inner, naga::TypeInner::Array { base, .. } if base == atomic)
                .then_some(handle)
        })
        .unwrap();
    let naga::TypeInner::Array { stride, .. } = module.types[atomic_array].inner else {
        unreachable!();
    };
    assert_eq!(stride, 8);

    let structure = module
        .types
        .iter()
        .find_map(|(handle, ty)| (ty.name.as_deref() == Some("S")).then_some(handle))
        .unwrap();
    let naga::TypeInner::Struct { ref members, span } = module.types[structure].inner else {
        unreachable!();
    };
    assert_eq!(members[1].offset, 8);
    assert_eq!(members[2].offset, 16);
    assert_eq!(span, 24);
}
