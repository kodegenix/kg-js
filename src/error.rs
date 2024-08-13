
//FIXME embed javascript error type
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct JsError(String);

impl std::fmt::Display for JsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error: {}", self.0)
    }
}

impl std::error::Error for JsError {}

impl From<String> for JsError {
    fn from(s: String) -> Self {
        JsError(s)
    }
}

impl Into<String> for JsError {
    fn into(self) -> String {
        self.0
    }
}
