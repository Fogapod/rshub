type Url = String;

#[derive(Debug)]
pub enum InstallationAction {
    Install(Url),
    Uninstall(Url),
    Delete(Url),
}

#[derive(Debug)]
pub struct Installation {
    fork: String,
    version: String,
    size: usize,
}
