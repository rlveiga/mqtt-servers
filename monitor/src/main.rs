use std::process;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;

static MQTT_BROKER: &str = "tcp://broker.emqx.io:1883";
static MQTT_CLIENT: &str = "inf1406/monitor";

extern crate paho_mqtt as mqtt;

const QOS: i32 = 1;

#[derive(Deserialize, Serialize)]
struct Request {
    idserv: i32
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let n: i32 = args[1].parse().unwrap();

    let mut last_seen: Vec<u64> = Vec::new();
    let mut server_online: Vec<bool> = Vec::new();

    for i in 0..n {
        last_seen.push(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
        server_online.push(true);
    }

    // Connect monitor to MQTT broket
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(MQTT_BROKER.to_string())
        .client_id(MQTT_CLIENT.to_string())
        .finalize();

    let mut cli = mqtt::Client::new(create_opts).unwrap_or_else(|err| {
        println!("Error creating the client: {:?}", err);
        process::exit(1);
    });

    let rx = cli.start_consuming();

    let lwt = mqtt::MessageBuilder::new()
        .topic("inf1406-reqs")
        .finalize();

    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(false)
        .will_message(lwt)
        .finalize();

    if let Err(e) = cli.connect(conn_opts) {
        println!("Unable to connect:\n\t{:?}", e);
        process::exit(1);
    }

    if let Err(e) = cli.subscribe("inf1406-monitor", QOS) {
        println!("Error subscribing to topic: {:?}", e);
        process::exit(1);
    }

    // Wait for messages from servers
    println!("Monitor waiting for messages...");
    for msg in rx.iter() {
        if let Some(msg) = msg {
            let json_start = "inf1406-monitor: ".len();
            let mut data: &str = &msg.to_string();
            data = &data[json_start..data.len()];

            if data != "Server lost connection" {
                let req: Request = serde_json::from_str(data).unwrap();

                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                last_seen[req.idserv as usize] = now; // Update last server heartbeat time

                for i in 0..n {
                    let elapsed = now - last_seen[i as usize];

                    // Server has timed out, request substitution
                    if elapsed > 5 && server_online[i as usize] {
                        println!("Server {} has timed out", i);

                        server_online[i as usize] = false;

                        let timeout_data = json!({
                            "tipomsg": "falhaserv",
                            "idserv": i,
                            "vistoem": last_seen[i as usize],
                            "chave": "",
                            "novovalor": "",
                            "topicoresp": "",
                            "idpedido": 0
                        });

                        let msg = mqtt::Message::new("inf1406-reqs", timeout_data.to_string(), QOS);
                        let tok = cli.publish(msg);
                
                        if let Err(e) = tok {
                            println!("Error sending message: {:?}", e);
                        } else {
                        }
                    }
                }
            }
        }
    }
}

// TO-DO:
// - implement server-monitor heartbeat
// - implement substitute servers