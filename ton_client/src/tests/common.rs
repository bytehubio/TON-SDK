use crate::{
    abi::ParamsOfEncodeMessage,
    net::{ParamsOfQueryCollection, ResultOfQueryCollection},
    processing::{ParamsOfSendMessage, ResultOfSendMessage},
};

use super::*;

#[test]
fn test_parallel_requests() {
    let client1 = std::sync::Arc::new(TestClient::new());
    let client2 = TestClient::new();
    let client3 = client1.clone();

    let start = std::time::Instant::now();
    let timeout: u32 = 5000;
    let long_wait = std::thread::spawn(move || {
        client3
            .request_json(
                "net.wait_for_collection",
                json!({
                    "collection": "accounts".to_owned(),
                    "filter": json!({
                        "id": { "eq": "123" }
                    }),
                    "result": "id",
                    "timeout": timeout
                }),
            )
            .unwrap_err();
        client3
    });

    std::thread::sleep(std::time::Duration::from_millis(500));

    let query = |client: &TestClient| {
        let _: ResultOfQueryCollection = client
            .request(
                "net.query_collection",
                ParamsOfQueryCollection {
                    collection: "accounts".to_owned(),
                    filter: Some(json!({})),
                    result: "id".to_owned(),
                    limit: Some(1),
                    order: None,
                },
            )
            .unwrap();
    };

    // check that request with another context doesn't wait
    query(&client2);
    assert!(start.elapsed().as_millis() < timeout as u128);

    // check that request with same context doesn't wait too
    query(&client1);
    assert!(start.elapsed().as_millis() < timeout as u128);

    long_wait.join().unwrap();
    assert!(start.elapsed().as_millis() > timeout as u128);
}

#[test]
fn test_deferred_init() {
    let client = TestClient::new_with_config(json!({
        "network": {
            "server_address": "123"
        }
    }));

    // local functions should work
    client.generate_sign_keys();

    // deferred network init should fail due to wrong server address
    let result = client
        .request_json(
            "net.query_collection",
            json!({
                "collection": "accounts",
                "result": "id".to_owned(),
            }),
        )
        .unwrap_err();
    //println!("{:#?}", result);

    assert_eq!(result.code, crate::net::ErrorCode::QueryFailed as u32);
}

#[tokio::test(core_threads = 2)]
async fn test_clock_sync() {
    let client = TestClient::new_with_config(json!({
        "network": {
            "endpoints": TestClient::endpoints(),
            "out_of_sync_threshold": 0,
        }
    }));

    // queries should not fail even when not synchronized
    let _: ResultOfQueryCollection = client
        .request_async(
            "net.query_collection",
            ParamsOfQueryCollection {
                collection: "accounts".to_owned(),
                result: "id".to_owned(),
                limit: Some(1),
                filter: None,
                order: None,
            },
        )
        .await
        .unwrap();

    let msg = client
        .encode_message(ParamsOfEncodeMessage {
            abi: TestClient::abi(HELLO, None),
            address: Some(client.giver_address().await),
            call_set: CallSet::some_with_function("touch"),
            deploy_set: None,
            processing_try_index: None,
            signer: Signer::None,
        })
        .await
        .unwrap();

    let result = client
        .request_async::<_, ResultOfSendMessage>(
            "processing.send_message",
            ParamsOfSendMessage {
                abi: None,
                message: msg.message,
                send_events: false,
            },
        )
        .await
        .unwrap_err();

    assert!(result
        .message
        .ends_with("Synchronize your device time with internet time"));
}

#[tokio::test(core_threads = 2)]
async fn test_alternative_workchain() {
    let client0 = TestClient::new_with_config(json!({
        "network": {
            "endpoints": ["common1.ton.dev"],
            "message_retries_count": 0,
        }
    }));

    let client1 = TestClient::new_with_config(json!({
        "network": {
            "endpoints": ["common2.ton.dev"],
        }
    }));

    let (abi, tvc) = TestClient::package(GIVER_V2, Some(2));
    let keys = client0.generate_sign_keys();

    let mut deploy_params = ParamsOfEncodeMessage {
        abi: abi.clone(),
        deploy_set: Some(DeploySet { 
            tvc: tvc.clone(),
            workchain_id: Some(-1),
            ..Default::default()
        }),
        call_set: CallSet::some_with_function("constructor"),
        signer: Signer::Keys { keys: keys.clone() },
        processing_try_index: None,
        address: None,
    };

    // deploy giver to masterchain first because messages between workchains are not supported yet
    let address = client0
        .deploy_with_giver_async(
            deploy_params.clone(),
            Some(2_000_000_000u64),
        )
        .await;

    deploy_params.deploy_set.as_mut().unwrap().workchain_id = Some(1);
    let target_address = address.trim_start_matches("-").to_owned();
    println!("{}", target_address);

    // send some value to target account
    let run_result = client0
        .net_process_function(
            address.clone(),
            abi.clone(),
            "sendTransaction",
            json!({
                "dest": target_address.to_string(),
                "value": 1_500_000_000u64,
                "bounce": false
            }),
            Signer::Keys { keys: keys.clone() },
        )
        .await
        .unwrap();

    println!("Before wait");

    // wait for tokens reception
    client0.wait_output_messages(run_result).await;

    println!("Before deploy");

    client0
        .net_process_message(
            ParamsOfProcessMessage {
                message_encode_params: deploy_params,
                send_events: false,
            },
            TestClient::default_callback
        ).await.unwrap();
    
    println!("Before send");

    // call target
    let run_result = client0
        .net_process_function(
            target_address.clone(),
            abi,
            "sendTransaction",
            json!({
                "dest": address.to_string(),
                "value": 500_000_000u64,
                "bounce": false
            }),
            Signer::Keys { keys: keys.clone() },
        )
        .await
        .unwrap();

    println!("After wait");

    // wait for tokens return
    client0.wait_output_messages(run_result).await;
}
