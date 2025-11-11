use serde::Deserialize;
use serde_json::Number;

#[derive(Deserialize, Debug)]
pub struct Ast {
    #[serde(rename = "metaVars")]
    pub global_names: GlobalNames,
    pub procedures: Vec<Procedure>,
    // TODO(mvp) add widgets
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GlobalNames {
    #[serde(rename = "globals")]
    pub global_vars: Vec<String>,
    pub turtle_vars: Vec<String>,
    pub patch_vars: Vec<String>,
    pub link_vars: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Procedure {
    pub name: String,
    #[serde(rename = "args")]
    pub arg_names: Vec<String>,
    pub return_type: ReturnType,
    pub agent_class: AgentClass,
    pub statements: Vec<Node>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ReturnType {
    Unit,
    Wildcard,
}

#[derive(Debug)]
pub struct AgentClass {
    pub observer: bool,
    pub turtle: bool,
    pub patch: bool,
    pub link: bool,
}

impl<'de> Deserialize<'de> for AgentClass {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        Ok(AgentClass {
            observer: s.chars().nth(0) == Some('O'),
            turtle: s.chars().nth(1) == Some('T'),
            patch: s.chars().nth(2) == Some('P'),
            link: s.chars().nth(3) == Some('L'),
        })
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "kebab-case", rename_all_fields = "camelCase")]
pub enum Node {
    CommandApp(CommandApp),
    ReporterCall(ReporterCall),
    ReporterProcCall { name: String, args: Vec<Node> },
    CommandBlock { statements: Vec<Node> },
    ReporterBlock { reporter_app: Box<Node> },
    LetBinding { var_name: String, value: Box<Node> },
    LetRef { name: String },
    ProcedureArgRef { name: String },
    Number { value: Number },
    String { value: String },
    List { items: Vec<Node> },
    Nobody,
}

type Bn = Box<Node>;

#[derive(Deserialize, Debug)]
#[serde(tag = "name", rename_all = "SCREAMING-KEBAB-CASE", content = "args")]
pub enum CommandApp {
    Stop([Bn; 0]),
    ClearAll([Bn; 0]),
    SetDefaultShape([Bn; 2]),
    CreateTurtles([Bn; 2]),
    Set([Bn; 2]),
    Fd([Bn; 1]),
    #[serde(rename = "LT")]
    Left([Bn; 1]),
    #[serde(rename = "RT")]
    Right([Bn; 1]),
    ResetTicks([Bn; 0]),
    Ask([Bn; 2]),
    If([Bn; 2]),
    #[serde(rename = "IFELSE")]
    IfElse([Bn; 3]),
    Diffuse([Bn; 2]),
    Tick([Bn; 0]),
    Report([Bn; 1]),
    #[serde(untagged)]
    UserProcCall {
        name: String,
        args: Vec<Node>,
    },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "name", rename_all = "SCREAMING-KEBAB-CASE", content = "args")]
pub enum ReporterCall {
    OneOf([Bn; 1]),
    Of([Bn; 2]),
    #[serde(rename = "<")]
    Lt([Bn; 2]),
    #[serde(rename = ">")]
    Gt([Bn; 2]),
    #[serde(rename = "=")]
    Eq([Bn; 2]),
    #[serde(rename = "<=")]
    Lte([Bn; 2]),
    #[serde(rename = ">=")]
    Gte([Bn; 2]),
    #[serde(rename = "-")]
    Sub([Bn; 2]),
    #[serde(rename = "+")]
    Add([Bn; 2]),
    #[serde(rename = "*")]
    Mul([Bn; 2]),
    #[serde(rename = "/")]
    Div([Bn; 2]),
    And([Bn; 2]),
    Or([Bn; 2]),
    Not([Bn; 1]),
    Distancexy([Bn; 2]),
    #[serde(rename = "CAN-MOVE?")]
    CanMove([Bn; 1]),
    PatchRightAndAhead([Bn; 2]),
    PatchLeftAndAhead([Bn; 2]),
    MaxPxcor([Bn; 0]),
    MaxPycor([Bn; 0]),
    ScaleColor([Bn; 4]),
    Ticks([Bn; 0]),
    Random([Bn; 1]),
    #[serde(untagged)]
    VarAccess {
        name: String,
    },
}
