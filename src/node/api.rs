use crate::error::{UnicomError, UnicomErrorKind};
use hyper::Method;
use serde_json::{Map, Value};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Api{
    pub id: u64,
    pub name: String,
    pub methods: Vec<ApiMethod>
}

impl Api {
    pub fn new(id: u64, name: &str, methods: Vec<ApiMethod>) -> Api{
        Api{
            id,
            name: name.to_owned(),
            methods,
        }
    }

    pub fn get_method(&self, method: &MethodKind) -> Result<&ApiMethod, UnicomError>{
        for api_method in &self.methods{
            if api_method.method == *method {
                return Ok(api_method)
            }
        }
        Err(UnicomError::new(UnicomErrorKind::MethodNotAllowed, &format!("Methode {:?} not allowed", method)))
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiMethod{
    pub method: MethodKind,
    pub parameters: Vec<Parameter>,
}

impl ApiMethod {
    pub fn new(method: MethodKind, parameters: Vec<Parameter>) -> ApiMethod{
        ApiMethod { method, parameters }
    }

    pub fn generate_parameters(&self, parameters: &Map<String, Value>) -> Result<(), UnicomError>{
        for parameter in &self.parameters{
            match parameters.get(&parameter.name){
                Some(value) => {
                    if !parameter.check(value){
                        return Err(UnicomError::new(UnicomErrorKind::ParameterInvalid, &format!("Wrong type of parameter {} {:?}", &parameter.name, value)))
                    }
                },
                None => {
                    if parameter.mandatory{
                        return Err(UnicomError::new(UnicomErrorKind::ParameterInvalid, &format!("parameter {} is missing", &parameter.name)))
                    }
                },
            }
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Parameter{
    pub name: String,
    pub kind: ValueKind,
    pub mandatory: bool,
}

impl Parameter{
    pub fn new(name: &str, kind: ValueKind, mandatory: bool) -> Parameter{
        Parameter{
            name: name.to_owned(),
            kind,
            mandatory,
        }
    }

    fn check(&self, value: &Value) -> bool{
        match self.kind {
            ValueKind::Integer => value.is_i64(),
            ValueKind::Float => value.is_f64(),
            ValueKind::String => value.is_string(),
            ValueKind::Url(_) => value.is_string(),
            ValueKind::Input => true,
            ValueKind::SessionID => value.is_string(),
            ValueKind::Session(_) => true,
            ValueKind::User => true,
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub enum MethodKind{
    GET,
    PUT,
    POST,
    DELETE,
}

impl From<Method> for MethodKind{
    fn from(m: Method) -> Self {
        return m.as_str().into()
    }
}

impl From<&str> for MethodKind{
    fn from(m: &str) -> Self {
        match m {
            "GET" => MethodKind::GET,
            "PUT" => MethodKind::PUT,
            "POST" => MethodKind::POST,
            "DELETE" => MethodKind::DELETE,
            _ => MethodKind::GET
        }
    }
}

impl From<String> for MethodKind{
    fn from(m: String) -> Self {
        m.as_str().into()
    }
}

impl From<MethodKind> for &str{
    fn from(m: MethodKind) -> Self {
        match m{
            MethodKind::GET => "GET",
            MethodKind::PUT => "PUT",
            MethodKind::POST => "POST",
            MethodKind::DELETE => "DELETE",
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ValueKind{
    Integer,
    Float,
    String,
    Url(usize),
    SessionID,
    Session(String),
    User,
    Input,
}

impl From<&str> for ValueKind{
    fn from(m: &str) -> Self {
        match &m[..3] {
            "int" => ValueKind::Integer,
            "str" => ValueKind::String,
            "flt" => ValueKind::Float,
            "ipt" => ValueKind::Input,
            "sid" => ValueKind::SessionID,
            "usr" => ValueKind::User,
            "ses" => {
                let v :Vec<&str> = m.split("_").collect();
                if v.len() <= 1{
                    return ValueKind::String
                }
                ValueKind::Session(v[1].to_string())
            }
            "url" => {
                let v :Vec<&str> = m.split("_").collect();
                if v.len() <= 1{
                    return ValueKind::String
                }
                if let Ok(value) = v[1].parse(){
                    return ValueKind::Url(value)
                }
                return ValueKind::String
            }
            _ => {
                return ValueKind::String
            }
        }
    }
}
