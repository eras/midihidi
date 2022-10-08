mod error;

use std::sync::{Arc, Mutex};
use std::{thread, time};

struct State {}

fn init_midi() -> (
    jack::Client,
    Arc<Mutex<jack::Port<jack::MidiIn>>>,
    Arc<Mutex<State>>,
) {
    let (client, _status) =
        jack::Client::new("midihid", jack::ClientOptions::NO_START_SERVER).unwrap();

    let midi_receiver = client
        .register_port("keyboard", jack::MidiIn::default())
        .unwrap();

    let state = State {};
    (
        client,
        Arc::new(Mutex::new(midi_receiver)),
        Arc::new(Mutex::new(state)),
    )
}

fn main() {
    let (client, midi_receiver, _state) = init_midi();

    let jack_callback = {
        //let state = Arc::clone(&state);
        let midi_receiver = Arc::clone(&midi_receiver);
        move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            let midi_receiver = Arc::clone(&midi_receiver);
            let midi = midi_receiver.lock().expect("Cannot lock midi");
            for event in midi.iter(ps) {
                match event.bytes {
                    [248] => (),
                    _ => println!("{event:?}"),
                }
            }
            jack::Control::Continue
        }
    };

    let port_names = client.ports(None, None, jack::PortFlags::IS_OUTPUT);

    let mut connected = false;
    for port_name in port_names {
        if let Some(port) = client.port_by_name(&port_name) {
            let matches = matches!(
                port.aliases()
                    .expect("aiee")
                    .iter()
                    .filter(|&x| x.as_str() == "Novation SL MkIII:Novation SL MkIII MIDI 1")
                    .next(),
                Some(_)
            );
            if matches {
                let midi_receiver = Arc::clone(&midi_receiver);
                let midi_receiver = midi_receiver.lock().expect("cannot lock midi_receiver");
                client
                    .connect_ports(&port, &midi_receiver)
                    .expect("cannot connect ports");
                println!("port: {port:?}");
                connected = true;
                break;
            }
        }
    }

    if !connected {
        println!("Did not find port to connect :(");
    }

    if true {
        let _active_client = client
            .activate_async((), jack::ClosureProcessHandler::new(jack_callback))
            .expect("Failed to activate client");

        // measurement_fin_rx
        //     .recv()
        //     .expect("Failed to received measurement finish response")?;

        loop {
            thread::sleep(time::Duration::from_secs(10));
        }

        // active_client
        //     .deactivate()
        //     .expect("Failed to deactivate Jack session");
    }
}
