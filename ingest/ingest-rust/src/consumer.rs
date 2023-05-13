use std::{collections::HashSet, str::FromStr};

use amqprs::{
    channel::{BasicConsumeArguments, Channel, QueueBindArguments, QueueDeclareArguments},
    connection::Connection as amqpConnection,
    connection::OpenConnectionArguments,
    consumer::BlockingConsumer,
    BasicProperties, Deliver,
};
use chrono;
use futures::pin_mut;
use lazy_static::lazy_static;
use regex::Regex;
use tokio::sync::{mpsc, Notify};
use tokio_postgres::{
    binary_copy::BinaryCopyInWriter,
    connect,
    types::{ToSql, Type},
    NoTls,
};
use tracing::{info, metadata};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Debug)]

pub struct LogRow {
    time: chrono::DateTime<chrono::Utc>,
    data: serde_json::Map<String, serde_json::Value>,
    level: String,
    words: Vec<String>,
}

impl LogRow {
    pub fn new(data: &serde_json::Map<String, serde_json::Value>) -> Self {
        let mut words = HashSet::new();
        let mut try_words: Vec<serde_json::Value> = Vec::new();
        let mut final_data: serde_json::Map<String, serde_json::Value> = data.clone();
        let level = data
            .get("level")
            .unwrap_or(&serde_json::Value::String("INFO".to_string()))
            .as_str()
            .unwrap()
            .to_string();
        if !level.is_empty() {
            final_data.remove_entry("level");
        }
        try_words.push(final_data.to_owned().into());
        loop {
            let maybe_v = try_words.pop();
            if maybe_v.is_none() {
                break;
            }
            let value = maybe_v.unwrap();
            let try_str = value.as_str();
            if try_str.is_some() {
                lazy_static! {
                    static ref RE: Regex = Regex::new(r"[\w]+([-_][\w]+)*").unwrap();
                };
                for cap in RE.captures_iter(try_str.unwrap()) {
                    words.insert(cap.get(0).unwrap().as_str().to_string());
                }
                continue;
            }
            let try_array = value.as_array();
            if try_array.is_some() {
                for val in try_array.unwrap() {
                    try_words.push(val.to_owned());
                }
                continue;
            }
            let try_nested = value.as_object();
            if try_nested.is_some() {
                for k in try_nested.unwrap().keys() {
                    words.insert(k.clone());
                }
                for val in try_nested.unwrap().values() {
                    try_words.push(val.to_owned());
                }
            }
        }
        Self {
            time: chrono::offset::Utc::now(),
            data: final_data,
            level: level,
            words: words.into_iter().collect(),
        }
    }
}
pub struct MyConsumer {
    sender: mpsc::Sender<LogRow>,
}

impl MyConsumer {
    /// Return a new consumer.
    ///
    /// See [Acknowledgement Modes](https://www.rabbitmq.com/consumers.html#acknowledgement-modes)
    pub fn new(sender: mpsc::Sender<LogRow>) -> Self {
        // Now we can execute a simple statement that just returns its parameter.
        Self { sender }
    }
}

impl BlockingConsumer for MyConsumer {
    fn consume(
        &mut self,
        _channel: &Channel,
        _deliver: Deliver,
        _basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        let utf8_content = String::from_utf8(content).unwrap_or("{}".to_string());
        let mut deser_res = serde_json::from_str(&utf8_content.as_str());
        if deser_res.is_err() {
            deser_res = serde_json::Value::from_str("[]");
        }
        for row in deser_res.unwrap().as_array().unwrap() {
            let log = LogRow::new(row.as_object().unwrap());
            self.sender.blocking_send(log).unwrap();
        }
        // ack explicitly if manual ack
        /*
        if !self.no_ack {
            info!("ack to delivery {} on channel {}", deliver, channel);
            let args = BasicAckArguments::new(deliver.delivery_tag(), false);
            channel.basic_ack(args).unwrap();
        }
        */
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 24)]
async fn main() {
    // construct a subscriber that prints formatted traces to stdout
    // global subscriber with log level according to RUST_LOG
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(metadata::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .try_init()
        .ok();

    // open a connection to RabbitMQ server

    // open a channel on the connection
    // declare a server-named transient queue

    // bind the queue to exchange
    //let rounting_key = "amqprs.example";
    //let exchange_name = "amq.topic";

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    //////////////////////////////////////////////////////////////////////////////
    // start consumer, auto ack

    let (tx, rx) = mpsc::channel(4096 * 4);
    let (tx_2, rx_2) = mpsc::channel(4096 * 4);
    let (tx_3, rx_3) = mpsc::channel(4096 * 4);
    let (tx_4, rx_4) = mpsc::channel(4096 * 4);

    for i in 0..8 {
        let mut tx2 = tx.clone();
        if i % 4 == 0 {
            tx2 = tx_2.clone();
        }
        if i % 4 == 1 {
            tx2 = tx_3.clone();
        }
        if i % 5 == 2 {
            tx2 = tx_4.clone();
        }
        tokio::spawn(async move {
            let connection = amqpConnection::open(&OpenConnectionArguments::new(
                "localhost",
                5672,
                "guest",
                "guest",
            ))
            .await
            .unwrap();
            let channel = connection.open_channel(None).await.unwrap();
            let (queue_name, _, _) = channel
                .queue_declare(QueueDeclareArguments::durable_client_named(
                    "amqprs.examples.basic",
                ))
                //.queue_declare(QueueDeclareArguments::default())
                .await
                .unwrap()
                .unwrap();
            channel
                .queue_bind(QueueBindArguments::new(
                    &queue_name,
                    "amq.topic",
                    "amqprs.example",
                ))
                .await
                .unwrap();
            let args = BasicConsumeArguments::new(&queue_name, "basic_consumer")
                .manual_ack(false)
                .finish();
            channel
                .basic_consume_blocking(MyConsumer::new(tx2), args)
                .await
                .unwrap();
            let guard = Notify::new();
            guard.notified().await;
        });
    }

    for rx_handle in [rx, rx_2, rx_3, rx_4] {
        let mut my_rx = rx_handle;
        let _manager = tokio::spawn(async move {
            // Establish a connection to the server
            let (mut client, db_connect) =
                connect("host=localhost user=postgres password=test", NoTls)
                    .await
                    .unwrap();
            tokio::spawn(async move {
                if let Err(e) = db_connect.await {
                    eprintln!("connection error: {}", e);
                }
            });
            // Start receiving messages
            while let Some(cmd) = my_rx.recv().await {
                let mut rows = Vec::new();
                rows.push(cmd);
                for _i in 0..19999 {
                    let res = my_rx.try_recv();
                    if res.is_err() {
                        break;
                    }
                    rows.push(res.unwrap());
                }
                info!("{}", rows.len());
                let transaction = client.transaction().await.unwrap();
                let sink = transaction
                    .copy_in("COPY logs (time, logdata, level, words) FROM STDIN BINARY")
                    .await
                    .unwrap();
                let writer = BinaryCopyInWriter::new(
                    sink,
                    &[Type::TIMESTAMPTZ, Type::JSONB, Type::TEXT, Type::TEXT_ARRAY],
                );
                pin_mut!(writer);
                for log in rows {
                    let mut row: Vec<&'_ (dyn ToSql + Sync)> = Vec::new();
                    let val = serde_json::Value::from(log.data);
                    row.push(&log.time);
                    row.push(&val);
                    row.push(&log.level);
                    row.push(&log.words);
                    let _num_written = writer.as_mut().write(&row).await.unwrap();
                }
                writer.finish().await.unwrap();
                transaction.commit().await.unwrap();
            }
        });
    }
    // consume forever
    info!("Consuming forever");
    let guard = Notify::new();
    guard.notified().await;
}
