use selecting::Selector;
use std::io::{Write, stdout};
use vbus_core::tools::pipe_flag::*;
use vbus_core::{Channel, Message};
use vbus_sensors::keyboard::*;

fn main() {
    let channel = Channel::<KeyboardData>::new();
    let _keyboard = Keyboard::new(&channel);

    println!(
        "This program echoes stdin (usually keyboard) to stdout, one character at a time. Press/input 'q' to exit."
    );

    let (pipe_reader, mut pipe_writer) = pipe_flag();
    let _tc = channel.new_threaded_consumer(move |messages| {
        process(messages, &mut pipe_writer);
    });

    // Wait for the thread to signal that we should exit
    let mut selector = Selector::new();
    selector.add_read(&pipe_reader);
    selector.select().unwrap();
}

fn process(messages: Vec<Message<KeyboardData>>, pipe_writer: &mut PipeFlagWriter) {
    for message in messages {
        let payload = message.get_payload();

        print!("{}", payload.data);

        if payload.data == 'Q' || payload.data == 'q' {
            pipe_writer.raise();
            println!("\nExiting...");
        }
    }

    stdout().flush().unwrap();
}
