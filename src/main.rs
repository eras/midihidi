mod error;

use input_linux::{
    EventKind, EventTime, InputEvent, InputId, KeyEvent, KeyState, SynchronizeEvent,
    SynchronizeKind, UInputHandle,
};
use nix::libc::O_NONBLOCK;
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::sync::mpsc::sync_channel;
use std::sync::{Arc, Mutex};

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

fn make_uhandle() -> UInputHandle<std::fs::File> {
    let uinput_file = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(O_NONBLOCK)
        .open("/dev/uinput")
        .expect("cannot read uinput");
    let uhandle = UInputHandle::new(uinput_file);

    uhandle.set_evbit(EventKind::Key).unwrap();
    uhandle.set_keybit(input_linux::Key::Q).unwrap();

    let input_id = InputId {
        bustype: input_linux::sys::BUS_USB,
        vendor: 0x1234,
        product: 0x5678,
        version: 0,
    };
    let device_name = b"midihid";
    uhandle
        .create(&input_id, device_name, 0, &[])
        .expect("Failed to create device");
    uhandle
}

fn main() {
    let (client, midi_receiver, _state) = init_midi();

    let uhandle = make_uhandle();

    const ZERO: EventTime = EventTime::new(0, 0);
    let events = [
        *InputEvent::from(KeyEvent::new(
            ZERO,
            input_linux::Key::Q,
            KeyState::pressed(true),
        ))
        .as_raw(),
        *InputEvent::from(KeyEvent::new(
            ZERO,
            input_linux::Key::Q,
            KeyState::pressed(false),
        ))
        .as_raw(),
        *InputEvent::from(SynchronizeEvent::new(ZERO, SynchronizeKind::Report, 0)).as_raw(),
    ];

    let jack_callback = {
        //let state = Arc::clone(&state);
        let midi_receiver = Arc::clone(&midi_receiver);
        move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            let midi_receiver = Arc::clone(&midi_receiver);
            let midi = midi_receiver.lock().expect("Cannot lock midi");
            for event in midi.iter(ps) {
                match event.bytes {
                    [248] => (),
                    _ => {
                        println!("{event:?}");
                        //uhandle.write(&events).expect("Failed to write events");
                    }
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
        let active_client = client
            .activate_async((), jack::ClosureProcessHandler::new(jack_callback))
            .expect("Failed to activate client");

        let (break_tx, break_rx) = sync_channel(1);
        ctrlc::set_handler(move || {
            break_tx.send(()).unwrap();
        })
        .expect("Error setting Ctrl-C handler");

        break_rx.recv().expect("Failed to receive ctrl-c");

        active_client
            .deactivate()
            .expect("Failed to deactivate Jack session");
    }
}
