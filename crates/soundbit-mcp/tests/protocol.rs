//! End-to-end tests: a real MCP client talks to the server over an in-memory transport.

#![allow(
    clippy::expect_used,
    reason = "tests panic on unexpected errors by design"
)]

use std::path::Path;

use rmcp::model::{CallToolRequestParams, ClientConfig};
use rmcp::service::RunningService;
use rmcp::{ClientHandler, RoleClient, ServiceExt};
use serde_json::{Value, json};
use soundbit_mcp::{ServerOptions, SoundBitServer};

#[derive(Debug, Clone, Default)]
struct TestClient;

impl ClientHandler for TestClient {
    fn get_info(&self) -> ClientConfig {
        ClientConfig::default()
    }
}

type Client = RunningService<RoleClient, TestClient>;

async fn connect(root: &Path) -> Client {
    let (server_io, client_io) = tokio::io::duplex(1 << 16);
    let options = ServerOptions {
        root: root.to_owned(),
        open: None,
    };
    let server = SoundBitServer::new(options).expect("valid workspace");
    tokio::spawn(async move {
        if let Ok(service) = server.serve(server_io).await {
            let _ = service.waiting().await;
        }
    });
    TestClient.serve(client_io).await.expect("client connects")
}

/// Calls a tool; returns the parsed JSON body (or the plain text) and whether it was an error.
async fn call(client: &Client, tool: &str, arguments: Value) -> (Value, bool) {
    let arguments = arguments.as_object().cloned().unwrap_or_default();
    let params = CallToolRequestParams::new(tool.to_owned()).with_arguments(arguments);
    let result = client
        .call_tool(params)
        .await
        .expect("protocol-level success");
    let text = result
        .content
        .first()
        .and_then(|content| content.as_text())
        .map(|text| text.text.clone())
        .unwrap_or_default();
    let body = serde_json::from_str(&text).unwrap_or(Value::String(text));
    (body, result.is_error == Some(true))
}

async fn ok(client: &Client, tool: &str, arguments: Value) -> Value {
    let (body, is_error) = call(client, tool, arguments).await;
    assert!(!is_error, "{tool} failed: {body}");
    body
}

async fn fails(client: &Client, tool: &str, arguments: Value) -> String {
    let (body, is_error) = call(client, tool, arguments).await;
    assert!(is_error, "{tool} should fail but returned {body}");
    body.as_str().unwrap_or_default().to_owned()
}

#[tokio::test]
async fn composes_exports_saves_and_undoes() {
    let dir = tempfile::tempdir().expect("temp dir");
    let client = connect(dir.path()).await;

    ok(
        &client,
        "song_new",
        json!({"title": "Test", "chip_profile": "nes", "bpm": 150}),
    )
    .await;
    let lead = ok(&client, "instrument_add", json!({
        "name": "Lead", "voice": {"type": "pulse", "duty": "quarter"}, "envelope": {"release_ms": 40}
    }))
    .await;
    assert_eq!(lead["id"], 1);
    assert_eq!(lead["envelope"]["release_ms"], 40);
    assert_eq!(lead["change"], "AI: Add instrument \"Lead\"");

    let pattern = ok(&client, "pattern_create", json!({
        "name": "Hook", "instrument_id": 1,
        "notes": [{"note": "C5", "start": 0, "length": 4}, {"note": "E5", "start": 4, "length": 4, "velocity": 90}]
    }))
    .await;
    assert!(
        pattern["grid"]
            .as_str()
            .expect("grid")
            .starts_with("00 C5\n01 |\n")
    );

    let track = ok(&client, "track_add", json!({"name": "Melody"})).await;
    assert_eq!(track["channel"], "Pulse 1");
    ok(
        &client,
        "arrangement_place",
        json!({"track_id": 1, "pattern_id": 1, "start_bar": 0, "count": 4}),
    )
    .await;

    let song = ok(&client, "song_get", json!({})).await;
    assert_eq!(song["length"]["bars"], 4);
    assert_eq!(song["chip_profile"], "nes");
    assert!(
        song["arrangement"]
            .as_str()
            .expect("grid")
            .contains("T1 Melody")
    );

    let export = ok(&client, "export_wav", json!({"path": "renders/test"})).await;
    assert_eq!(export["exported"], "renders/test.wav");
    assert!(dir.path().join("renders/test.wav").is_file());

    ok(&client, "song_save", json!({"path": "test"})).await;
    assert!(dir.path().join("test.sbproj").is_file());
    let listed = ok(&client, "files_list", json!({})).await;
    assert_eq!(listed["projects"], json!(["test.sbproj"]));

    let undone = ok(&client, "history_undo", json!({})).await;
    assert_eq!(
        undone["undone"],
        "AI: Place pattern 1 on track 1 (4 bar(s))"
    );
    assert_eq!(
        ok(&client, "song_get", json!({})).await["length"]["bars"],
        0
    );
    ok(&client, "history_redo", json!({})).await;
    assert_eq!(
        ok(&client, "song_get", json!({})).await["length"]["bars"],
        4
    );
}

#[tokio::test]
async fn errors_tell_the_agent_what_to_fix() {
    let dir = tempfile::tempdir().expect("temp dir");
    let client = connect(dir.path()).await;
    ok(&client, "song_update", json!({"chip_profile": "nes"})).await;
    ok(
        &client,
        "instrument_add",
        json!({"name": "Bass", "voice": {"type": "triangle"}}),
    )
    .await;
    ok(
        &client,
        "pattern_create",
        json!({"name": "Bass", "instrument_id": 1}),
    )
    .await;
    ok(&client, "track_add", json!({"name": "Pulse 1"})).await;

    let message = fails(
        &client,
        "pattern_add_notes",
        json!({"id": 1, "notes": [{"note": "H4", "start": 0}]}),
    )
    .await;
    assert!(
        message.starts_with("notes[0]: invalid note name `H4`"),
        "{message}"
    );

    let message = fails(
        &client,
        "pattern_add_notes",
        json!({"id": 1, "notes": [{"note": "C3", "start": 15, "length": 4}]}),
    )
    .await;
    assert!(
        message.contains("does not fit in a bar of 16 steps"),
        "{message}"
    );

    let message = fails(
        &client,
        "arrangement_place",
        json!({"track_id": 1, "pattern_id": 1, "bars": [0]}),
    )
    .await;
    assert!(
        message.contains("Pulse 1 channel, which cannot produce triangle voices"),
        "{message}"
    );

    let message = fails(&client, "export_wav", json!({"path": "../escape"})).await;
    assert!(message.contains("relative to the workspace"), "{message}");

    let message = fails(&client, "song_new", json!({"title": "Fresh"})).await;
    assert!(message.contains("unsaved changes"), "{message}");
    ok(
        &client,
        "song_new",
        json!({"title": "Fresh", "discard_unsaved": true}),
    )
    .await;
    assert_eq!(
        fails(&client, "history_undo", json!({})).await,
        "nothing to undo"
    );
}

#[tokio::test]
async fn presets_and_sound_shaping() {
    let dir = tempfile::tempdir().expect("temp dir");
    let client = connect(dir.path()).await;

    let presets = ok(&client, "preset_list", json!({"category": "drums"})).await;
    let ids: Vec<&str> = presets["presets"]
        .as_array()
        .expect("list")
        .iter()
        .filter_map(|preset| preset["id"].as_str())
        .collect();
    assert!(ids.contains(&"kick"), "{ids:?}");
    assert!(presets["categories"].as_array().expect("list").len() > 3);

    let kick = ok(&client, "instrument_add_preset", json!({"preset": "kick"})).await;
    assert_eq!(kick["pitch_envelope"]["semitones"], 36);
    let message = fails(&client, "instrument_add_preset", json!({"preset": "kazoo"})).await;
    assert!(message.contains("unknown preset \"kazoo\""), "{message}");

    let pad = ok(
        &client,
        "instrument_add",
        json!({
            "name": "Pad", "voice": {"type": "sawtooth"},
            "unison": {"voices": 3}, "filter": {"mode": "low_pass", "cutoff_hz": 900},
            "vibrato": {"depth_cents": 15, "rate_hz": 5.5}, "reverb_send_percent": 40
        }),
    )
    .await;
    assert_eq!(pad["unison"]["voices"], 3);
    assert_eq!(pad["vibrato"]["rate_hz"], 5.5);
    let pad = ok(
        &client,
        "instrument_update",
        json!({"id": 2, "filter": {"resonance_percent": 30}}),
    )
    .await;
    assert_eq!(pad["filter"]["cutoff_hz"], 900, "unset fields stay");
    assert_eq!(pad["filter"]["resonance_percent"], 30);
    let message = fails(
        &client,
        "instrument_update",
        json!({"id": 2, "vibrato": {"rate_hz": 99}}),
    )
    .await;
    assert!(
        message.contains("rate_hz must be between 0.1 and 20"),
        "{message}"
    );
}

#[tokio::test]
async fn batch_placement_and_mix_reports() {
    let dir = tempfile::tempdir().expect("temp dir");
    let client = connect(dir.path()).await;
    ok(&client, "instrument_add_preset", json!({"preset": "kick"})).await;
    ok(
        &client,
        "instrument_add",
        json!({"name": "Pad", "voice": {"type": "sawtooth"}, "reverb_send_percent": 40}),
    )
    .await;

    ok(
        &client,
        "pattern_create",
        json!({"name": "Beat", "instrument_id": 1, "notes": [{"note": "C2", "start": 0}]}),
    )
    .await;
    ok(&client, "pattern_create", json!({"name": "Chord", "instrument_id": 2, "notes": [{"note": "C4", "start": 0, "length": 16}]})).await;
    ok(&client, "track_add", json!({"name": "Drums"})).await;
    ok(&client, "track_add", json!({"name": "Pads"})).await;
    let placed = ok(
        &client,
        "arrangement_place_many",
        json!({"placements": [
            {"track_id": 1, "pattern_id": 1, "start_bar": 0, "count": 2},
            {"track_id": 2, "pattern_id": 2, "bars": [0, 1]}
        ]}),
    )
    .await;
    assert_eq!(placed["change"], "AI: Place 2 clips");
    let message = fails(
        &client,
        "arrangement_place_many",
        json!({"placements": [
            {"track_id": 1, "pattern_id": 1, "bars": [3]},
            {"track_id": 1, "pattern_id": 9, "bars": [4]}
        ]}),
    )
    .await;
    assert!(message.contains("pattern 9 does not exist"), "{message}");
    assert_eq!(
        ok(&client, "song_get", json!({})).await["length"]["bars"],
        2,
        "nothing changed"
    );

    let song = ok(&client, "song_update", json!({"reverb_size": 85})).await;
    assert_eq!(song["reverb"]["size_percent"], 85);

    let export = ok(&client, "export_wav", json!({"path": "mix"})).await;
    let tracks = export["tracks"].as_array().expect("levels");
    assert_eq!(tracks.len(), 2);
    assert_eq!(tracks[1]["name"], "Pads");
    assert!(
        tracks.iter().all(|track| track["rms"].as_f64() > Some(0.0)),
        "{export}"
    );
}

#[tokio::test]
async fn tool_schemas_are_stable() {
    let dir = tempfile::tempdir().expect("temp dir");
    let client = connect(dir.path()).await;
    let mut tools = client.list_all_tools().await.expect("lists tools");
    tools.sort_by(|a, b| a.name.cmp(&b.name));
    let summary: Vec<Value> = tools
        .iter()
        .map(|tool| {
            json!({
                "name": tool.name,
                "description": tool.description,
                "input_schema": tool.input_schema,
                "annotations": tool.annotations,
            })
        })
        .collect();
    assert_eq!(summary.len(), 30);
    insta::assert_json_snapshot!("tool_schemas", summary);
}
