#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub status: String,
    pub image: String,
}
