use crate::Key;

#[derive(Debug, Clone)]
pub struct Binding<K: Key> {
    pub key: K,
    pub description: &'static str,
}

#[derive(Debug, Clone)]
pub struct BindingGroup<K: Key> {
    pub category: String,
    pub bindings: Vec<Binding<K>>,
}

#[derive(Debug, Clone)]
pub struct DisplayBinding<K, C> {
    pub key: K,
    pub description: &'static str,
    pub category: C,
}

#[derive(Debug, Clone)]
pub enum NodeResult<K: Key, A> {
    Branch { children: Vec<Binding<K>> },
    Leaf { action: A },
}
