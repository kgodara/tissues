

pub struct LinearTeamSelectState {
    teams: serde_json::Value::Array,
}

impl LinearTeamSelectState {

    fn load(&self) {

    }

    fn 

}

impl Default for LinearTeamSelectState {
    fn default() -> LinearTeamSelectState {
        LinearTeamSelectState {
            teams: serde_json::Value::Array(vec![])
        }
    }
}