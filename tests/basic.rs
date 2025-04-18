use serde_envfile::{from_str, to_string, Value};

#[test]
fn derive_serde() {
    //* Given
    // Test struct with serde derive
    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct TestEnvFile {
        hello: String,
    }

    let input = "HELLO=\"WORLD\"";

    //* When
    // Serialize and deserialize
    let de_output = from_str::<TestEnvFile>(input).expect("Failed to deserialize");
    let ser_output = to_string(&de_output).expect("Failed to serialize");

    //* Then
    assert_eq!(input, ser_output);
}

#[test]
fn value_serde() {
    //* Given
    let mut env = Value::new();
    env.insert("hello".into(), "world".into());

    //* When
    // Serialize and deserialize
    let ser_output = to_string(&env).expect("Failed to serialize");
    let de_output = from_str::<Value>(&ser_output).expect("Failed to deserialize");

    //* Then
    assert_eq!(env, de_output);
}
