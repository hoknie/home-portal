use std::collections::{BTreeMap, BTreeSet};

use home_portal::{registered, rule_book};
use syn::visit::Visit;
use syn::{Expr, ImplItem, Item, Lit};

use crate::sources::{SourceFile, rust_files};

pub const ROUTED_FUNCTIONS: [&str; 3] = ["router", "widgets_router", "assemble"];
pub const METHODS: [&str; 4] = ["get", "post", "put", "delete"];
pub const FEATURE_FOLDER: &str = "/src/features/";
pub const WIDGET_ROUTES: &str = "crates/core/portal-widget/src/controllers/widgets.rs";
pub const ASSEMBLY: &str = "bin/home-portal/src/boot/router.rs";

pub type Route = (String, String);

pub fn routed_file(path: &str) -> bool {
    let feature = path.starts_with("crates/") && path.contains(FEATURE_FOLDER);
    (feature || path == WIDGET_ROUTES || path == ASSEMBLY) && !path.ends_with("tests.rs")
}

pub fn constants_of(file: &syn::File) -> BTreeMap<String, String> {
    let mut found = BTreeMap::new();
    for item in &file.items {
        match item {
            Item::Const(constant) => {
                if let Some(text) = text_of(&constant.expr) {
                    found.insert(constant.ident.to_string(), text);
                }
            }
            Item::Impl(block) => {
                for inner in &block.items {
                    if let ImplItem::Const(constant) = inner
                        && let Some(text) = text_of(&constant.expr)
                    {
                        found.insert(constant.ident.to_string(), text);
                    }
                }
            }
            _ => {}
        }
    }
    found
}

fn text_of(expression: &Expr) -> Option<String> {
    match expression {
        Expr::Lit(literal) => match &literal.lit {
            Lit::Str(text) => Some(text.value()),
            _ => None,
        },
        _ => None,
    }
}

struct Routes<'a> {
    local: &'a BTreeMap<String, String>,
    shared: &'a BTreeMap<String, String>,
    inside: bool,
    found: Vec<Route>,
}

impl Routes<'_> {
    fn path_of(&self, expression: &Expr) -> Option<String> {
        match expression {
            Expr::Path(path) => {
                let name = path.path.segments.last()?.ident.to_string();
                let local = self.local.get(&name);
                let own = path.path.segments.first()?.ident == "Self";
                local
                    .or_else(|| (!own).then(|| self.shared.get(&name)).flatten())
                    .cloned()
            }
            other => text_of(other),
        }
    }

    fn enter(&mut self, name: &str, visit: impl FnOnce(&mut Self)) {
        let before = self.inside;
        self.inside = ROUTED_FUNCTIONS.contains(&name);
        visit(self);
        self.inside = before;
    }
}

fn methods_of(expression: &Expr) -> Vec<String> {
    match expression {
        Expr::Call(call) => match call.func.as_ref() {
            Expr::Path(path) => path
                .path
                .segments
                .last()
                .map(|segment| segment.ident.to_string())
                .filter(|name| METHODS.contains(&name.as_str()))
                .into_iter()
                .collect(),
            _ => Vec::new(),
        },
        Expr::MethodCall(call) => {
            let mut methods = methods_of(&call.receiver);
            let name = call.method.to_string();
            if METHODS.contains(&name.as_str()) {
                methods.push(name);
            }
            methods
        }
        _ => Vec::new(),
    }
}

impl<'syntax> Visit<'syntax> for Routes<'_> {
    fn visit_item_fn(&mut self, function: &'syntax syn::ItemFn) {
        self.enter(&function.sig.ident.to_string(), |this| {
            syn::visit::visit_item_fn(this, function);
        });
    }

    fn visit_impl_item_fn(&mut self, function: &'syntax syn::ImplItemFn) {
        self.enter(&function.sig.ident.to_string(), |this| {
            syn::visit::visit_impl_item_fn(this, function);
        });
    }

    fn visit_expr_method_call(&mut self, call: &'syntax syn::ExprMethodCall) {
        if self.inside && call.method == "route" && call.args.len() == 2 {
            let path = self.path_of(&call.args[0]);
            for method in methods_of(&call.args[1]) {
                let path = path.clone().unwrap_or_else(|| "<unresolved>".to_string());
                self.found.push((method.to_uppercase(), path));
            }
        }
        syn::visit::visit_expr_method_call(self, call);
    }
}

pub fn routes_in(files: &[SourceFile]) -> BTreeSet<Route> {
    let parsed: Vec<(&SourceFile, syn::File)> = files
        .iter()
        .filter_map(|file| syn::parse_file(&file.text).ok().map(|tree| (file, tree)))
        .collect();
    let shared: BTreeMap<String, String> = parsed
        .iter()
        .flat_map(|(_, tree)| constants_of(tree))
        .collect();
    let mut found = BTreeSet::new();
    for (_, tree) in parsed.iter().filter(|(file, _)| routed_file(&file.path)) {
        let local = constants_of(tree);
        let mut routes = Routes {
            local: &local,
            shared: &shared,
            inside: false,
            found: Vec::new(),
        };
        routes.visit_file(tree);
        found.extend(routes.found);
    }
    found
}

pub fn differences(routes: &BTreeSet<Route>, rules: &BTreeSet<Route>) -> Vec<String> {
    let mut found: Vec<String> = routes
        .difference(rules)
        .map(|(method, path)| format!("{method} {path} is protected and has no rule"))
        .collect();
    found.extend(
        rules
            .difference(routes)
            .map(|(method, path)| format!("{method} {path} has a rule and no route")),
    );
    found
}

fn declared_rules() -> BTreeSet<Route> {
    let directory = tempfile::tempdir().unwrap();
    let path = crate::support::with_extra(&directory, "secret", "");
    let registry = registered(&crate::support::wiring_for(&path)).unwrap();
    rule_book(&registry)
        .rules()
        .into_iter()
        .map(|(method, path, _)| (method.to_string(), path.to_string()))
        .collect()
}

#[test]
fn every_protected_route_has_a_rule_and_every_rule_a_route() {
    let routes = routes_in(&rust_files());
    assert!(routes.len() > 50, "only {} routes were found", routes.len());
    let found = differences(&routes, &declared_rules());
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
fn a_route_without_a_rule_is_named_with_its_method_and_path() {
    let file = SourceFile::sample(
        "crates/features/portal-sample/src/features/sample.rs",
        "impl Feature for Sample {\n    fn router(&self) -> Router {\n        Router::new().route(Self::ITEM, get(show).put(change))\n    }\n\n    fn public_router(&self) -> Router {\n        Router::new().route(OPEN, post(open))\n    }\n}\n\nimpl Sample {\n    pub const ITEM: &'static str = \"/api/sample/{id}\";\n}\n",
    );
    let routes = routes_in(&[file]);
    let rules = BTreeSet::from([("GET".to_string(), "/api/sample/{id}".to_string())]);
    assert_eq!(
        differences(&routes, &rules),
        vec!["PUT /api/sample/{id} is protected and has no rule"]
    );
}
