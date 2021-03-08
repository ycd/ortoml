use std::{collections::HashMap, str::from_utf8};

use pyo3::wrap_pyfunction;
use pyo3::{prelude::*, types};
use toml;

#[pymodule]
fn ortoml(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(dumps, m)?).unwrap();
    m.add_function(wrap_pyfunction!(loads, m)?).unwrap();
    m.add_function(wrap_pyfunction!(to_json, m)?).unwrap();
    m.add_function(wrap_pyfunction!(from_json, m)?).unwrap();

    Ok(())
}

// Converts dictionary to YAML.
#[pyfunction]
fn dumps(dict: &pyo3::types::PyDict) -> PyResult<String> {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let orjson = PyModule::import(py, "orjson").unwrap();

    let dict_string = orjson.call1("dumps", (dict,))?.str()?.to_str()?.as_bytes();

    // This is used to convert python's b'' to => "".
    // TODO(ycd): find something more safe(i think it is safe enough) but quite sure not the best practice though.
    let ss = std::str::from_utf8(&dict_string[2..dict_string.iter().count() - 1])?;

    let toml_str: toml::Value = match serde_json::from_str(&ss) {
        Ok(v) => v,
        Err(why) => panic!("an error occured: {}", why),
    };

    Ok(toml_str.to_string())
}

// Converts TOML object to dictionary.
#[pyfunction]
fn loads(s: &str) -> PyResult<pyo3::PyObject> {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let r: HashMap<String, toml::Value> = toml::de::from_str(s).unwrap();
    println!("{:?}", r);

    let dict = pyo3::types::PyDict::new(py);
    dict.set_item("abc", "abc");
    Ok(PyObject::from(dict))
}

#[pyfunction]
fn to_json(s: &str) -> String {
    let json = convert_to_json(toml::de::from_str(s).unwrap());
    let v = serde_json::to_string(&json).unwrap();
    v
}

#[pyfunction]
fn from_json(s: &str) -> String {
    let v: toml::Value = serde_json::from_str(&s).unwrap();
    let toml_text: String = v.to_string();
    toml_text
}

// convert_to_json is  used by to_json function
// to convert toml types to json.
fn convert_to_json(toml: toml::Value) -> serde_json::Value {
    match toml {
        toml::Value::String(s) => serde_json::Value::String(s),
        toml::Value::Integer(i) => serde_json::Value::Number(i.into()),
        toml::Value::Float(f) => {
            let n = serde_json::Number::from_f64(f).expect("float infinite and nan not allowed");
            serde_json::Value::Number(n)
        }
        toml::Value::Boolean(b) => serde_json::Value::Bool(b),
        toml::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(convert_to_json).collect())
        }
        toml::Value::Table(table) => serde_json::Value::Object(
            table
                .into_iter()
                .map(|(k, v)| (k, convert_to_json(v)))
                .collect(),
        ),
        toml::Value::Datetime(dt) => serde_json::Value::String(dt.to_string()),
    }
}

#[test]
fn test_loads() {
    let v = crate::loads(
        r#"
        [dependencies]
        a = "123"
        b = 3
        c = true

        [naber]
        abi = 123

    "#,
    );
}

#[test]
fn test_to_json() {
    let v = crate::to_json(
        r#"
        [dependencies]
        a = "123"
        b = 3
        c = true

        [naber]
        abi = 123

    "#,
    );

    assert_eq!(
        v,
        "{\"dependencies\":{\"a\":\"123\",\"b\":3,\"c\":true},\"naber\":{\"abi\":123}}"
    )
}

#[test]
fn test_from_json() {
    let v = crate::from_json(
        "{\"dependencies\":{\"a\":\"123\",\"b\":3,\"c\":true},\"naber\":{\"abi\":123}}",
    );

    assert_eq!(
        "[dependencies]\na = \"123\"\nb = 3\nc = true\n\n[naber]\nabi = 123\n",
        v
    )
}

#[test]
fn test_dumps() {
    use pyo3::prelude::*;
    use pyo3::types::PyDict;
    let gil = Python::acquire_gil();
    let py = gil.python();

    let dict = pyo3::types::PyDict::new(py);
    dict.set_item("test", "value");

    let new_dict = PyDict::new(py);
    new_dict.set_item("test1", "valuefortest1");
    dict.set_item("dict_inside_dict", new_dict);
    dict.set_item("test1", "valueueueueue");
    dict.set_item("test2", "another value");
    dict.set_item("test3", "yet another value");

    let v = crate::dumps(dict);
    println!("{:#?}", dict)
}
