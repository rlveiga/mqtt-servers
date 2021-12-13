use serde_json::json;
use std::process;
use std::thread;
use std::time::Duration;

extern crate paho_mqtt as mqtt;

static MQTT_BROKER: &str = "tcp://broker.emqx.io:1883";
static MQTT_CLIENT: &str = "inf1406/client";

const QOS: i32 = 1;

const REQUEST_KEYS: &[&str] = &["melhor time", "melhor faculdade"];
const RQUEST_VALUES: &[&str] = &["Botafogo", "PUC-Rio"];

fn response_listener(rx: mqtt::Receiver<Option<mqtt::Message>>) {
    println!("Waiting for server response...");

    for msg in rx.iter() {
        if let Some(msg) = msg {
            let msg_start = "inf1406-resp: ".len();
            let mut data: &str = &msg.to_string();
            data = &data[msg_start..data.len()];

            println!("Received from server: {}", data);
        }
    }
}

fn insere(cli: &mqtt::Client, key: String, value: String, request_id: i32) {
    let data = json!({
        "tipomsg": "insere",
        "chave": key,
        "novovalor": value,
        "topicoresp": "inf1406-resp",
        "idpedido": request_id,
        "idserv": 0 as i32,
        "vistoem": 0 as u64
    });

    let msg = mqtt::Message::new("inf1406-reqs", data.to_string(), QOS);
    let tok = cli.publish(msg);

    if let Err(e) = tok {
        println!("Error sending message: {:?}", e);
    } else {
        println!("Mensagem enviada!")
    }
}

fn consulta(cli: &mqtt::Client, key: String, request_id: i32) {
    let data = json!({
        "tipomsg": "consulta",
        "chave": key,
        "novovalor": "",
        "topicoresp": "inf1406-resp",
        "idpedido": request_id,
        "idserv": 0 as i32,
        "vistoem": 0 as u64
    });

    let msg = mqtt::Message::new("inf1406-reqs", data.to_string(), QOS);
    let tok = cli.publish(msg);

    if let Err(e) = tok {
        println!("Error sending message: {:?}", e);
    } else {
        println!("Mensagem enviada!")
    }
}

fn main() {
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(MQTT_BROKER)
        .client_id(MQTT_CLIENT.to_string())
        .finalize();

    let mut cli = mqtt::Client::new(create_opts).unwrap_or_else(|err| {
        println!("Error creating the client: {:?}", err);
        process::exit(1);
    });

    let rx = cli.start_consuming();
    
    let lwt = mqtt::MessageBuilder::new()
        .topic("inf1406-reqs")
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

    if let Err(e) = cli.subscribe("inf1406-resp", QOS) {
        println!("Error subscribe: {:?}", e);
        process::exit(1);
    }

    thread::spawn(move || {
        response_listener(rx);
    });

    // Send requests
    for num in 0..50 {
        if num % 5 == 0 {
            if num % 2 == 0 {
                insere(
                    &cli,
                    String::from(REQUEST_KEYS[0]),
                    String::from(RQUEST_VALUES[0]),
                    num,
                );
            }

            else {
                insere(
                    &cli,
                    String::from(REQUEST_KEYS[1]),
                    String::from(RQUEST_VALUES[1]),
                    num,
                );
            }
        }
        
        else {
            if num % 2 == 0 {
                consulta(&cli, String::from(REQUEST_KEYS[0]), num);
            }            

            else {
                consulta(&cli, String::from(REQUEST_KEYS[1]), num);
            }
        }
    }
}
