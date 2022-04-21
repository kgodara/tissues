// use serde;
use serde::{ Serialize, Deserialize, Deserializer };
use serde_json::Value;


fn fallback_str_from_types<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum JSONValueTypes<'a> {
        Str(&'a str),
        I64(i64),
        Null,
    }

    Ok(match JSONValueTypes::deserialize(deserializer)? {
        JSONValueTypes::Str(v) => v.to_string(), // Ignoring parsing errors
        JSONValueTypes::I64(v) => v.to_string(),
        JSONValueTypes::Null => String::default(),
    })
}

pub fn label_node_extractor<'de, D>(deserializer: D) -> Result<Vec<Label>, D::Error>
    where D: serde::de::Deserializer<'de>,
{
    #[derive(Deserialize, Debug, )]
    struct Labels {
        pub nodes: Vec<LabelNode>,
    }

    #[derive(Deserialize, Debug, )]
    struct LabelNode {
        pub id: String,

        #[serde(default)]
        pub name: Option<String>,

        #[serde(default)]
        pub color: Option<String>,
    }

    let x = Labels::deserialize(deserializer)?;
    let mut result_vec: Vec<Label> = Vec::default();

    for label in x.nodes.iter() {
        result_vec.push(Label { id: label.id.clone(), name: label.name.clone(), color: label.color.clone() });
    }

    Ok(result_vec)
}

pub fn subscriber_node_extractor<'de, D>(deserializer: D) -> Result<Vec<User>, D::Error>
    where D: serde::de::Deserializer<'de>,
{
    #[derive(Deserialize, Debug, )]
    struct Subscribers {
        pub nodes: Vec<SubscriberNode>,
    }

    #[derive(Deserialize, Debug, )]
    struct SubscriberNode {
        pub id: String,

        #[serde(default)]
        pub name: Option<String>,

        #[serde(default)]
        pub display_name: Option<String>,
    }

    let x = Subscribers::deserialize(deserializer)?;
    let mut result_vec: Vec<User> = Vec::default();

    for subscriber in x.nodes.iter() {
        result_vec.push(User { id: subscriber.id.clone(), name: subscriber.name.clone(), display_name: subscriber.display_name.clone() });
    }

    Ok(result_vec)
}





#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Organization {
    pub id: String,
    pub name: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: String,

    #[serde(default)]
    pub name: Option<String>,

    #[serde(default)]
    pub key: Option<String>,
    
    #[serde(default)]
    pub description: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomView {

    pub id: String,
    pub name: String,

    #[serde(deserialize_with="fallback_str_from_types")]
    pub description: String,
    pub color: String,

    #[serde(rename="filterData")]
    pub filter_data: Value,
    pub filters: Value,

    #[serde(rename="organization")]
    pub org: Organization,
    pub team: Option<Team>,
}

impl Default for CustomView {
    fn default() -> CustomView {
        CustomView {
            id: String::default(),
            name: String::default(),
            description: String::default(),
            color: String::default(),
            filter_data: Value::Null,
            filters: Value::Null,
            org: Organization::default(),
            team: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    pub id: String,

    pub name: String,

    #[serde(rename="type")]
    pub state_type: String,

    pub color: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,

    #[serde(default)]
    pub name: Option<String>,

    #[serde(rename="displayName")]
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,

    #[serde(default)]
    pub name: Option<String>,

    #[serde(default)]
    pub color: Option<String>,

    #[serde(default)]
    pub state: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Cycle {
    pub id: String,
    pub name: Option<String>,
    pub number: i64,

    #[serde(rename="startsAt")]
    pub starts_at: String,

    #[serde(rename="endsAt")]
    pub ends_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub id: String,

    #[serde(default)]
    pub name: Option<String>,

    #[serde(default)]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IssueRelatableObject {
    WorkflowState(WorkflowState),
    Assignee(User),
    Project(Project),
    Cycle(Cycle),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,

    #[serde(rename="createdAt")]
    pub created_at: String,

    pub number: i64,

    #[serde(rename="dueDate")]
    pub due_date: Option<String>,

    pub title: String,
    pub description: Option<String>,
    pub priority: i64,
    pub estimate: Option<String>,

    pub team: Team,

    pub state: WorkflowState,

    pub creator: Option<User>,

    pub assignee: Option<User>,


    #[serde(deserialize_with="label_node_extractor")]
    pub labels: Vec<Label>,

    #[serde(default)]
    pub cycle: Cycle,

    pub project: Option<Project>,

    #[serde(deserialize_with="subscriber_node_extractor")]
    #[serde(default)]
    pub subscribers: Vec<User>,
}