use crate::ast::{block::Block, literal::Literal, operator::Operator};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum MethodDispatch {
    #[default]
    Infer,
    /// User / metatable / `data:extend`-style: `recv:method(args)`.
    Colon,
    /// Factorio `LuaObject`: attribute reads, setters, and `.method(args)`.
    Factorio,
    /// `storage[key]`
    StorageGet,
    /// `storage[key] = value`
    StorageSet,
    /// Mod-settings: `recv[key].value` (or `recv[key]` for `.setting`).
    SettingsGet,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    QualifiedPath {
        segments: Vec<String>,
    },
    FieldAccess {
        base: Box<Self>,
        field: String,
    },
    Call {
        func: Box<Self>,
        args: Vec<Self>,
    },
    MethodCall {
        receiver: Box<Self>,
        method: String,
        args: Vec<Self>,
        dispatch: MethodDispatch,
    },
    StructLiteral {
        /// The Rust struct name that produced this literal, used by codegen to inject
        /// fixed Factorio prototype fields (e.g. `type = "bool-setting"`).
        struct_name: Option<String>,
        fields: Vec<(String, Self)>,
    },
    /// Tagged enum value `{ tag = "Variant", ...payload }`.
    EnumLiteral {
        enum_name: String,
        variant: String,
        /// Named fields, or `_1` / `_2` / ... for tuple variants.
        fields: Vec<(String, Self)>,
    },
    /// An operation between a `lhs` and a `rhs` with an [`Operator`]
    BinaryOp {
        lhs: Box<Self>,
        op: Operator,
        rhs: Box<Self>,
    },
    /// String interpolation parts joined with `..` in Lua.
    FormatConcat {
        parts: Vec<Self>,
    },
    /// Lua array literal `{ a, b, c }`.
    Array {
        elements: Vec<Self>,
    },
    /// Lua table index expression `base[key]`.
    Index {
        base: Box<Self>,
        key: Box<Self>,
    },
    /// Logical `not EXPR` in Lua.
    Not(Box<Self>),
    /// Length operator `#EXPR` in Lua.
    Len(Box<Self>),
    /// Safe if-expression (avoids falsey `and`/`or` pitfalls).
    If {
        condition: Box<Self>,
        then_expr: Box<Self>,
        else_expr: Box<Self>,
    },
    /// Anonymous Lua function value (`function(params) ... end`).
    Closure {
        params: Vec<String>,
        body: Block,
    },
    /// Fat pointer for `dyn Trait`: `{ _data = ..., _vt = __vt_Trait_Concrete }`.
    FatPointer {
        data: Box<Self>,
        /// Fully qualified vtable symbol name, e.g. `__vt_Display_Point`.
        vtable: String,
    },
    /// Dynamic dispatch: `recv._vt.method(recv, args...)`.
    DynMethodCall {
        receiver: Box<Self>,
        method: String,
        args: Vec<Self>,
    },
}

impl Expression {
    /// Build a [`MethodCall`] with [`MethodDispatch::Infer`].
    #[must_use]
    pub fn method_call(receiver: Self, method: impl Into<String>, args: Vec<Self>) -> Self {
        Self::MethodCall {
            receiver: Box::new(receiver),
            method: method.into(),
            args,
            dispatch: MethodDispatch::Infer,
        }
    }

    /// Build a [`MethodCall`] with an explicit dispatch tag.
    #[must_use]
    pub fn method_call_with(
        receiver: Self,
        method: impl Into<String>,
        args: Vec<Self>,
        dispatch: MethodDispatch,
    ) -> Self {
        Self::MethodCall {
            receiver: Box::new(receiver),
            method: method.into(),
            args,
            dispatch,
        }
    }
}
