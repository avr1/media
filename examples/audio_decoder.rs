extern crate servo_media;

use servo_media::audio::buffer_source_node::AudioBufferSourceNodeMessage;
use servo_media::audio::decoder::AudioDecoderCallbacks;
use servo_media::audio::node::{AudioNodeInit, AudioNodeMessage, AudioScheduledSourceNodeMessage};
use servo_media::ServoMedia;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::{thread, time};

fn run_example(servo_media: Arc<ServoMedia>) {
    let context = servo_media.create_audio_context(Default::default());
    let args: Vec<_> = env::args().collect();
    let default = "./examples/resources/viper.ogg";
    let filename: &str = if args.len() == 2 {
        args[1].as_ref()
    } else if Path::new(default).exists() {
        default
    } else {
        panic!("Usage: cargo run --bin audio_decoder <file_path>")
    };
    let mut file = File::open(filename).unwrap();
    let mut bytes = vec![];
    file.read_to_end(&mut bytes).unwrap();
    let decoded_audio: Arc<Mutex<Vec<Vec<f32>>>> = Arc::new(Mutex::new(Vec::new()));
    let decoded_audio_ = decoded_audio.clone();
    let decoded_audio__ = decoded_audio.clone();
    let (sender, receiver) = mpsc::channel();
    let callbacks = AudioDecoderCallbacks::new()
        .eos(move || {
            sender.send(()).unwrap();
        })
        .error(|| {
            eprintln!("Error decoding audio");
        })
        .progress(move |buffer, channel| {
            let mut decoded_audio = decoded_audio_.lock().unwrap();
            decoded_audio[(channel - 1) as usize].extend_from_slice((*buffer).as_ref());
        })
        .ready(move |channels| {
            println!("There are {:?} audio channels", channels);
            decoded_audio__
                .lock()
                .unwrap()
                .resize(channels as usize, Vec::new());
        })
        .build();
    context.decode_audio_data(bytes.to_vec(), callbacks);
    println!("Decoding audio");
    receiver.recv().unwrap();
    println!("Audio decoded");
    let buffer_source =
        context.create_node(AudioNodeInit::AudioBufferSourceNode(Default::default()), Default::default());
    let dest = context.dest_node();
    context.connect_ports(buffer_source.output(0), dest.input(0));
    context.message_node(
        buffer_source,
        AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(0.)),
    );
    context.message_node(
        buffer_source,
        AudioNodeMessage::AudioBufferSourceNode(AudioBufferSourceNodeMessage::SetBuffer(Some(
            decoded_audio.lock().unwrap().to_vec().into(),
        ))),
    );
    let _ = context.resume();
    thread::sleep(time::Duration::from_millis(5000));
    let _ = context.close();
}

fn main() {
    if let Ok(servo_media) = ServoMedia::get() {
        run_example(servo_media);
    } else {
        unreachable!()
    }
}
