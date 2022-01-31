use lazy_static::lazy_static;
use std::collections::HashMap;


// map view ids to expected issue ids
lazy_static! {
    pub static ref VIEW_ANSWER_KEY: HashMap<&'static str, Vec<&'static str>> = {
        let mut m: HashMap<&str, Vec<&str>> = HashMap::new();

        // SelectedPriority
        m.insert("cf9db35e-5eb9-475a-8ae0-8a130821ead0", vec![
            "f8a8309b-bc06-4c87-a19f-282aa7bff614",
            "eea7a43a-e4d3-4ebf-8f0d-e6538f4f5ae3",
            "9ec131be-f97e-4110-b9f1-39a3b61723ef",
            "1872404c-c3be-4938-a259-43ba8deb511e",
            "13bfaa1c-4cf0-40d6-9d70-6c76b948261b",
        ]);

        // SelectedProject
        m.insert("c0c7c852-5f4c-4a57-8a55-a306d86368f6", vec![
            "a1d909f3-21c1-4654-af2b-5a3daadd5ee3",
            "f8a8309b-bc06-4c87-a19f-282aa7bff614",
            "9ec131be-f97e-4110-b9f1-39a3b61723ef",
            "1872404c-c3be-4938-a259-43ba8deb511e",
        ]);

        // SelectedTeam
        m.insert("5a8a4fa5-cdae-4a62-bcf2-bc69e14fdeb2", vec![
            "ae0b7451-c176-4bec-a09a-f52e824ead11"
        ]);

        // SelectedCreator
        m.insert("5895b38b-d98c-4898-815c-97f166de3316", vec![
            "a1d909f3-21c1-4654-af2b-5a3daadd5ee3",
            "ae0b7451-c176-4bec-a09a-f52e824ead11",
            "ace38e69-8a64-46f8-ad57-dc70c61f5599",
        ]);

        // SelectedAssignee
        m.insert("1477aacd-465c-49d3-9e14-a3b7952f4e22", vec![
            "a1d909f3-21c1-4654-af2b-5a3daadd5ee3"
        ]);


        // Due Date Views

        // OverDue
        m.insert("52719a63-d7aa-4f1b-8157-91103ba51e0f", vec![
            "a1d909f3-21c1-4654-af2b-5a3daadd5ee3",
            "ae0b7451-c176-4bec-a09a-f52e824ead11",
            "eea7a43a-e4d3-4ebf-8f0d-e6538f4f5ae3",
            "63234ed0-035a-4aec-914b-19802c066b5a",
            "13bfaa1c-4cf0-40d6-9d70-6c76b948261b",
        ]);

        // NoDueDate
        m.insert("3dfa04a4-ce78-45cd-882b-866774faee50", vec![
            "ace38e69-8a64-46f8-ad57-dc70c61f5599",
            "f8a8309b-bc06-4c87-a19f-282aa7bff614",
            "9ec131be-f97e-4110-b9f1-39a3b61723ef",
            "1872404c-c3be-4938-a259-43ba8deb511e",
        ]);

        // DueDateBefore
        m.insert("ee372cb9-6e3d-4da4-b7b7-003013293491", vec![
            "a1d909f3-21c1-4654-af2b-5a3daadd5ee3",
            "ae0b7451-c176-4bec-a09a-f52e824ead11",
            "eea7a43a-e4d3-4ebf-8f0d-e6538f4f5ae3",
            "63234ed0-035a-4aec-914b-19802c066b5a",
            "13bfaa1c-4cf0-40d6-9d70-6c76b948261b",
        ]);

        // DueDateAfter
        m.insert("2a19d661-73ca-4208-8fe0-5f3554892a60", vec![
            "a1d909f3-21c1-4654-af2b-5a3daadd5ee3",
            "ae0b7451-c176-4bec-a09a-f52e824ead11",
            "eea7a43a-e4d3-4ebf-8f0d-e6538f4f5ae3",
            "63234ed0-035a-4aec-914b-19802c066b5a",
            "13bfaa1c-4cf0-40d6-9d70-6c76b948261b",
        ]);



        // Workflow State Views

        // SelectedState
        m.insert("aa09c686-9668-4104-87fc-58cdfea6fb8b", vec![
            "eea7a43a-e4d3-4ebf-8f0d-e6538f4f5ae3",
            "9ec131be-f97e-4110-b9f1-39a3b61723ef",
            "63234ed0-035a-4aec-914b-19802c066b5a",
            "13bfaa1c-4cf0-40d6-9d70-6c76b948261b",
        ]);
        
        // NotSelectedState
        m.insert("2fc599c6-19e4-44ba-bd58-2bcdc024fdea", vec![
            "a1d909f3-21c1-4654-af2b-5a3daadd5ee3",
            "ae0b7451-c176-4bec-a09a-f52e824ead11",
            "9ec131be-f97e-4110-b9f1-39a3b61723ef",
            "1872404c-c3be-4938-a259-43ba8deb511e",
        ]);

        // SingleSelectedState
        m.insert("8c5c35c0-8702-42ae-a74a-71afeb6a02f6", vec![
            "eea7a43a-e4d3-4ebf-8f0d-e6538f4f5ae3",
            "63234ed0-035a-4aec-914b-19802c066b5a",
            "13bfaa1c-4cf0-40d6-9d70-6c76b948261b",            
        ]);

        // SingleNotSelectedState
        m.insert("372f0ae9-035e-4314-97ee-f6614391df13", vec![
            "a1d909f3-21c1-4654-af2b-5a3daadd5ee3",
            "ae0b7451-c176-4bec-a09a-f52e824ead11",
            "ace38e69-8a64-46f8-ad57-dc70c61f5599",
            "f8a8309b-bc06-4c87-a19f-282aa7bff614",
            "9ec131be-f97e-4110-b9f1-39a3b61723ef",
            "1872404c-c3be-4938-a259-43ba8deb511e",
        ]);

        m
    };
}