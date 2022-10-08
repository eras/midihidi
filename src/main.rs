extern crate lazy_static;
mod error;

use input_linux::{
    EventKind, EventTime, InputEvent, InputId, KeyEvent, KeyState, SynchronizeEvent,
    SynchronizeKind, UInputHandle,
};
use lazy_static::lazy_static;
use nix::libc::O_NONBLOCK;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::sync::mpsc::sync_channel;
use std::sync::{Arc, Mutex};

use clap::Parser;

#[derive(clap::ValueEnum, Clone, Debug)]
enum KeyMappingOptions {
    Alphabet,
    Qwerty,
}

impl KeyMappingOptions {
    fn get_keymap(&self) -> KeyMap {
        match self {
            Self::Alphabet => KeyMap::new(ALPHABET_MAPPING),
            Self::Qwerty => KeyMap::new(QWERTY_MAPPING),
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    keymap: KeyMappingOptions,

    /// Connect to this input device
    #[arg(short, long)]
    input: Option<String>,
}

#[derive(Debug, PartialEq, PartialOrd)]
enum Mapping {
    Alphabet(u8),
    Fixed(input_linux::Key),
}

type KeyMapping = [input_linux::Key; 26];

const ALPHABET_MAPPING: KeyMapping = [
    input_linux::Key::A,
    input_linux::Key::B,
    input_linux::Key::C,
    input_linux::Key::D,
    input_linux::Key::E,
    input_linux::Key::F,
    input_linux::Key::G,
    input_linux::Key::H,
    input_linux::Key::I,
    input_linux::Key::J,
    input_linux::Key::K,
    input_linux::Key::L,
    input_linux::Key::M,
    input_linux::Key::N,
    input_linux::Key::O,
    input_linux::Key::P,
    input_linux::Key::Q,
    input_linux::Key::R,
    input_linux::Key::S,
    input_linux::Key::T,
    input_linux::Key::U,
    input_linux::Key::V,
    input_linux::Key::W,
    input_linux::Key::X,
    input_linux::Key::Y,
    input_linux::Key::Z,
];

const QWERTY_MAPPING: KeyMapping = [
    input_linux::Key::Q,
    input_linux::Key::W,
    input_linux::Key::E,
    input_linux::Key::R,
    input_linux::Key::T,
    input_linux::Key::Y,
    input_linux::Key::U,
    input_linux::Key::I,
    input_linux::Key::O,
    input_linux::Key::P,
    input_linux::Key::A,
    input_linux::Key::S,
    input_linux::Key::D,
    input_linux::Key::F,
    input_linux::Key::G,
    input_linux::Key::H,
    input_linux::Key::J,
    input_linux::Key::K,
    input_linux::Key::L,
    input_linux::Key::Z,
    input_linux::Key::X,
    input_linux::Key::C,
    input_linux::Key::V,
    input_linux::Key::B,
    input_linux::Key::N,
    input_linux::Key::M,
];

struct KeyMap {
    mapping: KeyMapping,
}

impl KeyMap {
    fn new(mapping: KeyMapping) -> Self {
        KeyMap { mapping }
    }

    fn map_midi_to_key(&self, note: MidiNote) -> Option<input_linux::Key> {
        match MIDI_KEY_MAP.get(&note) {
            Some(Mapping::Fixed(key)) => Some(*key),
            Some(Mapping::Alphabet(index)) => Some(self.mapping[*index as usize]),
            None => None,
        }
    }
}

type MidiNote = u8;

lazy_static! {
    static ref MIDI_KEY_MAP: HashMap<MidiNote, Mapping> = vec![
        (36u8, Mapping::Alphabet(0)),
        (37u8, Mapping::Fixed(input_linux::Key::LeftCtrl)),
        (38u8, Mapping::Alphabet(1)),
        (39u8, Mapping::Fixed(input_linux::Key::LeftAlt)),
        (40u8, Mapping::Alphabet(2)),
        (41u8, Mapping::Alphabet(3)),
        (42u8, Mapping::Fixed(input_linux::Key::CapsLock)),
        (43u8, Mapping::Alphabet(4)),
        (44u8, Mapping::Fixed(input_linux::Key::Esc)),
        (45u8, Mapping::Alphabet(5)),
        (47u8, Mapping::Alphabet(6)),
    //
        (48u8, Mapping::Alphabet(7)),
        (49u8, Mapping::Fixed(input_linux::Key::Num1)),
        (50u8, Mapping::Alphabet(8)),
        (51u8, Mapping::Fixed(input_linux::Key::Num2)),
        (52u8, Mapping::Alphabet(9)),
        (53u8, Mapping::Alphabet(10)),
        (54u8, Mapping::Fixed(input_linux::Key::Num3)),
        (55u8, Mapping::Alphabet(11)),
        (56u8, Mapping::Fixed(input_linux::Key::Num4)),
        (57u8, Mapping::Alphabet(12)),
        (58u8, Mapping::Fixed(input_linux::Key::Num5)),
        (59u8, Mapping::Alphabet(13)),
    //
        (60u8, Mapping::Alphabet(14)),
        (61u8, Mapping::Fixed(input_linux::Key::Num6)),
        (62u8, Mapping::Alphabet(15)),
        (63u8, Mapping::Fixed(input_linux::Key::Num7)),
        (64u8, Mapping::Alphabet(16)),
        (65u8, Mapping::Alphabet(17)),
        (66u8, Mapping::Fixed(input_linux::Key::Num8)),
        (67u8, Mapping::Alphabet(18)),
        (68u8, Mapping::Fixed(input_linux::Key::Num9)),
        (69u8, Mapping::Alphabet(19)),
        (70u8, Mapping::Fixed(input_linux::Key::Num0)),
        (71u8, Mapping::Alphabet(20)),
    //
        (72u8, Mapping::Alphabet(21)),
        (73u8, Mapping::Fixed(input_linux::Key::Left)),
        (74u8, Mapping::Alphabet(22)),
        (75u8, Mapping::Fixed(input_linux::Key::Down)),
        (76u8, Mapping::Alphabet(23)),
        (77u8, Mapping::Alphabet(24)),
        (78u8, Mapping::Fixed(input_linux::Key::Up)),
        (79u8, Mapping::Alphabet(25)),
        (80u8, Mapping::Fixed(input_linux::Key::Right)),
    //
        (81u8, Mapping::Fixed(input_linux::Key::Backspace)),
        (82u8, Mapping::Fixed(input_linux::Key::Break)),
        (83u8, Mapping::Fixed(input_linux::Key::Space)),
        (84u8, Mapping::Fixed(input_linux::Key::Enter)),
    ]
    .into_iter()
    .collect();
}

fn init_midi() -> (jack::Client, Arc<Mutex<jack::Port<jack::MidiIn>>>) {
    let (client, _status) =
        jack::Client::new("midihid", jack::ClientOptions::NO_START_SERVER).unwrap();

    let midi_receiver = client
        .register_port("keyboard", jack::MidiIn::default())
        .unwrap();

    (client, Arc::new(Mutex::new(midi_receiver)))
}

fn make_uhandle(keymap: &KeyMap) -> UInputHandle<std::fs::File> {
    let uinput_file = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(O_NONBLOCK)
        .open("/dev/uinput")
        .expect("cannot read uinput");
    let uhandle = UInputHandle::new(uinput_file);

    uhandle.set_evbit(EventKind::Key).unwrap();
    for (note, _) in MIDI_KEY_MAP.iter() {
        if let Some(key) = keymap.map_midi_to_key(*note) {
            uhandle.set_keybit(key).unwrap();
        }
    }
    uhandle.set_keybit(input_linux::Key::LeftShift).unwrap();

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

struct State {
    uhandle: UInputHandle<std::fs::File>,
    keymap: KeyMap,
    was_shift: bool,
}

fn process_key(state: &mut State, key: u8, pressed: bool, velocity: u8) {
    let mut time: EventTime = EventTime::new(0, 0);
    if let Some(uinput_key) = state.keymap.map_midi_to_key(key) {
        let shift = pressed && velocity > 100;
        let mut events = Vec::with_capacity(6);
        if shift != state.was_shift {
            state.was_shift = shift;
            events.push(
                *InputEvent::from(KeyEvent::new(
                    time,
                    input_linux::Key::LeftShift,
                    KeyState::pressed(shift),
                ))
                .as_raw(),
            );
            time.tv_usec += 10000;
            events.push(
                *InputEvent::from(SynchronizeEvent::new(time, SynchronizeKind::Report, 0)).as_raw(),
            );
            state
                .uhandle
                .write(&events)
                .expect("Failed to write events");
            events.clear();
            time.tv_usec += 10000;
        }
        events.push(
            *InputEvent::from(KeyEvent::new(time, uinput_key, KeyState::pressed(pressed))).as_raw(),
        );
        time.tv_usec += 10000;
        events.push(
            *InputEvent::from(SynchronizeEvent::new(time, SynchronizeKind::Report, 0)).as_raw(),
        );
        time.tv_usec += 10000;
        state
            .uhandle
            .write(&events)
            .expect("Failed to write events");
        events.clear();
        time.tv_usec += 10000;
        if shift && !pressed {
            state.was_shift = false;
            events.push(
                *InputEvent::from(KeyEvent::new(
                    time,
                    input_linux::Key::LeftShift,
                    KeyState::pressed(false),
                ))
                .as_raw(),
            );
            time.tv_usec += 10000;
            events.push(
                *InputEvent::from(SynchronizeEvent::new(time, SynchronizeKind::Report, 0)).as_raw(),
            );
            state
                .uhandle
                .write(&events)
                .expect("Failed to write events");
        }
        let insert_shift_message = if shift { " with shift" } else { "" };
        println!("{key}->{uinput_key:?}{insert_shift_message}");
    } else {
        println!("Unhandled input: {key}");
    }
}

fn work(args: Args) {
    let (client, midi_receiver) = init_midi();

    let keymap = args.keymap.get_keymap();

    let uhandle = make_uhandle(&keymap);

    let state = Arc::new(Mutex::new(State {
        keymap: keymap,
        uhandle,
        was_shift: false,
    }));

    let jack_callback = {
        //let state = Arc::clone(&state);
        let midi_receiver = Arc::clone(&midi_receiver);
        move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            let state = state.clone();
            let mut state = state.lock().expect("state cannot be locked");
            let midi_receiver = Arc::clone(&midi_receiver);
            let midi = midi_receiver.lock().expect("Cannot lock midi");
            for event in midi.iter(ps) {
                match event.bytes {
                    [248] => (),
                    [0x90, key, velocity] => process_key(&mut state, *key, true, *velocity),
                    [0x80, key, velocity] => process_key(&mut state, *key, false, *velocity),
                    _ => (),
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
                    .filter(|&x| match &args.input {
                        Some(input) => input == x,
                        None => false,
                    })
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

fn main() {
    work(Args::parse());
}
