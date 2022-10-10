Copyright 2022 Erkki Seppälä <erkki.seppala@vincit.fi>

Licensed under the [`MIT license`](LICENSE.MIT).

# midihidi

`midihidi` converts a MIDI keyboard to a regular `evdev`-supported one
by reading MIDI data (via Jack or the Jack layer of PipeWire) and
providing them via `/dev/uinput` to `/dev/input/event?`. This provides
the ability to enter keyboard input to any Linux application,
including virtual consoles.

# Installing

Grab a binary built on Linux x86/64 Ubuntu from Releases, `chmod +x`
it and move it to a directory of your choice.

You can also compile it yourself. To compile it you first need have a
the Rust compiler. If your operating system doesn't come with the Rust
compiler, or the compiler is too old, then the easiest way to install
one is to use [`rustup`](https://rustup.rs/).

Once the compiler (and `cargo`, the Rust package manager) is
installed, you can just run the command `cargo install --git
https://github.com/eras/midihidi` to install the latest version to
`~/.cargo/bin`.

# Setup

You need to arrange both Jack (or PipeWire) and `/dev/uinput` to be
accessible by the same user. The easiest __and least secure way__ to
achieve this is by changing the ownership of `/dev/uinput` to have the
same user id as your current user id:

```
sudo chown $USER /dev/uinput
```

This change is temporary and will be reverted on boot. You can add an
`udev` rule to make permanent changes to the permissions. Be aware
that after this change any process you own is able to behave as if it
were a mouse or a keyboard.

The secure alternative would be to have special user that is able to
access the device via ownership or group ownership, and also able to
interact with the user's media. I haven't yet tried how to make this
happen..

One step easier, but still probably slightly involved due to media
access permissions, would be to run `midihidi` as root. I would
consider this secure.

# Usage

For basic use you can use 
```
midihidi --keymap alphabet
```

If you want it to automatically connect to some MIDI input, you can also use like

```
midihidi --keymap alphabet --input "Novation SL MkIII:Novation SL MkIII MIDI 1"
```

Once running, you can make use of the following mapping. On my
keyboard with zero transposition, the lowest key of the 49-key is key
36, while the highest key is 84.

Alphabet(0) is either the key 'a' or the key 'q', on the 'alphabet'
and 'qwerty' mappings, correspondingly.

```
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
```
