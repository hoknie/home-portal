use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use schemars::generate::SchemaSettings;
use serde::Serialize;

use crate::{WRITE_VARIABLE, check};

const SCHEMAS: &str = "web/src/shared/api/json-schemas";

fn schema_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(SCHEMAS)
        .join(format!("{name}.schema.json"))
}

pub fn typed<T: Serialize + JsonSchema>(name: &str, value: &T) {
    check(name, serde_json::to_value(value).unwrap());
    let generator = SchemaSettings::draft2020_12()
        .for_serialize()
        .into_generator();
    let schema = serde_json::to_value(generator.into_root_schema_for::<T>()).unwrap();
    let text = format!("{}\n", serde_json::to_string_pretty(&schema).unwrap());
    let path = schema_path(name);
    if env::var(WRITE_VARIABLE).as_deref() == Ok("write") {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, &text).unwrap();
        return;
    }
    let on_disk = fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(
        on_disk,
        text,
        "{} is out of date with the serializers; run `just samples`",
        path.display()
    );
}
