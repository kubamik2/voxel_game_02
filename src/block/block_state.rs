use hashbrown::HashMap;

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, Default, PartialEq)]
pub struct BlockState(HashMap<String, Value>);
 
impl BlockState {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get(&self, state_name: &str) -> Option<&Value> {
        self.0.get(state_name)
    }
}


#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub enum Value {
    Number(serde_json::Number),
    // I32(i32),
    // U32(u32),
    // F32(f32),
    String(String),
    Struct(HashMap<String, Value>),
    // Bytes(Box<[u8]>),
    Bool(bool)
}
