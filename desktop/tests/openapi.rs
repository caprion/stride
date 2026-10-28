#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The router and the OpenAPI document.
//!
//! The document is the one the Python app generates — `app.openapi()` — and
//! it is the contract the React app was written against. The router in
//! `stride-api` is the port of it, and this test is what says the two are the
//! same thing: the same paths, the same methods, the same path parameters,
//! and the same request bodies.
//!
//! The document is checked in, so this test does not need Python.

use std::sync::Arc;

use serde_json::{Map, Value};
use stride_api::{Api, CoreBackend, MemBackend};

/// The document the reference generates.
const DOCUMENT: &str = include_str!("fixtures/openapi.json");

/// The document, parsed.
fn document() -> Value {
    serde_json::from_str(DOCUMENT).expect("the OpenAPI document parses")
}

/// The router over an empty backend.
fn api() -> Api {
    let backend = Arc::new(CoreBackend::new(Arc::new(MemBackend::new())));
    Api::new(backend)
}

/// The paths and the methods of the document.
fn document_operations() -> Vec<(String, String)> {
    let mut operations = Vec::new();
    let document = document();
    let paths = document["paths"]
        .as_object()
        .expect("the paths of the document")
        .clone();
    for (path, methods) in paths {
        for method in methods.as_object().expect("the methods").keys() {
            operations.push((method.to_uppercase(), path.clone()));
        }
    }
    operations.sort();
    operations
}

/// Every path and method in the document is a route.
#[test]
fn the_router_covers_the_openapi_document() {
    let document = document();
    let paths = document["paths"]
        .as_object()
        .expect("the paths of the document");
    assert_eq!(
        paths.len(),
        62,
        "the reference has 62 paths, the document has {}",
        paths.len()
    );

    for (method, path) in document_operations() {
        let found = stride_api::router::ROUTES
            .iter()
            .find(|route| route.method == method && route.pattern == path);
        assert!(
            found.is_some(),
            "the route table is missing {method} {path}"
        );
    }
}

/// Every route in the router is in the document.
#[test]
fn the_openapi_document_covers_the_router() {
    let operations = document_operations();
    for route in stride_api::router::ROUTES {
        let key = (route.method.to_string(), route.pattern.to_string());
        assert!(
            operations.contains(&key),
            "the OpenAPI document is missing {} {}",
            route.method,
            route.pattern
        );
    }
}

/// The path parameters are the ones the document declares.
#[test]
fn the_path_parameters_match_the_document() {
    let document = document();
    let paths = document["paths"].as_object().expect("the paths");
    for (path, methods) in paths {
        for (method, operation) in methods.as_object().expect("the operations") {
            let method = method.to_uppercase();
            let route = stride_api::router::ROUTES
                .iter()
                .find(|route| route.method == method && route.pattern == path)
                .unwrap_or_else(|| panic!("the route table is missing {method} {path}"));
            for parameter in operation["parameters"]
                .as_array()
                .cloned()
                .unwrap_or_default()
            {
                if parameter["in"].as_str() != Some("path") {
                    continue;
                }
                let name = parameter["name"].as_str().unwrap_or_default();
                assert!(
                    route.pattern.contains(&format!("{{{name}}}")),
                    "the route {method} {path} does not carry the path parameter {name}"
                );
            }
        }
    }
}

/// The request bodies the document declares are the ones the router reads.
///
/// For every operation with a body, a body is built from the document's own
/// schema and handed to the router. The router must not answer with a
/// validation error: a shape the document promises is a shape the router
/// accepts.
#[test]
fn the_request_bodies_match_the_document() {
    let document = document();
    let api = api();
    let schemas = document["components"]["schemas"]
        .as_object()
        .cloned()
        .unwrap_or_default();
    let paths = document["paths"].as_object().cloned().unwrap_or_default();

    for (path, methods) in paths {
        for (method, operation) in methods.as_object().cloned().unwrap_or_default() {
            let method = method.to_uppercase();
            let Some(content) = operation["requestBody"]["content"].as_object() else {
                continue;
            };
            let Some(schema) = content
                .get("application/json")
                .map(|entry| entry["schema"].clone())
                .filter(|schema| schema.is_object())
            else {
                // The FIT import is a multipart form; the router takes the
                // bytes, not a JSON body.
                continue;
            };
            let body = example_body(&schema, &schemas);
            let rendered = serde_json::to_string(&body).expect("the body renders");
            let path = fill_path(&path);
            let answer = api.call(&method, &path, Some(&rendered));
            if let Err(error) = answer {
                assert_ne!(
                    error.status, 422,
                    "{method} {path} rejects the body the document promises: {error}"
                );
            }
        }
    }
}

/// Put a real value in every path parameter.
///
/// The document names the parameters; the router wants an integer where the
/// parameter is an id, and a tile coordinate where it is a map tile.
fn fill_path(pattern: &str) -> String {
    let mut out = String::with_capacity(pattern.len());
    let mut rest = pattern;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let Some(end) = rest[start..].find('}') else {
            out.push_str(&rest[start..]);
            return out;
        };
        let name = &rest[start + 1..start + end];
        let value = match name {
            "z" => "1",
            "x" => "2",
            "y" => "3",
            "kind" => "hr",
            other if other.contains("id") => "1",
            _ => "stride",
        };
        out.push_str(value);
        rest = &rest[start + end + 1..];
    }
    out.push_str(rest);
    out
}

/// A body that satisfies a schema: every property, with a value of its type.
fn example_body(schema: &Value, schemas: &Map<String, Value>) -> Value {
    let schema = resolve(schema, schemas);
    let mut body = Map::new();
    if let Some(properties) = schema["properties"].as_object() {
        for (name, property) in properties {
            body.insert(name.clone(), example_value(property, schemas));
        }
    }
    Value::Object(body)
}

/// A value of the type a property declares.
fn example_value(schema: &Value, schemas: &Map<String, Value>) -> Value {
    let schema = resolve(schema, schemas);
    if let Some(example) = schema.get("example") {
        return example.clone();
    }
    let kind = schema["type"].as_str().unwrap_or("string");
    match kind {
        "integer" => {
            let minimum = schema["minimum"].as_f64().unwrap_or(1.0);
            Value::from(minimum.max(1.0) as i64)
        }
        "number" => {
            let minimum = schema["minimum"].as_f64().unwrap_or(1.0);
            Value::from(minimum.max(1.0))
        }
        "boolean" => Value::Bool(true),
        "array" => Value::Array(Vec::new()),
        "object" => example_body(schema, schemas),
        _ => {
            if schema["anyOf"].is_array() {
                // The first branch of a union is enough: the router reads the
                // fields it knows.
                return example_value(&schema["anyOf"][0], schemas);
            }
            // A string that is long enough: the document says how long.
            let minimum = schema["minLength"].as_u64().unwrap_or(0) as usize;
            let maximum = schema["maxLength"].as_u64().unwrap_or(u64::MAX) as usize;
            let length = minimum.max(1).min(maximum);
            Value::String("x".repeat(length))
        }
    }
}

/// Resolve a `$ref` to the schema it names.
fn resolve<'a>(schema: &'a Value, schemas: &'a Map<String, Value>) -> &'a Value {
    if let Some(reference) = schema["$ref"].as_str() {
        let name = reference.rsplit('/').next().unwrap_or_default();
        if let Some(resolved) = schemas.get(name) {
            return resolved;
        }
    }
    schema
}
