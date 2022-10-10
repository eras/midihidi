Copyright 2022 Erkki Seppälä <erkki.seppala@vincit.fi>

Licensed under the [`MIT license`](LICENSE.MIT).

* midihidi

`midihidi` converts a MIDI keyboard to a regular `evdev`-supported one
by reading MIDI data (via Jack or the Jack layer of PipeWire) and
providing them via `/dev/uinput` to `/dev/input/event?`. This provides
the ability to enter keyboard input to any Linux application,
including virtual consoles.

* Setup

You need to arrange both Jack/PipeWire and uinput to be accessible by
the sabme user. Most easily this can be achieved by changing to
ownership of `/dev/uinput` to have the same user id as your current
user id:

```
sudo chown $USER /dev/uinput
```

The best alternative would be to have special user that is able to
access the device via ownership or group ownership, and also able to
interact with the user's media. I haven't yet tried how to make this
happen..

* Usage
