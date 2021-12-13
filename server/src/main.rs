use serde::{Deserialize, Serialize};
use std::process;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{thread, time};
use serde_json::json;
use std::env;

#[derive(Deserialize, Serialize)]
struct Request {
    tipomsg: String,
    chave: String,
    novovalor: String,
    topicoresp: String,
    idpedido: i32,
    idserv: i32,
    vistoem: u64
}

struct RequestLogEntry {
    request: Request,
    timestamp: u64
}

struct ServerInfo {
    id: i32,
    chave: String,
    valor: String,
}

extern crate paho_mqtt as mqtt;

static MQTT_BROKER: &str = "tcp://broker.emqx.io:1883";

const QOS: i32 = 1;

fn send_heartbeat(server_id: i32) {
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(MQTT_BROKER.to_string())
        .client_id(format!("inf1406/server-monitor-{}", server_id).to_string())
        .finalize();

    let cli = mqtt::Client::new(create_opts).unwrap_or_else(|err| {
        println!("Error creating the client: {:?}", err);
        process::exit(1);
    });

    let lwt = mqtt::MessageBuilder::new()
        .topic("inf1406-monitor")
        .payload("Server lost connection")
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

    let data = json!({
        "idserv": server_id as i32 // force i32 type to fix string bug
    });

    // Send a hearbeat to monitor every 5 seconds
    loop {
        let msg = mqtt::Message::new("inf1406-monitor", data.to_string(), QOS);
        let tok = cli.publish(msg);

        if let Err(e) = tok {
            println!("Error sending message heartbeat: {:?}", e);
        } else {
            println!("Server {} sent heartbeat", server_id);
        }

        thread::sleep(time::Duration::from_millis(5000));
    }
}

fn get_request_server(key: &String, n: i32) -> i32 {
    let mut result = 0;

    for i in 1..(key.len() + 1) as i32 {
        result += i;
    }

    result = result % n;

    return result;
}

fn handle_message(msg: mqtt::Message) -> Request {
    let json_start = "inf1406-reqs: ".len();
    let mut data: &str = &msg.to_string();
    data = &data[json_start..data.len()];

    let req: Request = serde_json::from_str(data).unwrap();

    return req;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let id: i32 = args[1].parse().unwrap();
    let n: i32 = args[2].parse().unwrap();

    let mut server = ServerInfo {
        id: id,
        chave: String::from(""),
        valor: String::from(""),
    };

    let request_log: Vec<RequestLogEntry> = Vec::new();

    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(MQTT_BROKER.to_string())
        .client_id(format!("inf1406/server-{}", server.id).to_string())
        .finalize();

    let mut cli = mqtt::Client::new(create_opts).unwrap_or_else(|err| {
        println!("Error creating the client: {:?}", err);
        process::exit(1);
    });

    let rx = cli.start_consuming();

    let lwt = mqtt::MessageBuilder::new()
        .topic("inf1406-resp")
        .payload("Server lost connection")
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

    if let Err(e) = cli.subscribe("inf1406-reqs", QOS) {
        println!("Error subscribing to topic: {:?}", e);
        process::exit(1);
    }

    let server_id = server.id;
    
    thread::spawn(move || {
        send_heartbeat(server_id);
    });

    println!("Processing requests...");

    for msg in rx.iter() {
        if let Some(msg) = msg {
            if &msg.to_string()[0..12] != "inf1406-resp" {
                let req = handle_message(msg);
                
                // Update server info
                if &req.tipomsg == "insere" {
                    server.chave = req.chave;
                    server.valor = req.novovalor;
                }

                // Return data to client
                else if &req.tipomsg == "consulta" {
                    let server_id = get_request_server(&req.chave, n);

                    println!("Server {} will respond", server_id);

                    if server_id == server.id {
                        let msg = mqtt::Message::new(&req.topicoresp.to_string(), server.valor.to_string(), 1);
                        let tok = cli.publish(msg);

                        if let Err(e) = tok {
                            println!("Error sending message consulta: {:?}", e);
                        } else {
                        }
                    } else {
                    }
                }

                else if &req.tipomsg == "falhaserv" {
                    let substitute_server = (&req.idserv + 1) % n;

                    println!("Server {} will subtitute {}", substitute_server, &req.idserv);

                    if substitute_server == server.id {
                        // println!("Server {} will subtitute {}", server.id, &req.idserv);
                    }
                }

                // Não estou conseguindo adicionar o request para o log de requests..
                // request_log.push(RequestLogEntry {
                //     request: req,
                //     timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
                // });
            }
        }
    }
}

// TO-DO:
// - implement substitute servers
