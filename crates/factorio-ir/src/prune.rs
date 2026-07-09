use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    block::Block,
    expression::Expression,
    function::Function,
    module::Module,
    statement::Statement,
    structure::Struct,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ItemKey {
    Function(String),
    Struct(String),
    StructMethod(String, String),
    StructConstant(String, String),
}

#[derive(Debug, Default)]
struct ModuleReachability {
    items: HashSet<ItemKey>,
}

struct ModuleGraph<'a> {
    modules: &'a [Module],
    by_name: HashMap<&'a str, &'a Module>,
    children: HashMap<&'a str, Vec<&'a str>>,
}

impl<'a> ModuleGraph<'a> {
    fn new(modules: &'a [Module]) -> Self {
        let by_name = modules
            .iter()
            .map(|module| (module.name.as_str(), module))
            .collect::<HashMap<_, _>>();
        let mut children = HashMap::<&str, Vec<&str>>::new();
        for module in modules {
            for child in &module.submodules {
                if let Some(child_module) = by_name.get(child.as_str()) {
                    children
                        .entry(module.name.as_str())
                        .or_default()
                        .push(child_module.name.as_str());
                }
            }
        }
        Self {
            modules,
            by_name,
            children,
        }
    }

    fn get(&self, name: &str) -> Option<&'a Module> {
        self.by_name.get(name).copied()
    }

    fn child_modules(&self, name: &str) -> &[&'a str] {
        self.children
            .get(name)
            .map_or(&[], Vec::as_slice)
    }
}

/// Remove unreachable functions and exports from transpiled modules.
pub fn prune_modules(modules: &mut [Module]) {
    if modules.is_empty() {
        return;
    }

    let graph = ModuleGraph::new(modules);
    let reachability = compute_reachability(&graph);

    for module in modules {
        if let Some(reach) = reachability.get(&module.name) {
            prune_module(module, reach);
        }
    }
}

fn compute_reachability(graph: &ModuleGraph<'_>) -> HashMap<String, ModuleReachability> {
    let mut reachability = graph
        .modules
        .iter()
        .map(|module| (module.name.clone(), ModuleReachability::default()))
        .collect::<HashMap<_, _>>();

    let mut pending = VecDeque::new();

    for module in graph.modules {
        for symbol in &module.symbols {
            if let Statement::FunctionDecl(function) = &symbol.statement
                && function.event.is_some()
            {
                enqueue_item(
                    &mut reachability,
                    &mut pending,
                    &module.name,
                    ItemKey::Function(function.name.clone()),
                );
            }
        }
    }

    while let Some((module_name, item)) = pending.pop_front() {
        let Some(module) = graph.get(&module_name) else {
            continue;
        };

        match &item {
            ItemKey::Function(name) => {
                if let Some(function) = find_function(module, name) {
                    collect_references_from_function(
                        graph,
                        module,
                        function,
                        &mut reachability,
                        &mut pending,
                    );
                }
            }
            ItemKey::Struct(name) => {
                if let Some(struct_decl) = find_struct(module, name) {
                    for (constant, value) in &struct_decl.constants {
                        enqueue_item(
                            &mut reachability,
                            &mut pending,
                            &module_name,
                            ItemKey::StructConstant(name.clone(), constant.clone()),
                        );
                        collect_references_from_expression(
                            graph,
                            module,
                            value,
                            &HashMap::new(),
                            &mut reachability,
                            &mut pending,
                        );
                    }
                    for method in &struct_decl.methods {
                        enqueue_item(
                            &mut reachability,
                            &mut pending,
                            &module_name,
                            ItemKey::StructMethod(name.clone(), method.name.clone()),
                        );
                    }
                }
            }
            ItemKey::StructMethod(struct_name, method_name) => {
                if let Some(method) = find_struct_method(module, struct_name, method_name) {
                    collect_references_from_function(
                        graph,
                        module,
                        method,
                        &mut reachability,
                        &mut pending,
                    );
                } else if let Some((owner_module, method)) =
                    find_struct_method_in_module_tree(graph, &module_name, struct_name, method_name)
                {
                    enqueue_item(
                        &mut reachability,
                        &mut pending,
                        &owner_module,
                        ItemKey::StructMethod(struct_name.clone(), method_name.clone()),
                    );
                    if let Some(owner) = graph.get(&owner_module) {
                        collect_references_from_function(
                            graph,
                            owner,
                            method,
                            &mut reachability,
                            &mut pending,
                        );
                    }
                }
            }
            ItemKey::StructConstant(struct_name, constant_name) => {
                if let Some(value) = find_struct_constant(module, struct_name, constant_name) {
                    collect_references_from_expression(
                        graph,
                        module,
                        value,
                        &HashMap::new(),
                        &mut reachability,
                        &mut pending,
                    );
                } else if let Some((owner_module, value)) =
                    find_struct_constant_in_module_tree(
                        graph,
                        &module_name,
                        struct_name,
                        constant_name,
                    )
                {
                    enqueue_item(
                        &mut reachability,
                        &mut pending,
                        &owner_module,
                        ItemKey::StructConstant(struct_name.clone(), constant_name.clone()),
                    );
                    if let Some(owner) = graph.get(&owner_module) {
                        collect_references_from_expression(
                            graph,
                            owner,
                            value,
                            &HashMap::new(),
                            &mut reachability,
                            &mut pending,
                        );
                    }
                }
            }
        }
    }

    reachability
}

fn enqueue_item(
    reachability: &mut HashMap<String, ModuleReachability>,
    pending: &mut VecDeque<(String, ItemKey)>,
    module_name: &str,
    item: ItemKey,
) {
    let reach = reachability
        .entry(module_name.to_string())
        .or_default();
    if reach.items.insert(item.clone()) {
        pending.push_back((module_name.to_string(), item));
    }
}

fn prune_module(module: &mut Module, reach: &ModuleReachability) {
    module.body.statements.retain(|statement| is_statement_reachable(statement, reach));
    module
        .symbols
        .retain(|symbol| is_statement_reachable(&symbol.statement, reach));

    for statement in module
        .body
        .statements
        .iter_mut()
        .chain(module.symbols.iter_mut().map(|symbol| &mut symbol.statement))
    {
        if let Statement::StructDecl(struct_decl) = statement {
            prune_struct(struct_decl, reach);
        }
    }
}

fn is_statement_reachable(statement: &Statement, reach: &ModuleReachability) -> bool {
    match statement {
        Statement::FunctionDecl(function) => {
            reach.items.contains(&ItemKey::Function(function.name.clone()))
        }
        Statement::StructDecl(struct_decl) => is_struct_reachable(struct_decl, reach),
        Statement::VariableDecl { .. }
        | Statement::Assignment { .. }
        | Statement::Conditional { .. }
        | Statement::Return(_)
        | Statement::Expr(_) => true,
    }
}

fn is_struct_reachable(struct_decl: &Struct, reach: &ModuleReachability) -> bool {
    reach.items.contains(&ItemKey::Struct(struct_decl.name.clone()))
        || struct_decl.methods.iter().any(|method| {
            reach.items.contains(&ItemKey::StructMethod(
                struct_decl.name.clone(),
                method.name.clone(),
            ))
        })
        || struct_decl.constants.iter().any(|(name, _)| {
            reach.items.contains(&ItemKey::StructConstant(
                struct_decl.name.clone(),
                name.clone(),
            ))
        })
}

fn prune_struct(struct_decl: &mut Struct, reach: &ModuleReachability) {
    struct_decl.constants.retain(|(name, _)| {
        reach.items.contains(&ItemKey::StructConstant(
            struct_decl.name.clone(),
            name.clone(),
        ))
    });
    struct_decl.methods.retain(|method| {
        reach.items.contains(&ItemKey::StructMethod(
            struct_decl.name.clone(),
            method.name.clone(),
        ))
    });
}

fn collect_references_from_function(
    graph: &ModuleGraph<'_>,
    module: &Module,
    function: &Function,
    reachability: &mut HashMap<String, ModuleReachability>,
    pending: &mut VecDeque<(String, ItemKey)>,
) {
    let mut locals = HashMap::new();
    for parameter in &function.params {
        if parameter.name != "self"
            && let Some(source_type) = &parameter.source_type
            && let Some(struct_name) = type_name_from_source(source_type)
        {
            locals.insert(parameter.name.clone(), struct_name);
        }
    }

    collect_references_from_block(
        graph,
        module,
        &function.body,
        &mut locals,
        reachability,
        pending,
    );
}

fn collect_references_from_block(
    graph: &ModuleGraph<'_>,
    module: &Module,
    block: &Block,
    locals: &mut HashMap<String, String>,
    reachability: &mut HashMap<String, ModuleReachability>,
    pending: &mut VecDeque<(String, ItemKey)>,
) {
    for statement in &block.statements {
        collect_references_from_statement(
            graph,
            module,
            statement,
            locals,
            reachability,
            pending,
        );
    }
}

fn collect_references_from_statement(
    graph: &ModuleGraph<'_>,
    module: &Module,
    statement: &Statement,
    locals: &mut HashMap<String, String>,
    reachability: &mut HashMap<String, ModuleReachability>,
    pending: &mut VecDeque<(String, ItemKey)>,
) {
    match statement {
        Statement::FunctionDecl(function) => {
            collect_references_from_function(graph, module, function, reachability, pending);
        }
        Statement::StructDecl(_) => {}
        Statement::VariableDecl {
            name,
            source_type,
            value,
            ..
        } => {
            if let Some(source_type) = source_type
                && let Some(struct_name) = type_name_from_source(source_type)
            {
                locals.insert(name.clone(), struct_name);
            } else if let Some(struct_name) = infer_struct_type_from_expression(value) {
                locals.insert(name.clone(), struct_name);
            }
            collect_references_from_expression(
                graph,
                module,
                value,
                locals,
                reachability,
                pending,
            );
        }
        Statement::Assignment { target, value } => {
            collect_references_from_expression(
                graph,
                module,
                target,
                locals,
                reachability,
                pending,
            );
            collect_references_from_expression(
                graph,
                module,
                value,
                locals,
                reachability,
                pending,
            );
        }
        Statement::Conditional {
            condition,
            then_block,
            else_block,
        } => {
            collect_references_from_expression(
                graph,
                module,
                condition,
                locals,
                reachability,
                pending,
            );
            for statement in then_block {
                collect_references_from_statement(
                    graph,
                    module,
                    statement,
                    locals,
                    reachability,
                    pending,
                );
            }
            for statement in else_block {
                collect_references_from_statement(
                    graph,
                    module,
                    statement,
                    locals,
                    reachability,
                    pending,
                );
            }
        }
        Statement::Return(value) => {
            if let Some(value) = value {
                collect_references_from_expression(
                    graph,
                    module,
                    value,
                    locals,
                    reachability,
                    pending,
                );
            }
        }
        Statement::Expr(expression) => {
            collect_references_from_expression(
                graph,
                module,
                expression,
                locals,
                reachability,
                pending,
            );
        }
    }
}

fn collect_references_from_expression(
    graph: &ModuleGraph<'_>,
    module: &Module,
    expression: &Expression,
    locals: &HashMap<String, String>,
    reachability: &mut HashMap<String, ModuleReachability>,
    pending: &mut VecDeque<(String, ItemKey)>,
) {
    match expression {
        Expression::Literal(_) | Expression::Identifier(_) => {}
        Expression::QualifiedPath { segments } => {
            resolve_struct_member_reference(
                graph,
                module,
                segments,
                reachability,
                pending,
            );
        }
        Expression::FieldAccess { base, field } => {
            if let Expression::Identifier(name) = base.as_ref() {
                if let Some((target_module, struct_name)) = resolve_import(module, name) {
                    enqueue_item(
                        reachability,
                        pending,
                        &target_module,
                        ItemKey::Struct(struct_name.clone()),
                    );
                    enqueue_item(
                        reachability,
                        pending,
                        &target_module,
                        ItemKey::StructMethod(struct_name, field.clone()),
                    );
                } else if let Some(struct_name) = locals.get(name) {
                    queue_struct_member(
                        graph,
                        module,
                        struct_name,
                        field,
                        reachability,
                        pending,
                    );
                } else {
                    queue_struct_member(graph, module, name, field, reachability, pending);
                }
            } else {
                collect_references_from_expression(
                    graph,
                    module,
                    base,
                    locals,
                    reachability,
                    pending,
                );
            }
        }
        Expression::Call { func, args } => {
            resolve_call_target(graph, module, func, locals, reachability, pending);
            for arg in args {
                collect_references_from_expression(
                    graph,
                    module,
                    arg,
                    locals,
                    reachability,
                    pending,
                );
            }
        }
        Expression::MethodCall {
            receiver,
            method,
            args,
        } => {
            if let Expression::Identifier(name) = receiver.as_ref() {
                if let Some((target_module, struct_name)) = resolve_import(module, name) {
                    enqueue_item(
                        reachability,
                        pending,
                        &target_module,
                        ItemKey::StructMethod(struct_name, method.clone()),
                    );
                } else if let Some(struct_name) = locals.get(name) {
                    let owner = struct_owner_module(graph, module, struct_name);
                    enqueue_item(
                        reachability,
                        pending,
                        &owner,
                        ItemKey::StructMethod(struct_name.clone(), method.clone()),
                    );
                }
            } else {
                collect_references_from_expression(
                    graph,
                    module,
                    receiver,
                    locals,
                    reachability,
                    pending,
                );
            }
            for arg in args {
                collect_references_from_expression(
                    graph,
                    module,
                    arg,
                    locals,
                    reachability,
                    pending,
                );
            }
        }
        Expression::StructLiteral { fields } => {
            for (_, value) in fields {
                collect_references_from_expression(
                    graph,
                    module,
                    value,
                    locals,
                    reachability,
                    pending,
                );
            }
        }
        Expression::BinaryOp { lhs, rhs, .. } => {
            collect_references_from_expression(
                graph, module, lhs, locals, reachability, pending,
            );
            collect_references_from_expression(
                graph, module, rhs, locals, reachability, pending,
            );
        }
        Expression::FormatConcat { parts } => {
            for part in parts {
                collect_references_from_expression(
                    graph,
                    module,
                    part,
                    locals,
                    reachability,
                    pending,
                );
            }
        }
    }
}

fn resolve_call_target(
    graph: &ModuleGraph<'_>,
    module: &Module,
    func: &Expression,
    locals: &HashMap<String, String>,
    reachability: &mut HashMap<String, ModuleReachability>,
    pending: &mut VecDeque<(String, ItemKey)>,
) {
    match func {
        Expression::Identifier(name) => {
            enqueue_item(
                reachability,
                pending,
                &module.name,
                ItemKey::Function(name.clone()),
            );
        }
        Expression::FieldAccess { base, field } => {
            if let Expression::Identifier(name) = base.as_ref() {
                if let Some((target_module, struct_name)) = resolve_import(module, name) {
                    enqueue_item(
                        reachability,
                        pending,
                        &target_module,
                        ItemKey::Struct(struct_name.clone()),
                    );
                    enqueue_item(
                        reachability,
                        pending,
                        &target_module,
                        ItemKey::StructMethod(struct_name, field.clone()),
                    );
                } else if let Some(struct_name) = locals.get(name) {
                    let owner = struct_owner_module(graph, module, struct_name);
                    enqueue_item(
                        reachability,
                        pending,
                        &owner,
                        ItemKey::StructMethod(struct_name.clone(), field.clone()),
                    );
                } else {
                    queue_struct_member(graph, module, name, field, reachability, pending);
                }
            } else {
                collect_references_from_expression(
                    graph,
                    module,
                    base,
                    locals,
                    reachability,
                    pending,
                );
            }
        }
        Expression::QualifiedPath { segments } => {
            if let Some((target_module, rest)) = resolve_module_path(module, segments) {
                enqueue_import_path(reachability, pending, &target_module, &rest);
            } else if segments.len() >= 2 {
                let struct_name = segments[0].clone();
                let member = segments[1].clone();
                if function_exists(module, &member) && segments.len() == 2 {
                    enqueue_item(
                        reachability,
                        pending,
                        &module.name,
                        ItemKey::Function(member),
                    );
                } else {
                    queue_struct_member(
                        graph,
                        module,
                        &struct_name,
                        &member,
                        reachability,
                        pending,
                    );
                }
            }
        }
        _ => collect_references_from_expression(
            graph,
            module,
            func,
            locals,
            reachability,
            pending,
        ),
    }
}

fn resolve_struct_member_reference(
    graph: &ModuleGraph<'_>,
    module: &Module,
    segments: &[String],
    reachability: &mut HashMap<String, ModuleReachability>,
    pending: &mut VecDeque<(String, ItemKey)>,
) {
    if segments.is_empty() {
        return;
    }

    if segments.len() == 1 {
        let name = &segments[0];
        if let Some((target_module, struct_name)) = resolve_import(module, name) {
            enqueue_item(
                reachability,
                pending,
                &target_module,
                ItemKey::Struct(struct_name),
            );
        } else if struct_exists(module, name) {
            enqueue_item(
                reachability,
                pending,
                &module.name,
                ItemKey::Struct(name.clone()),
            );
        } else if function_exists(module, name) {
            enqueue_item(
                reachability,
                pending,
                &module.name,
                ItemKey::Function(name.clone()),
            );
        }
        return;
    }

    let first = &segments[0];
    if let Some((target_module, rest)) = resolve_module_path(module, segments) {
        enqueue_import_path(reachability, pending, &target_module, &rest);
        return;
    }

    if let Some((target_module, struct_name)) = resolve_import(module, first) {
        enqueue_item(
            reachability,
            pending,
            &target_module,
            ItemKey::Struct(struct_name.clone()),
        );
        enqueue_item(
            reachability,
            pending,
            &target_module,
            ItemKey::StructMethod(struct_name, segments[1].clone()),
        );
        return;
    }

    queue_struct_member(
        graph,
        module,
        first,
        &segments[1],
        reachability,
        pending,
    );
}

fn queue_struct_member(
    graph: &ModuleGraph<'_>,
    module: &Module,
    struct_name: &str,
    member: &str,
    reachability: &mut HashMap<String, ModuleReachability>,
    pending: &mut VecDeque<(String, ItemKey)>,
) {
    enqueue_item(
        reachability,
        pending,
        &module.name,
        ItemKey::Struct(struct_name.to_string()),
    );

    if struct_has_constant(module, struct_name, member) {
        enqueue_item(
            reachability,
            pending,
            &module.name,
            ItemKey::StructConstant(struct_name.to_string(), member.to_string()),
        );
        return;
    }

    if struct_has_method(module, struct_name, member) {
        enqueue_item(
            reachability,
            pending,
            &module.name,
            ItemKey::StructMethod(struct_name.to_string(), member.to_string()),
        );
        return;
    }

    if function_exists(module, member) {
        enqueue_item(
            reachability,
            pending,
            &module.name,
            ItemKey::Function(member.to_string()),
        );
        return;
    }

    for child in graph.child_modules(&module.name) {
        if let Some(child_module) = graph.get(child)
            && child_module.is_imported_type_extension(&Struct {
                name: struct_name.to_string(),
                fields: vec![],
                constants: vec![],
                methods: vec![],
                doc: None,
                debug: None,
            })
        {
            if struct_has_constant(child_module, struct_name, member) {
                enqueue_item(
                    reachability,
                    pending,
                    &child_module.name,
                    ItemKey::StructConstant(struct_name.to_string(), member.to_string()),
                );
            } else if struct_has_method(child_module, struct_name, member) {
                enqueue_item(
                    reachability,
                    pending,
                    &child_module.name,
                    ItemKey::StructMethod(struct_name.to_string(), member.to_string()),
                );
            }
        }
    }
}

fn resolve_import(module: &Module, local: &str) -> Option<(String, String)> {
    for import in &module.imports {
        for item in &import.items {
            if item.local == local {
                return Some((import.module.clone(), item.name.clone()));
            }
        }
    }
    None
}

fn resolve_module_path(module: &Module, segments: &[String]) -> Option<(String, Vec<String>)> {
    if segments.is_empty() {
        return None;
    }

    let import = module
        .imports
        .iter()
        .find(|import| import.local == segments[0])?;
    Some((import.module.clone(), segments[1..].to_vec()))
}

fn enqueue_import_path(
    reachability: &mut HashMap<String, ModuleReachability>,
    pending: &mut VecDeque<(String, ItemKey)>,
    target_module: &str,
    rest: &[String],
) {
    if rest.is_empty() {
        return;
    }

    let struct_name = rest[0].clone();
    enqueue_item(
        reachability,
        pending,
        target_module,
        ItemKey::Struct(struct_name.clone()),
    );

    if rest.len() >= 2 {
        enqueue_item(
            reachability,
            pending,
            target_module,
            ItemKey::StructMethod(struct_name, rest[1].clone()),
        );
    }
}

fn infer_struct_type_from_expression(expression: &Expression) -> Option<String> {
    match expression {
        Expression::Call { func, .. } => infer_struct_type_from_call(func),
        Expression::MethodCall { receiver, .. } => match receiver.as_ref() {
            Expression::Identifier(name) => Some(name.clone()),
            Expression::QualifiedPath { segments } if !segments.is_empty() => {
                Some(segments[0].clone())
            }
            _ => None,
        },
        _ => None,
    }
}

fn infer_struct_type_from_call(func: &Expression) -> Option<String> {
    match func {
        Expression::QualifiedPath { segments } if segments.len() >= 2 => {
            Some(segments[segments.len() - 2].clone())
        }
        Expression::FieldAccess { base, .. } => match base.as_ref() {
            Expression::Identifier(name) => Some(name.clone()),
            Expression::QualifiedPath { segments } if !segments.is_empty() => {
                Some(segments[0].clone())
            }
            _ => None,
        },
        _ => None,
    }
}

fn type_name_from_source(source_type: &str) -> Option<String> {
    source_type
        .rsplit("::")
        .next()
        .map(str::to_string)
        .filter(|name| !name.is_empty())
}

fn function_exists(module: &Module, name: &str) -> bool {
    find_function(module, name).is_some()
}

fn struct_exists(module: &Module, name: &str) -> bool {
    find_struct(module, name).is_some()
}

fn find_function<'a>(module: &'a Module, name: &str) -> Option<&'a Function> {
    module
        .body
        .statements
        .iter()
        .chain(module.symbols.iter().map(|symbol| &symbol.statement))
        .find_map(|statement| match statement {
            Statement::FunctionDecl(function) if function.name == name => Some(function),
            _ => None,
        })
}

fn find_struct<'a>(module: &'a Module, name: &str) -> Option<&'a Struct> {
    module
        .body
        .statements
        .iter()
        .chain(module.symbols.iter().map(|symbol| &symbol.statement))
        .find_map(|statement| match statement {
            Statement::StructDecl(struct_decl) if struct_decl.name == name => Some(struct_decl),
            _ => None,
        })
}

fn find_struct_method<'a>(
    module: &'a Module,
    struct_name: &str,
    method_name: &str,
) -> Option<&'a Function> {
    find_struct(module, struct_name).and_then(|struct_decl| {
        struct_decl
            .methods
            .iter()
            .find(|method| method.name == method_name)
    })
}

fn find_struct_constant<'a>(
    module: &'a Module,
    struct_name: &str,
    constant_name: &str,
) -> Option<&'a Expression> {
    find_struct(module, struct_name).and_then(|struct_decl| {
        struct_decl
            .constants
            .iter()
            .find_map(|(name, value)| (name == constant_name).then_some(value))
    })
}

fn struct_has_constant(module: &Module, struct_name: &str, constant_name: &str) -> bool {
    find_struct_constant(module, struct_name, constant_name).is_some()
}

fn struct_has_method(module: &Module, struct_name: &str, method_name: &str) -> bool {
    find_struct_method(module, struct_name, method_name).is_some()
}

fn struct_owner_module(graph: &ModuleGraph<'_>, module: &Module, struct_name: &str) -> String {
    if struct_exists(module, struct_name) {
        return module.name.clone();
    }

    for import in &module.imports {
        for item in &import.items {
            if item.name == struct_name || item.local == struct_name {
                return import.module.clone();
            }
        }

        if module_defines_struct(graph, &import.module, struct_name) {
            return import.module.clone();
        }
    }

    module.name.clone()
}

fn module_defines_struct(graph: &ModuleGraph<'_>, module_name: &str, struct_name: &str) -> bool {
    let mut stack = vec![module_name.to_string()];
    let mut seen = HashSet::new();

    while let Some(current) = stack.pop() {
        if !seen.insert(current.clone()) {
            continue;
        }

        if let Some(module) = graph.get(&current)
            && struct_exists(module, struct_name)
        {
            return true;
        }

        if let Some(module) = graph.get(&current) {
            for child in graph.child_modules(&module.name) {
                stack.push((*child).to_string());
            }
        }
    }

    false
}

fn find_struct_method_in_module_tree<'a>(
    graph: &ModuleGraph<'a>,
    module_name: &str,
    struct_name: &str,
    method_name: &str,
) -> Option<(String, &'a Function)> {
    let mut stack = vec![module_name.to_string()];
    let mut seen = HashSet::new();

    while let Some(current) = stack.pop() {
        if !seen.insert(current.clone()) {
            continue;
        }

        let Some(module) = graph.get(&current) else {
            continue;
        };

        if let Some(method) = find_struct_method(module, struct_name, method_name) {
            return Some((current, method));
        }

        for child in graph.child_modules(&current) {
            stack.push((*child).to_string());
        }
    }

    None
}

fn find_struct_constant_in_module_tree<'a>(
    graph: &ModuleGraph<'a>,
    module_name: &str,
    struct_name: &str,
    constant_name: &str,
) -> Option<(String, &'a Expression)> {
    let mut stack = vec![module_name.to_string()];
    let mut seen = HashSet::new();

    while let Some(current) = stack.pop() {
        if !seen.insert(current.clone()) {
            continue;
        }

        let Some(module) = graph.get(&current) else {
            continue;
        };

        if let Some(value) = find_struct_constant(module, struct_name, constant_name) {
            return Some((current, value));
        }

        for child in graph.child_modules(&current) {
            stack.push((*child).to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use crate::{
        block::Block,
        function::Function,
        module::{Module, Symbol},
        scope::Scope,
        stage::Stage,
        statement::Statement,
    };

    use super::prune_modules;

    #[test]
    fn prunes_unreachable_private_functions() {
        let mut modules = vec![Module {
            name: "control".to_string(),
            stage: Stage::Control,
            body: Block {
                statements: vec![Statement::FunctionDecl(Function {
                    name: "add".to_string(),
                    params: vec![],
                    body: Block { statements: vec![] },
                    doc: None,
                    debug: None,
                    event: None,
                })],
            },
            imports: vec![],
            submodules: vec![],
            symbols: vec![Symbol {
                scope: Scope::Public,
                statement: Statement::FunctionDecl(Function {
                    name: "on_init".to_string(),
                    params: vec![],
                    body: Block { statements: vec![] },
                    doc: None,
                    debug: None,
                    event: Some("on_init".to_string()),
                }),
            }],
        }];

        prune_modules(&mut modules);

        assert!(modules[0].body.statements.is_empty());
        assert_eq!(modules[0].symbols.len(), 1);
        assert_eq!(
            match &modules[0].symbols[0].statement {
                Statement::FunctionDecl(function) => function.name.as_str(),
                _ => panic!("expected function"),
            },
            "on_init"
        );
    }

    #[test]
    fn prunes_unused_public_exports() {
        let mut modules = vec![Module {
            name: "control".to_string(),
            stage: Stage::Control,
            body: Block { statements: vec![] },
            imports: vec![],
            submodules: vec![],
            symbols: vec![
                Symbol {
                    scope: Scope::Public,
                    statement: Statement::FunctionDecl(Function {
                        name: "unused".to_string(),
                        params: vec![],
                        body: Block { statements: vec![] },
                        doc: None,
                        debug: None,
                        event: None,
                    }),
                },
                Symbol {
                    scope: Scope::Public,
                    statement: Statement::FunctionDecl(Function {
                        name: "on_init".to_string(),
                        params: vec![],
                        body: Block { statements: vec![] },
                        doc: None,
                        debug: None,
                        event: Some("on_init".to_string()),
                    }),
                },
            ],
        }];

        prune_modules(&mut modules);

        assert_eq!(modules[0].symbols.len(), 1);
    }
}
