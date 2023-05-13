use amqprs::{
    channel::{BasicPublishArguments, QueueBindArguments, QueueDeclareArguments},
    connection::{Connection, OpenConnectionArguments},
    BasicProperties,
};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use std::sync::mpsc::channel;
use std::time::Duration;
use std::{io::stdin, thread::sleep};

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    // construct a subscriber that prints formatted traces to stdout
    // global subscriber with log level according to RUST_LOG
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .try_init()
        .ok();

    // open a connection to RabbitMQ server
    let connection = Connection::open(&OpenConnectionArguments::new(
        "localhost",
        5672,
        "guest",
        "guest",
    ))
    .await
    .unwrap();

    // open a channel on the connection
    let amqp_channel = connection.open_channel(None).await.unwrap();

    // declare a durable queue
    let (queue_name, _, _) = amqp_channel
        .queue_declare(QueueDeclareArguments::durable_client_named(
            "amqprs.examples.basic",
        ))
        .await
        .unwrap()
        .unwrap();

    // bind the queue to exchange
    let rounting_key = "amqprs.example";
    let exchange_name = "amq.topic";
    amqp_channel
        .queue_bind(QueueBindArguments::new(
            &queue_name,
            exchange_name,
            rounting_key,
        ))
        .await
        .unwrap();

    let args = BasicPublishArguments::new(exchange_name, rounting_key);
    let (tx, rx) = channel();
    let _reader_manager = tokio::spawn(async move {
        loop {
            let mut line = String::new();
            let read_line = stdin().read_line(&mut line);
            if read_line.is_ok() {
                let _res = tx.send(line);
            }
        }
    });
    loop {
        let mut final_str: String = "[".to_owned();
        for i in 0..32 {
            let read_event = rx.try_recv();
            if read_event.is_err() {
                break;
            }
            let read_bytes = read_event.unwrap();
            let mut format_prefix = ",";
            if i == 0 {
                format_prefix = "";
            }
            final_str.push_str(format!("{}{}", format_prefix, read_bytes).as_str());
        }
        final_str.push_str("]");
        if final_str.len() > 2 {
            amqp_channel
                .basic_publish(
                    BasicProperties::default(),
                    final_str.as_bytes().to_vec(),
                    args.clone(),
                )
                .await
                .unwrap();
        } else {
            sleep(Duration::from_millis(200));
        }
    }
    // explicitly close
    /*
    amqp_channel.close().await.unwrap();
    connection.close().await.unwrap();
    */
}
